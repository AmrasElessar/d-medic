//! Kaldırma SONRASI derin kalıntı taraması.
//!
//! İki yüzey:
//!  * **Registry** — HKCU/HKLM `SOFTWARE` altındaki ürün/yayıncı anahtarları,
//!    `Run` otomatik başlatma değerleri.
//!  * **Dosya sistemi** — Program Files, ProgramData, AppData (Roaming/Local),
//!    Public, Başlat menüsü kısayolları.
//!
//! Her bulgu bir [`LeftoverConfidence`] ile etiketlenir (ad eşleşmesinin
//! gücüne göre). Hiçbir şey burada SİLİNMEZ — yalnızca aday listesi üretilir;
//! silme [`super::remove`] içindedir.

use std::path::{Path, PathBuf};

use crate::error::DMedicResult;
use crate::models::{
    InstalledProgram, LeftoverConfidence, LeftoverItem, LeftoverKind, LeftoverScanResult,
};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Programın kalıntılarını ara. Bloklayıcı registry/FS işleri spawn_blocking'te.
pub async fn scan(program: InstalledProgram) -> DMedicResult<LeftoverScanResult> {
    tokio::task::spawn_blocking(move || scan_blocking(&program))
        .await
        .map_err(|e| crate::error::DMedicError::Other(format!("kalıntı tarama join: {e}")))?
}

/// Ad/yayıncıdan üretilen eşleştirme anahtarları.
struct Tokens {
    /// Alfanümerik-küçük harf ürün anahtarı (örn. "AcmeApp" → "acmeapp").
    product_key: String,
    /// Yayıncı anahtarı (varsa).
    publisher_key: Option<String>,
}

/// Bir dizeyi karşılaştırma anahtarına indirger: küçük harf + yalnız alfanümerik.
fn norm_key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Ürün adından sürüm/parantez gürültüsünü ayıklayıp anahtar üret.
fn product_token(name: &str) -> String {
    // "Acme App 3.2 (x64)" → "Acme App " → norm → "acmeapp"
    let cut = name
        .split(|c: char| c == '(' || c == '[')
        .next()
        .unwrap_or(name);
    let no_ver: String = cut
        .chars()
        .take_while(|c| !(c.is_ascii_digit() && cut.len() > 6))
        .collect();
    let base = if norm_key(&no_ver).len() >= 3 {
        no_ver
    } else {
        cut.to_string()
    };
    norm_key(&base)
}

fn build_tokens(program: &InstalledProgram) -> Tokens {
    let publisher_key = program
        .publisher
        .as_deref()
        .map(norm_key)
        .filter(|k| k.len() >= 4);
    Tokens {
        product_key: product_token(&program.name),
        publisher_key,
    }
}

/// Bir klasör/anahtar adının ürün anahtarıyla eşleşme güveni.
fn match_confidence(name: &str, tokens: &Tokens) -> Option<LeftoverConfidence> {
    let key = norm_key(name);
    if key.is_empty() || tokens.product_key.len() < 3 {
        return None;
    }
    if key == tokens.product_key {
        return Some(LeftoverConfidence::Safe);
    }
    if tokens.product_key.len() >= 4 && key.contains(&tokens.product_key) {
        return Some(LeftoverConfidence::Probable);
    }
    if key.len() >= 5 && tokens.product_key.contains(&key) {
        return Some(LeftoverConfidence::Possible);
    }
    None
}

fn is_publisher_dir(name: &str, tokens: &Tokens) -> bool {
    match &tokens.publisher_key {
        Some(pk) => {
            let k = norm_key(name);
            !k.is_empty() && (k == *pk || (pk.len() >= 5 && k.contains(pk.as_str())))
        }
        None => false,
    }
}

fn make_item(
    kind: LeftoverKind,
    confidence: LeftoverConfidence,
    path: String,
    value_name: Option<String>,
    size_bytes: Option<u64>,
    reason: impl Into<String>,
) -> LeftoverItem {
    let mut hasher = blake3::Hasher::new();
    hasher.update(path.as_bytes());
    if let Some(v) = &value_name {
        hasher.update(b"|");
        hasher.update(v.as_bytes());
    }
    let id = hasher.finalize().to_hex()[..16].to_string();
    let default_selected = matches!(
        confidence,
        LeftoverConfidence::Safe | LeftoverConfidence::Probable
    );
    LeftoverItem {
        id,
        kind,
        confidence,
        path,
        value_name,
        size_bytes,
        reason: reason.into(),
        default_selected,
    }
}

