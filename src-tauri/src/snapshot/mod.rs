//! Rollback altyapısı — 3 katman.
//! Bkz. Teknik Dökümandaki "7.5 Geri Alma Mimarisi".

pub mod reg_export;
pub mod restore_point;
pub mod service_state;
pub mod storage;

use chrono::Utc;
use uuid::Uuid;

use crate::error::DMedicResult;
use crate::models::{ServiceStateRecord, Snapshot};

/// Tam snapshot oluştur: restore point + reg export + servis state.
pub async fn capture_full(description: &str, services: &[&str]) -> DMedicResult<Snapshot> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let restore_ok = restore_point::create(description).await.unwrap_or(false);
    let reg_paths = reg_export::export_safe_hives(&id).await.unwrap_or_default();
    let service_states: Vec<ServiceStateRecord> = service_state::capture(services).await.unwrap_or_default();

    let snap = Snapshot {
        id: id.clone(),
        timestamp: now,
        description: description.to_string(),
        restore_point_created: restore_ok,
        registry_export_paths: reg_paths,
        service_states,
    };
    storage::save(&snap).await?;
    Ok(snap)
}
