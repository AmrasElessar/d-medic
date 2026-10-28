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

use std::path::Path;
#[cfg(windows)]
use std::path::PathBuf;
use std::sync::Mutex;

#[cfg(windows)]
use super::matching::is_publisher_dir;
use super::matching::{self, Strength, Tokens};
use crate::error::DMedicResult;
use crate::models::{
    InstalledProgram, LeftoverConfidence, LeftoverItem, LeftoverKind, LeftoverScanResult,
};
use crate::safety::FsGuard;
#[cfg(windows)]
use crate::safety::{self, norm_key};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Son taramanın bulguları. Silme komutu frontend'den yalnız öğe id'si alır ve
/// yolu buradan çözer — webview'ın gönderdiği yol/anahtar dizesine güvenilmez.
static LAST_SCAN: Mutex<Vec<LeftoverItem>> = Mutex::new(Vec::new());

/// Programın kalıntılarını ara. Bloklayıcı registry/FS işleri spawn_blocking'te.
///
/// Program hâlâ kurulu görünüyorsa (kaldırıcı iptal edildi / sürüyor ya da
/// "yalnız tara" seçildi) hiçbir öğe varsayılan seçili gelmez.
pub async fn scan(program: InstalledProgram) -> DMedicResult<LeftoverScanResult> {
    let still_registered = super::inventory::is_still_installed(&program).await;
    let mut result = tokio::task::spawn_blocking(move || scan_blocking(&program))
        .await
        .map_err(|e| crate::error::DMedicError::Other(format!("kalıntı tarama join: {e}")))??;
    if still_registered {
        result.program_still_registered = true;
        for item in &mut result.items {
            item.default_selected = false;
        }
    }
    *LAST_SCAN.lock().unwrap_or_else(|e| e.into_inner()) = result.items.clone();
    Ok(result)
}

/// Son taramada bulunan öğeleri id ile çöz. Bilinmeyen id'ler `Err(id)` döner.
pub fn resolve_items(ids: &[String]) -> Vec<Result<LeftoverItem, String>> {
    let cache = LAST_SCAN.lock().unwrap_or_else(|e| e.into_inner());
    ids.iter()
        .map(|id| {
            cache
                .iter()
                .find(|i| &i.id == id)
                .cloned()
                .ok_or_else(|| id.clone())
        })
        .collect()
}

/// Kalıntı gerekçesine eşleşme açıklamasını ekle (UI'da "neden önerildi").
#[cfg(windows)]
fn with_note(base: &str, name: &str, tokens: &Tokens) -> String {
    match matching::match_strength(name, tokens) {
        Some(s) => format!("{base} — {}", matching::describe(s, tokens)),
        None => base.to_string(),
    }
}

/// Bir klasör/anahtar adının ürünle eşleşme güveni.
fn match_confidence(name: &str, tokens: &Tokens) -> Option<LeftoverConfidence> {
    matching::match_strength(name, tokens).map(|s| match s {
        Strength::Exact => LeftoverConfidence::Safe,
        Strength::Strong => LeftoverConfidence::Probable,
        Strength::Weak => LeftoverConfidence::Possible,
    })
}