// ----------------------------------------------------------------------------
// Çekirdek (bloklayıcı)
// ----------------------------------------------------------------------------

#[cfg(windows)]
fn scan_blocking(program: &InstalledProgram) -> DMedicResult<LeftoverScanResult> {
    let tokens = build_tokens(program);
    let mut items: Vec<LeftoverItem> = Vec::new();
    let mut scanned_roots: Vec<String> = Vec::new();

    scan_filesystem(program, &tokens, &mut items, &mut scanned_roots);
    scan_registry(&tokens, &mut items, &mut scanned_roots);

    let registry_hits = items
        .iter()
        .filter(|i| matches!(i.kind, LeftoverKind::RegKey | LeftoverKind::RegValue))
        .count();
    let file_hits = items.len() - registry_hits;

    Ok(LeftoverScanResult {
        program_id: program.id.clone(),
        items,
        scanned_roots,
        registry_hits,
        file_hits,
    })
}

#[cfg(not(windows))]
fn scan_blocking(program: &InstalledProgram) -> DMedicResult<LeftoverScanResult> {
    Ok(LeftoverScanResult {
        program_id: program.id.clone(),
        ..Default::default()
    })
}

// ----------------------------------------------------------------------------
// Dosya sistemi
// ----------------------------------------------------------------------------

fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

#[cfg(windows)]
fn scan_filesystem(
    program: &InstalledProgram,
    tokens: &Tokens,
    items: &mut Vec<LeftoverItem>,
    scanned_roots: &mut Vec<String>,
) {
    // 1) Kurulum klasörü hâlâ duruyorsa en güçlü kalıntıdır.
    if let Some(loc) = &program.install_location {
        let p = PathBuf::from(loc);
        if p.is_dir() {
            items.push(make_item(
                LeftoverKind::Folder,
                LeftoverConfidence::Safe,
                p.display().to_string(),
                None,
                Some(dir_size(&p)),
                "Kurulum klasörü kaldırma sonrası hâlâ mevcut",
            ));
        }
    }

    // 2) Bilinen kökleri sığ derinlikte tara.
    let roots: Vec<(PathBuf, &str)> = [
        ("ProgramFiles", "Program Files"),
        ("ProgramFiles(x86)", "Program Files (x86)"),
        ("ProgramData", "ProgramData"),
        ("APPDATA", "AppData\\Roaming"),
        ("LOCALAPPDATA", "AppData\\Local"),
        ("PUBLIC", "Public"),
    ]
    .iter()
    .filter_map(|(env, label)| std::env::var_os(env).map(|v| (PathBuf::from(v), *label)))
    .collect();

    for (root, label) in &roots {
        if !root.is_dir() {
            continue;
        }
        scanned_roots.push(label.to_string());
        scan_root_dir(root, tokens, items, 0);
    }

    // 3) Başlat menüsü kısayolları (.lnk).
    let start_menus: Vec<PathBuf> = [
        std::env::var_os("APPDATA").map(|v| {
            PathBuf::from(v).join(r"Microsoft\Windows\Start Menu\Programs")
        }),
        std::env::var_os("ProgramData").map(|v| {
            PathBuf::from(v).join(r"Microsoft\Windows\Start Menu\Programs")
        }),
    ]
    .into_iter()
    .flatten()
    .collect();

    for sm in &start_menus {
        if sm.is_dir() {
            scan_start_menu(sm, tokens, items);
        }
    }
}

/// Bir kök altındaki dizinleri tara. Eşleşeni kalıntı işaretle; eşleşen ama
/// yayıncı klasörü olanların bir alt seviyesine in (Publisher\Product deseni).
#[cfg(windows)]
fn scan_root_dir(root: &Path, tokens: &Tokens, items: &mut Vec<LeftoverItem>, depth: usize) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.filter_map(Result::ok) {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();

        if let Some(conf) = match_confidence(&name, tokens) {
            // Aynı yolu iki kez eklemeyi önle.
            let ps = path.display().to_string();
            if !items.iter().any(|i| i.path == ps) {
                items.push(make_item(
                    LeftoverKind::Folder,
                    conf,
                    ps,
                    None,
                    Some(dir_size(&path)),
                    "Program adıyla eşleşen artık klasör",
                ));
            }
        } else if depth == 0 && is_publisher_dir(&name, tokens) {
            // Yayıncı klasörüne bir seviye in.
            scan_root_dir(&path, tokens, items, depth + 1);
        }
    }
}

