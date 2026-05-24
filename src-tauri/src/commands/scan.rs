use chrono::Utc;
use serde::Serialize;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

use crate::diagnostic::{self, ScanEvent, ScanEventKind};
use crate::error::DMedicResult;
use crate::models::{ScanKind, ScanResult};

/// Frontend tarafında `listen("scan-progress", ...)` ile yakalanan event gövdesi.
/// Aynı tarama içindeki tüm event'ler aynı `scan_id`'yi taşır.
#[derive(Debug, Serialize, Clone)]
struct ScanProgressPayload<'a> {
    scan_id: &'a str,
    kind: &'static str,
    index: usize,
    total: usize,
    check_id: &'static str,
    /// "started" | "finished"
    status: &'static str,
    /// Finished anında doldurulur: bu check kaç finding üretti, başarılı mıydı.
    success: Option<bool>,
    finding_count: Option<usize>,
}

fn payload_from_event<'a>(
    scan_id: &'a str,
    kind: &'static str,
    evt: &ScanEvent,
) -> ScanProgressPayload<'a> {
    let (status, success, finding_count) = match evt.kind {
        ScanEventKind::Started => ("started", None, None),
        ScanEventKind::Finished { success, finding_count } => {
            ("finished", Some(success), Some(finding_count))
        }
    };
    ScanProgressPayload {
        scan_id,
        kind,
        index: evt.index,
        total: evt.total,
        check_id: evt.check_id,
        status,
        success,
        finding_count,
    }
}

async fn run_scan_with_progress(
    app: AppHandle,
    scan_kind: ScanKind,
    kind_str: &'static str,
) -> DMedicResult<ScanResult> {
    let started = Utc::now();
    let scan_id = Uuid::new_v4().to_string();
    tracing::info!(scan_kind = kind_str, scan_id = %scan_id, "Tarama başlatıldı");

    let app_for_cb = app.clone();
    let scan_id_for_cb = scan_id.clone();
    let findings = diagnostic::run_with_progress(scan_kind, move |evt| {
        let payload = payload_from_event(&scan_id_for_cb, kind_str, &evt);
        if let Err(e) = app_for_cb.emit("scan-progress", &payload) {
            tracing::warn!(error = %e, "scan-progress emit failed");
        }
    })
    .await?;

    let finished = Utc::now();
    let duration_ms = (finished - started).num_milliseconds();
    tracing::info!(
        scan_kind = kind_str,
        scan_id = %scan_id,
        finding_count = findings.len(),
        duration_ms = duration_ms,
        "Tarama tamamlandı"
    );
    Ok(ScanResult {
        scan_id,
        kind: scan_kind,
        started_at: started,
        finished_at: finished,
        findings,
    })
}

#[tauri::command]
pub async fn quick_scan(app: AppHandle) -> DMedicResult<ScanResult> {
    run_scan_with_progress(app, ScanKind::Quick, "quick").await
}

#[tauri::command]
pub async fn deep_scan(app: AppHandle) -> DMedicResult<ScanResult> {
    run_scan_with_progress(app, ScanKind::Deep, "deep").await
}
