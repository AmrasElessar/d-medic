use crate::error::DMedicResult;
use crate::models::Snapshot;
use crate::snapshot;

#[tauri::command]
pub async fn list_snapshots() -> DMedicResult<Vec<Snapshot>> {
    snapshot::storage::list_all().await
}

#[tauri::command]
pub async fn create_snapshot(description: String) -> DMedicResult<Snapshot> {
    let services = ["SysMain", "DiagTrack", "WSearch"];
    snapshot::capture_full(&description, &services).await
}

#[tauri::command]
pub async fn rollback_snapshot(id: String) -> DMedicResult<()> {
    let snap = snapshot::storage::load(&id).await?;
    snapshot::service_state::restore(&snap.service_states).await?;
    Ok(())
}

#[tauri::command]
pub async fn delete_snapshot(id: String) -> DMedicResult<()> {
    snapshot::storage::delete(&id).await
}
