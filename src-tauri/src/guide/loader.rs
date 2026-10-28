//! Kılavuz JSON dosyalarını oku. Tauri resource path'i kullanılır.

use std::path::PathBuf;

use crate::error::{DMedicError, DMedicResult};

use super::schema::Guide;

/// Tauri resource dizini içinden tüm `*.json` kılavuzlarını yükle.
pub async fn list_all(resource_dir: PathBuf) -> DMedicResult<Vec<Guide>> {
    let dir = resource_dir.join("guides");
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(bytes) = std::fs::read(&path) {
                match serde_json::from_slice::<Guide>(&bytes) {
                    Ok(g) => out.push(g),
                    Err(e) => tracing::warn!(
                        path = %path.display(),
                        error = %e,
                        "Kılavuz parse edilemedi"
                    ),
                }
            }
        }
    }
    out.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(out)
}

pub async fn load_one(resource_dir: PathBuf, id: &str) -> DMedicResult<Guide> {
    // id IPC'den gelir ve dosya yoluna eklenir — `..\` ile dışarı çıkılmasın.
    if !crate::safety::is_valid_storage_id(id) {
        return Err(DMedicError::Validation(format!("geçersiz kılavuz id: {id}")));
    }
    let path = resource_dir.join("guides").join(format!("{id}.json"));
    let bytes = std::fs::read(&path)
        .map_err(|_| DMedicError::NotFound(format!("Kılavuz {id} bulunamadı")))?;
    Ok(serde_json::from_slice(&bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guides_dir() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources")
    }

    /// Paketlenen her kılavuz şemaya uymalı ve dosya adı id'siyle aynı olmalı —
    /// aksi halde `list_all` onu sessizce atlar / `load_one` bulamaz.
    #[test]
    fn every_bundled_guide_parses_and_matches_its_file_name() {
        let dir = guides_dir().join("guides");
        let mut count = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().map(|e| e != "json").unwrap_or(true) {
                continue;
            }
            let bytes = std::fs::read(&path).unwrap();
            let guide: Guide = serde_json::from_slice(&bytes)
                .unwrap_or_else(|e| panic!("{} parse edilemedi: {e}", path.display()));
            let stem = path.file_stem().unwrap().to_string_lossy();
            assert_eq!(guide.id, stem, "{} id/dosya adı uyuşmuyor", path.display());
            count += 1;
        }
        assert!(count > 0, "hiç kılavuz bulunamadı");
    }

    #[test]
    fn load_one_rejects_traversal() {
        let rt = tokio::runtime::Builder::new_current_thread().build().unwrap();
        let res = rt.block_on(load_one(guides_dir(), r"..\..\Cargo"));
        assert!(matches!(res, Err(DMedicError::Validation(_))));
    }
}
