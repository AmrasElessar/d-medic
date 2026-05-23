//! Rollback altyapısı — 3 katman:
//!
//! 1. **Checkpoint-Computer** — sistemin global geri yükleme noktası (Windows
//!    System Restore). Driver+registry+system file düzeyinde reverse.
//! 2. **reg export** — D-Medic'in dokunduğu HKCU/HKLM dallarının .reg yedeği.
//!    Hedefli rollback için yeterli, hive dump'tan çok daha hafif.
//! 3. **Service state JSON** — Stop/Disable edilen servislerin orijinal
//!    StartupType + Status'unu kaydet, restore'da geri ver.
//!
//! UWP paket silimleri snapshot'a alınmaz — Remove-AppxPackage geri alınamaz
//! (Microsoft Store re-install yönlendirmesi yapılır).

pub mod reg_export;
pub mod restore_point;
pub mod service_state;
pub mod storage;

use chrono::Utc;
use uuid::Uuid;

use crate::error::DMedicResult;
use crate::models::{ServiceStateRecord, Snapshot};

/// Tam snapshot: restore point + reg export + servis state. Tüm katmanlar
/// best-effort — biri fail olsa diğerleri kaydedilir; Snapshot'tan hangisinin
/// başardığı görünür.
pub async fn capture_full(description: &str, services: &[&str]) -> DMedicResult<Snapshot> {
    let id = Uuid::new_v4().to_string();
    let now = Utc::now();

    let restore_ok = restore_point::create(description).await.unwrap_or(false);
    let reg_paths = reg_export::export_safe_hives(&id).await.unwrap_or_default();
    let service_states: Vec<ServiceStateRecord> = service_state::capture(services)
        .await
        .unwrap_or_default();

    let snap = Snapshot {
        id: id.clone(),
        timestamp: now,
        description: description.to_string(),
        restore_point_created: restore_ok,
        registry_export_paths: reg_paths,
        service_states,
    };
    storage::save(&snap).await?;
    tracing::info!(
        snapshot_id = %id,
        restore_point = restore_ok,
        reg_files = snap.registry_export_paths.len(),
        svc_count = snap.service_states.len(),
        "Snapshot oluşturuldu"
    );
    Ok(snap)
}

/// Tam rollback: reg import + servis state restore. Sistem restore point
/// kullanıcı tarafından `rstrui.exe` ile manuel başlatılır — uygulamadan
/// otomatik tetiklemek tehlikeli (kullanıcı oturumunu öldürür).
#[derive(Debug, Clone, serde::Serialize)]
pub struct RollbackReport {
    pub snapshot_id: String,
    pub registry_imports: Vec<(String, bool)>,
    pub services_restored: usize,
}

pub async fn rollback_full(id: &str) -> DMedicResult<RollbackReport> {
    let snap = storage::load(id).await?;
    let registry_imports = reg_export::import_safe_hives(&snap.registry_export_paths)
        .await
        .unwrap_or_default();
    service_state::restore(&snap.service_states).await?;
    let report = RollbackReport {
        snapshot_id: snap.id.clone(),
        registry_imports,
        services_restored: snap.service_states.len(),
    };
    tracing::info!(
        snapshot_id = %id,
        reg_imports = report.registry_imports.len(),
        svc = report.services_restored,
        "Rollback tamamlandı"
    );
    Ok(report)
}
