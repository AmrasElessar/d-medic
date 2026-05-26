//! Defrag IOCTL motoru — IPC komutları.

use tauri::{AppHandle, Emitter};

use crate::defrag;
use crate::error::{DMedicError, DMedicResult};
use crate::models::{ClusterMap, DefragMode, DefragProgress, FragmentationReport, VolumeInfo};

fn parse_letter(s: &str) -> DMedicResult<char> {
    s.chars()
        .next()
        .filter(|c| c.is_ascii_alphabetic())
        .ok_or_else(|| DMedicError::Validation(format!("geçersiz sürücü harfi: {s}")))
}

/// Tüm sabit birimleri (HDD/SSD + NTFS bilgisiyle) listele.
#[tauri::command]
pub async fn list_volumes() -> DMedicResult<Vec<VolumeInfo>> {
    tokio::task::spawn_blocking(defrag::volume::list)
        .await
        .map_err(|e| DMedicError::Other(format!("birim liste join: {e}")))?
}

/// Bir birimin parçalanma analizini çalıştır. Tarama uzun sürebilir; her ~1500
/// dosyada `defrag-progress` (phase="analyzing") event'i yayılır.
#[tauri::command]
pub async fn analyze_volume(app: AppHandle, letter: String) -> DMedicResult<FragmentationReport> {
    let c = parse_letter(&letter)?;
    let app2 = app.clone();
    tokio::task::spawn_blocking(move || {
        defrag::retrieval::analyze(c, |count, path| {
            let payload = DefragProgress {
                job_id: "analyze".to_string(),
                phase: "analyzing".to_string(),
                current_file: path.map(|s| s.to_string()),
                clusters_moved: 0,
                clusters_total: 0,
                files_processed: count,
                percent: 0.0,
            };
            let _ = app2.emit("defrag-progress", &payload);
        })
    })
    .await
    .map_err(|e| DMedicError::Other(format!("analiz join: {e}")))?
}

/// UI cluster haritası (downsample edilmiş grid).
#[tauri::command]
pub async fn get_cluster_map(letter: String, resolution: u32) -> DMedicResult<ClusterMap> {
    let c = parse_letter(&letter)?;
    tokio::task::spawn_blocking(move || defrag::map::cluster_map(c, resolution))
        .await
        .map_err(|e| DMedicError::Other(format!("harita join: {e}")))?
}

/// Defrag işini başlat (event'lerle ilerler).
#[tauri::command]
pub async fn start_defrag(app: AppHandle, letter: String, mode: DefragMode) -> DMedicResult<()> {
    let c = parse_letter(&letter)?;
    defrag::engine::run(app, c, mode).await
}

/// Çalışan defrag'i iptal et.
#[tauri::command]
pub fn cancel_defrag() -> DMedicResult<()> {
    defrag::request_cancel();
    Ok(())
}