fn build_tokens(program: &InstalledProgram) -> Tokens {
    matching::build_tokens(&program.name, program.publisher.as_deref())
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

    let guard = FsGuard::from_env();
    scan_filesystem(program, &tokens, &guard, &mut items, &mut scanned_roots);
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
        // `scan` kurulum durumunu ayrıca kontrol edip günceller.
        program_still_registered: false,
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
    guard: &FsGuard,
    items: &mut Vec<LeftoverItem>,
    scanned_roots: &mut Vec<String>,
) {
    // 1) Kurulum klasörü hâlâ duruyorsa güçlü bir kalıntıdır — ama
    //    InstallLocation'ı yanlış yazan kurulumlar var (`C:\Program Files`,
    //    ortak yayıncı klasörü…). Korumalı yollar atlanır; klasör adı ürünle
    //    eşleşmiyorsa yalnız "olası" olarak (varsayılan seçili değil) önerilir.
    if let Some(loc) = &program.install_location {
        let p = PathBuf::from(loc);
        if p.is_dir() {
            match guard.check(loc) {
                Err(reason) => {
                    tracing::warn!(path = %loc, %reason, "Kurulum klasörü korumalı, kalıntı önerilmedi");
                }
                Ok(()) => {
                    let leaf = p
                        .file_name()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    let (conf, reason) = match match_confidence(&leaf, tokens) {
                        Some(LeftoverConfidence::Safe | LeftoverConfidence::Probable) => (
                            LeftoverConfidence::Safe,
                            "Kurulum klasörü kaldırma sonrası hâlâ mevcut",
                        ),
                        _ => (
                            LeftoverConfidence::Possible,
                            "Kayıtlı kurulum klasörü (adı ürünle eşleşmiyor — dikkatle inceleyin)",
                        ),
                    };
                    items.push(make_item(
                        LeftoverKind::Folder,
                        conf,
                        p.display().to_string(),
                        None,
                        Some(dir_size(&p)),
                        reason,
                    ));
                }
            }
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
        scan_root_dir(root, tokens, guard, items, 0);
    }

    // 3) Başlat menüsü kısayolları (.lnk).
    let start_menus: Vec<PathBuf> = [
        std::env::var_os("APPDATA")
            .map(|v| PathBuf::from(v).join(r"Microsoft\Windows\Start Menu\Programs")),
        std::env::var_os("ProgramData")
            .map(|v| PathBuf::from(v).join(r"Microsoft\Windows\Start Menu\Programs")),
    ]
    .into_iter()
    .flatten()
    .collect();

    for sm in &start_menus {
        if sm.is_dir() {
            scan_start_menu(sm, tokens, guard, items);
        }
    }
}

/// Bir kök altındaki dizinleri tara. Eşleşeni kalıntı işaretle; eşleşen ama
/// yayıncı klasörü olanların bir alt seviyesine in (Publisher\Product deseni).
#[cfg(windows)]
fn scan_root_dir(
    root: &Path,
    tokens: &Tokens,
    guard: &FsGuard,
    items: &mut Vec<LeftoverItem>,
    depth: usize,
) {
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
            if guard.check(&ps).is_ok() && !items.iter().any(|i| i.path == ps) {
                items.push(make_item(
                    LeftoverKind::Folder,
                    conf,
                    ps,
                    None,
                    Some(dir_size(&path)),
                    with_note("Program adıyla eşleşen artık klasör", &name, tokens),
                ));
            }
        } else if depth == 0 && is_publisher_dir(&name, tokens) {
            // Yayıncı klasörüne bir seviye in.
            scan_root_dir(&path, tokens, guard, items, depth + 1);
        }
    }
}

#[cfg(windows)]
fn scan_start_menu(dir: &Path, tokens: &Tokens, guard: &FsGuard, items: &mut Vec<LeftoverItem>) {
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
            let lnk = path.display().to_string();
            if guard.check(&lnk).is_err() {
                continue;
            }
            if let Some(conf) = match_confidence(&stem, tokens) {
                items.push(make_item(
                    LeftoverKind::File,
                    conf,
                    lnk,
                    None,
                    path.metadata().ok().map(|m| m.len()),
                    with_note("Başlat menüsü kısayolu", &stem, tokens),
                ));
            }
        }
    }
}

#[cfg(not(windows))]
fn scan_filesystem(
    _p: &InstalledProgram,
    _t: &Tokens,
    _g: &FsGuard,
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
                    push_regkey(
                        items,
                        label,
                        &format!("SOFTWARE\\{sub}"),
                        conf,
                        &with_note("Program adıyla eşleşen registry anahtarı", &sub, tokens),
                    );
                } else if is_publisher_dir(&sub, tokens) {
                    // Yayıncı anahtarının altında ürün anahtarı ara.
                    if let Ok(pub_key) = software.open_subkey_with_flags(&sub, KEY_READ | view) {
                        for child in pub_key.enum_keys().filter_map(Result::ok) {
                            if let Some(conf) = match_confidence(&child, tokens) {
                                push_regkey(
                                    items,
                                    label,
                                    &format!("SOFTWARE\\{sub}\\{child}"),
                                    conf,
                                    &with_note(
                                        "Yayıncı altında eşleşen ürün anahtarı",
                                        &child,
                                        tokens,
                                    ),
                                );
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
                if hit && tokens.product_key.len() >= 4 && !vname.trim().is_empty() {
                    items.push(make_item(
                        LeftoverKind::RegValue,
                        LeftoverConfidence::Probable,
                        format!("{label}\\{run_path}"),
                        Some(vname),
                        None,
                        format!(
                            "Otomatik başlatma (Run) kaydı — değer adı veya komutu '{}' içeriyor",
                            tokens.product_key
                        ),
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
    if safety::check_reg_key(&full).is_ok() && !items.iter().any(|i| i.path == full) {
        items.push(make_item(
            LeftoverKind::RegKey,
            conf,
            full,
            None,
            None,
            reason,
        ));
    }
}

#[cfg(not(windows))]
fn scan_registry(_t: &Tokens, _i: &mut Vec<LeftoverItem>, _r: &mut Vec<String>) {}
