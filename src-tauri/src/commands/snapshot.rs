use crate::error::DMedicResult;
use crate::models::Snapshot;
use crate::snapshot::{self, RollbackReport};

/// D-Medic'in dokunabileceği tüm servisler — pre-action snapshot kapsamı.
/// `apply_action` öncesi bu liste yakalanır, gerekirse restore'da kullanılır.
pub const TRACKED_SERVICES: &[&str] = &[
    "SysMain",
    "WSearch",
    "DiagTrack",
    "dmwappushservice",
    "wuauserv",
    "bits",
    "cryptSvc",
    "msiserver",
];

#[tauri::command]
pub async fn list_snapshots() -> DMedicResult<Vec<Snapshot>> {
    snapshot::storage::list_all().await
}

#[tauri::command]
pub async fn create_snapshot(description: String) -> DMedicResult<Snapshot> {
    snapshot::capture_full(&description, TRACKED_SERVICES).await
}

#[tauri::command]
pub async fn rollback_snapshot(id: String) -> DMedicResult<RollbackReport> {
    snapshot::rollback_full(&id).await
}

#[tauri::command]
pub async fn delete_snapshot(id: String) -> DMedicResult<()> {
    snapshot::storage::delete(&id).await
}
