use chrono::Utc;
use uuid::Uuid;

use crate::diagnostic;
use crate::error::DMedicResult;
use crate::models::{ScanKind, ScanResult};

#[tauri::command]
pub async fn quick_scan() -> DMedicResult<ScanResult> {
    let started = Utc::now();
    tracing::info!(scan_kind = "quick", "Hızlı tarama başlatıldı");
    let findings = diagnostic::run_quick().await?;
    let finished = Utc::now();
    Ok(ScanResult {
        scan_id: Uuid::new_v4().to_string(),
        kind: ScanKind::Quick,
        started_at: started,
        finished_at: finished,
        findings,
    })
}

#[tauri::command]
pub async fn deep_scan() -> DMedicResult<ScanResult> {
    let started = Utc::now();
    tracing::info!(scan_kind = "deep", "Derin tarama başlatıldı");
    let findings = diagnostic::run_deep().await?;
    let finished = Utc::now();
    Ok(ScanResult {
        scan_id: Uuid::new_v4().to_string(),
        kind: ScanKind::Deep,
        started_at: started,
        finished_at: finished,
        findings,
    })
}