#[cfg(windows)]
fn scan_start_menu(dir: &Path, tokens: &Tokens, items: &mut Vec<LeftoverItem>) {
    for entry in walkdir::WalkDir::new(dir)
        .max_depth(3)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        let is_lnk = path
            .extension()
            .map(|e| e.eq_ignore_ascii_case("lnk"))
            .unwrap_or(false);
        if !is_lnk {
            continue;
        }
        let stem = path.file_stem().map(|s| s.to_string_lossy().to_string());
        if let Some(stem) = stem {
            if let Some(conf) = match_confidence(&stem, tokens) {
                items.push(make_item(
                    LeftoverKind::File,
                    conf,
                    path.display().to_string(),
                    None,
                    path.metadata().ok().map(|m| m.len()),
                    "Başlat menüsü kısayolu",
                ));
            }
        }
    }
}

#[cfg(not(windows))]
fn scan_filesystem(
    _p: &InstalledProgram,
    _t: &Tokens,
    _i: &mut Vec<LeftoverItem>,
    _r: &mut Vec<String>,
) {
}

// ----------------------------------------------------------------------------
// Registry
// ----------------------------------------------------------------------------

#[cfg(windows)]
fn scan_registry(tokens: &Tokens, items: &mut Vec<LeftoverItem>, scanned_roots: &mut Vec<String>) {
    // Label tuple'a gömülü — winreg HKEY tipine bağımlılık yok.
    let sources = [
        (HKEY_CURRENT_USER, "HKCU", KEY_WOW64_64KEY),
        (HKEY_LOCAL_MACHINE, "HKLM", KEY_WOW64_64KEY),
        (HKEY_LOCAL_MACHINE, "HKLM", KEY_WOW64_32KEY),
    ];

    for (root, label, view) in sources {
        // SOFTWARE altındaki ürün/yayıncı anahtarları.
        if let Ok(software) =
            RegKey::predef(root).open_subkey_with_flags("SOFTWARE", KEY_READ | view)
        {
            scanned_roots.push(format!("{label}\\SOFTWARE"));
            for sub in software.enum_keys().filter_map(Result::ok) {
                if let Some(conf) = match_confidence(&sub, tokens) {
                    push_regkey(items, label, &format!("SOFTWARE\\{sub}"), conf,
                        "Program adıyla eşleşen registry anahtarı");
                } else if is_publisher_dir(&sub, tokens) {
                    // Yayıncı anahtarının altında ürün anahtarı ara.
                    if let Ok(pub_key) =
                        software.open_subkey_with_flags(&sub, KEY_READ | view)
                    {
                        for child in pub_key.enum_keys().filter_map(Result::ok) {
                            if let Some(conf) = match_confidence(&child, tokens) {
                                push_regkey(items, label,
                                    &format!("SOFTWARE\\{sub}\\{child}"), conf,
                                    "Yayıncı altında eşleşen ürün anahtarı");
                            }
                        }
                    }
                }
            }
        }

        // Run otomatik başlatma değerleri.
        let run_path = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
        if let Ok(run) = RegKey::predef(root).open_subkey_with_flags(run_path, KEY_READ | view) {
            for (vname, vval) in run.enum_values().filter_map(Result::ok) {
                let data = format!("{vval:?}");
                let hit = match_confidence(&vname, tokens).is_some()
                    || norm_key(&data).contains(&tokens.product_key);
                if hit && tokens.product_key.len() >= 4 {
                    items.push(make_item(
                        LeftoverKind::RegValue,
                        LeftoverConfidence::Probable,
                        format!("{label}\\{run_path}"),
                        Some(vname),
                        None,
                        "Otomatik başlatma (Run) kaydı",
                    ));
                }
            }
        }
    }
}

#[cfg(windows)]
fn push_regkey(
    items: &mut Vec<LeftoverItem>,
    label: &str,
    subpath: &str,
    conf: LeftoverConfidence,
    reason: &str,
) {
    let full = format!("{label}\\{subpath}");
    if !items.iter().any(|i| i.path == full) {
        items.push(make_item(LeftoverKind::RegKey, conf, full, None, None, reason));
    }
}

#[cfg(not(windows))]
fn scan_registry(_t: &Tokens, _i: &mut Vec<LeftoverItem>, _r: &mut Vec<String>) {}
