//! Snapshot JSON serileştirme — %APPDATA%\D-Medic\snapshots\<id>.json

use crate::error::{DMedicError, DMedicResult};
use crate::models::Snapshot;
use crate::paths;

pub async fn save(snapshot: &Snapshot) -> DMedicResult<()> {
    let dir = paths::snapshot_dir().map_err(|e| DMedicError::Other(e.to_string()))?;
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(format!("{}.json", snapshot.id));
    let json = serde_json::to_vec_pretty(snapshot)?;
    std::fs::write(&path, json)?;
    Ok(())
}

pub async fn load(id: &str) -> DMedicResult<Snapshot> {
    let dir = paths::snapshot_dir().map_err(|e| DMedicError::Other(e.to_string()))?;
    let path = dir.join(format!("{id}.json"));
    let bytes = std::fs::read(&path)
        .map_err(|_| DMedicError::NotFound(format!("Snapshot {id} bulunamadı")))?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub async fn list_all() -> DMedicResult<Vec<Snapshot>> {
    let dir = paths::snapshot_dir().map_err(|e| DMedicError::Other(e.to_string()))?;
    if !dir.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        if entry.path().extension().map(|e| e == "json").unwrap_or(false) {
            if let Ok(bytes) = std::fs::read(entry.path()) {
                if let Ok(snap) = serde_json::from_slice::<Snapshot>(&bytes) {
                    out.push(snap);
                }
            }
        }
    }
    out.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    Ok(out)
}

pub async fn delete(id: &str) -> DMedicResult<()> {
    let dir = paths::snapshot_dir().map_err(|e| DMedicError::Other(e.to_string()))?;
    let path = dir.join(format!("{id}.json"));
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
