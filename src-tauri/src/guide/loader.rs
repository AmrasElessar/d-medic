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
    let path = resource_dir.join("guides").join(format!("{id}.json"));
    let bytes = std::fs::read(&path)
        .map_err(|_| DMedicError::NotFound(format!("Kılavuz {id} bulunamadı")))?;
    Ok(serde_json::from_slice(&bytes)?)
}
