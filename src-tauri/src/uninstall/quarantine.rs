//! Silme güvenlik ağı: karantina deposu.
//!
//! Silinen dosya/klasörler doğrudan yok edilmez; `%APPDATA%\D-Medic\quarantine\<id>\`
//! altına taşınır, registry anahtarları silinmeden önce `.reg` export edilir.
//! Her parti bir `manifest.json` tutar — tam geri yükleme için yeterli. Partiler
//! [`RETENTION_DAYS`] gün sonra otomatik temizlenebilir.
//!
//! Tüm işlemler senkron (çağıran `spawn_blocking` içinden kullanır).

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::{DMedicError, DMedicResult};
use crate::models::QuarantineEntry;
use crate::paths;

/// Karantina partilerinin saklama süresi.
pub const RETENTION_DAYS: i64 = 14;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedFile {
    pub original_path: String,
    /// `files/` altındaki saklanan ad (çakışmasız indeksli).
    pub stored_name: String,
    pub is_dir: bool,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantinedReg {
    /// Örn. "HKLM\\SOFTWARE\\AcmeApp".
    pub original_path: String,
    /// `registry/` altındaki .reg dosya adı.
    pub reg_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineManifest {
    pub id: String,
    pub created_at: String,
    pub program_label: String,
    pub files: Vec<QuarantinedFile>,
    pub registry: Vec<QuarantinedReg>,
}

/// Aktif bir karantina partisini biriktiren yazıcı.
pub struct Batch {
    dir: PathBuf,
    manifest: QuarantineManifest,
    file_counter: usize,
    reg_counter: usize,
}

impl Batch {
    /// Yeni parti oluştur — zaman damgalı id, klasör iskeleti.
    pub fn create(program_label: &str) -> DMedicResult<Self> {
        let now = Utc::now();
        let id = format!("{}", now.format("%Y%m%d-%H%M%S-%3f"));
        let dir = quarantine_root()?.join(&id);
        fs::create_dir_all(dir.join("files"))?;
        fs::create_dir_all(dir.join("registry"))?;
        Ok(Self {
            dir,
            manifest: QuarantineManifest {
                id,
                created_at: now.to_rfc3339(),
                program_label: program_label.to_string(),
                files: Vec::new(),
                registry: Vec::new(),
            },
            file_counter: 0,
            reg_counter: 0,
        })
    }

    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// Bir dosya/klasörü karantinaya TAŞI (orijinal yerinden kaldırır).
    pub fn quarantine_path(&mut self, original: &Path) -> DMedicResult<u64> {
        let is_dir = original.is_dir();
        let size = if is_dir {
            dir_size(original)
        } else {
            original.metadata().map(|m| m.len()).unwrap_or(0)
        };
        let stem = original
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "öğe".to_string());
        let stored_name = format!("{:04}_{}", self.file_counter, sanitize(&stem));
        self.file_counter += 1;
        let dest = self.dir.join("files").join(&stored_name);

        move_path(original, &dest)?;

        self.manifest.files.push(QuarantinedFile {
            original_path: original.display().to_string(),
            stored_name,
            is_dir,
            size_bytes: size,
        });
        Ok(size)
    }

    /// Bir registry anahtarını .reg export et (silmeden ÖNCE çağrılmalı).
    /// "HKLM\\SOFTWARE\\X" → registry/NNNN.reg.
    pub fn export_reg(&mut self, reg_path: &str) -> DMedicResult<()> {
        let reg_file = format!("{:04}.reg", self.reg_counter);
        self.reg_counter += 1;
        let dest = self.dir.join("registry").join(&reg_file);
        reg_export(reg_path, &dest)?;
        self.manifest.registry.push(QuarantinedReg {
            original_path: reg_path.to_string(),
            reg_file,
        });
        Ok(())
    }

    /// Manifest'i diske yaz, parti özetini döndür.
    pub fn finalize(self) -> DMedicResult<QuarantineManifest> {
        let manifest_path = self.dir.join("manifest.json");
        let json = serde_json::to_string_pretty(&self.manifest)?;
        fs::write(manifest_path, json)?;
        Ok(self.manifest)
    }
}

// ----------------------------------------------------------------------------
// Liste / geri yükle / temizle
// ----------------------------------------------------------------------------

/// Tüm karantina partilerini özet olarak listele (yeniden eskiye).
pub fn list() -> DMedicResult<Vec<QuarantineEntry>> {
    let root = match quarantine_root() {
        Ok(r) if r.is_dir() => r,
        _ => return Ok(Vec::new()),
    };
    let mut out = Vec::new();
    for entry in fs::read_dir(root)?.filter_map(Result::ok) {
        if !entry.path().is_dir() {
            continue;
        }
        if let Ok(m) = read_manifest(&entry.path()) {
            out.push(to_entry(&m));
        }
    }
    out.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(out)
}

/// Bir partiyi geri yükle: dosyaları orijinal yerine taşı, .reg'leri import et.
pub fn restore(id: &str) -> DMedicResult<()> {
    let dir = quarantine_root()?.join(id);
    let manifest = read_manifest(&dir)?;

    for f in &manifest.files {
        let src = dir.join("files").join(&f.stored_name);
        let dest = PathBuf::from(&f.original_path);
        if let Some(parent) = dest.parent() {
            let _ = fs::create_dir_all(parent);
        }
        move_path(&src, &dest)?;
    }
    for r in &manifest.registry {
        let reg = dir.join("registry").join(&r.reg_file);
        reg_import(&reg)?;
    }
    // Geri yüklendi → partiyi temizle.
    purge(id)
}

/// Bir partiyi kalıcı sil (geri alınamaz).
pub fn purge(id: &str) -> DMedicResult<()> {
    let dir = quarantine_root()?.join(id);
    if dir.is_dir() {
        fs::remove_dir_all(dir)?;
    }
    Ok(())
}

/// Süresi dolmuş partileri temizle (uygulama açılışında çağrılabilir).
pub fn purge_expired() -> DMedicResult<usize> {
    let mut removed = 0;
    for e in list()? {
        if e.expires_in_days <= 0 {
            if purge(&e.id).is_ok() {
                removed += 1;
            }
        }
    }
    Ok(removed)
}

// ----------------------------------------------------------------------------
// Yardımcılar
// ----------------------------------------------------------------------------

fn quarantine_root() -> DMedicResult<PathBuf> {
    paths::quarantine_dir().map_err(|e| DMedicError::Other(e.to_string()))
}

fn read_manifest(dir: &Path) -> DMedicResult<QuarantineManifest> {
    let raw = fs::read_to_string(dir.join("manifest.json"))?;
    Ok(serde_json::from_str(&raw)?)
}

fn to_entry(m: &QuarantineManifest) -> QuarantineEntry {
    let total_bytes = m.files.iter().map(|f| f.size_bytes).sum();
    let expires_in_days = m
        .created_at
        .parse::<DateTime<Utc>>()
        .map(|created| RETENTION_DAYS - (Utc::now() - created).num_days())
        .unwrap_or(RETENTION_DAYS);
    QuarantineEntry {
        id: m.id.clone(),
        created_at: m.created_at.clone(),
        program_label: m.program_label.clone(),
        file_count: m.files.len(),
        reg_export_count: m.registry.len(),
        total_bytes,
        expires_in_days,
    }
}

fn sanitize(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>()
        .chars()
        .take(80)
        .collect()
}

fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(Result::ok)
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

/// Taşıma: önce rename (aynı birim), olmazsa kopyala + sil (birimler arası).
fn move_path(src: &Path, dest: &Path) -> DMedicResult<()> {
    if fs::rename(src, dest).is_ok() {
        return Ok(());
    }
    if src.is_dir() {
        copy_dir(src, dest)?;
        fs::remove_dir_all(src)?;
    } else {
        fs::copy(src, dest)?;
        fs::remove_file(src)?;
    }
    Ok(())
}

fn copy_dir(src: &Path, dest: &Path) -> DMedicResult<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)?.filter_map(Result::ok) {
        let from = entry.path();
        let to = dest.join(entry.file_name());
        if from.is_dir() {
            copy_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

/// reg.exe export — senkron. `.reg` UTF-16 LE BOM ile yazılır (import sorunsuz).
fn reg_export(reg_path: &str, dest: &Path) -> DMedicResult<()> {
    let output = std::process::Command::new("reg.exe")
        .args(["export", reg_path, &dest.to_string_lossy(), "/y"])
        .output()
        .map_err(|e| DMedicError::Other(format!("reg export spawn: {e}")))?;
    if !output.status.success() {
        return Err(DMedicError::Other(format!(
            "reg export başarısız: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}

fn reg_import(reg_file: &Path) -> DMedicResult<()> {
    let output = std::process::Command::new("reg.exe")
        .args(["import", &reg_file.to_string_lossy()])
        .output()
        .map_err(|e| DMedicError::Other(format!("reg import spawn: {e}")))?;
    if !output.status.success() {
        return Err(DMedicError::Other(format!(
            "reg import başarısız: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(())
}
