//! Defrag taşıma motoru — FSCTL_MOVE_FILE ile dosya konsolidasyonu.
//!
//! Strateji (Quick/Full): parçalı her dosya için bitmap'te yeterli bitişik boş
//! aralık bulunur ve dosyanın tüm küme aralığı (VCN 0..N) o hedefe tek çağrıda
//! taşınır — NTFS bütünleşik yerleştirir. Taşıma `FSCTL_MOVE_FILE` ile yapılır;
//! NTFS bunu journaller, yarıda kesilse veri kaybı olmaz.
//!
//! Güvenlik: SSD'de full defrag reddedilir; MFT zone hedef alınmaz; iptal her
//! dosya sınırında kontrol edilir.

use tauri::{AppHandle, Emitter};

use crate::error::{DMedicError, DMedicResult};
use crate::models::{DefragMode, DefragProgress};

/// Tek çağrıda taşınacak azami küme (≈4 GB @ 4K). Daha büyük parçalı dosyalar
/// atlanır (uzun tek taşıma yerine güvenli tarafta kal).
#[cfg(windows)]
const MAX_MOVE_CLUSTERS: u64 = 1 << 20;

pub async fn run(app: AppHandle, letter: char, mode: DefragMode) -> DMedicResult<()> {
    super::reset_cancel();
    let job_id = uuid::Uuid::new_v4().to_string();

    // Analiz-only: taşıma yok.
    if matches!(mode, DefragMode::AnalyzeOnly) {
        emit(&app, &progress(&job_id, "analyzing", None, 0, 0, 0, 0.0));
        emit(&app, &progress(&job_id, "done", None, 0, 0, 0, 100.0));
        return Ok(());
    }

    #[cfg(windows)]
    {
        let app2 = app.clone();
        let job = job_id.clone();
        tokio::task::spawn_blocking(move || run_blocking(app2, job, letter, mode))
            .await
            .map_err(|e| DMedicError::Other(format!("defrag join: {e}")))?
    }
    #[cfg(not(windows))]
    {
        let _ = (letter, mode);
        emit(&app, &progress(&job_id, "done", None, 0, 0, 0, 100.0));
        Ok(())
    }
}

#[cfg(windows)]
struct Candidate {
    path: std::path::PathBuf,
    total_clusters: u64,
    extents: Vec<super::retrieval::Extent>,
}

#[cfg(windows)]
fn run_blocking(
    app: AppHandle,
    job_id: String,
    letter: char,
    _mode: DefragMode,
) -> DMedicResult<()> {
    use super::{bitmap, retrieval, volume};

    // SSD / desteklenmeyen birim koruması.
    let vol = volume::describe(letter);
    if !vol.defrag_supported {
        emit(&app, &progress(&job_id, "error", None, 0, 0, 0, 0.0));
        return Err(DMedicError::Validation(format!(
            "{}: defrag desteklenmiyor (SSD veya NTFS değil)",
            vol.letter
        )));
    }

    emit(&app, &progress(&job_id, "analyzing", None, 0, 0, 0, 0.0));

    let geo = volume::geometry(letter)?;
    let mft_zone = (
        geo.mft_zone_start.max(0) as u64,
        (geo.mft_zone_end - geo.mft_zone_start).max(0) as u64,
    );
    let mut bm = bitmap::read(letter)?;

    // Pass 1 — parçalı dosya adaylarını topla.
    let root = format!("{}:\\", letter.to_ascii_uppercase());
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut clusters_total: u64 = 0;

    for entry in walkdir::WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if super::is_cancelled() {
            emit(&app, &progress(&job_id, "cancelled", None, 0, clusters_total, 0, 0.0));
            return Ok(());
        }
        if entry.path_is_symlink() || !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Ok(file) = volume::FileRef::open(path) else {
            continue;
        };
        let Some(layout) = retrieval::get_layout(file.raw()) else {
            continue;
        };
        if layout.fragment_count() > 1
            && layout.total_clusters > 0
            && layout.total_clusters <= MAX_MOVE_CLUSTERS
        {
            clusters_total += layout.total_clusters;
            candidates.push(Candidate {
                path: path.to_path_buf(),
                total_clusters: layout.total_clusters,
                extents: layout.extents,
            });
        }
    }

    // Pass 2 — taşı.
    let mut clusters_moved: u64 = 0;
    let mut files_processed: u64 = 0;

    for cand in &candidates {
        if super::is_cancelled() {
            emit(
                &app,
                &progress(&job_id, "cancelled", None, clusters_moved, clusters_total, files_processed, pct(clusters_moved, clusters_total)),
            );
            return Ok(());
        }
        files_processed += 1;
        emit(
            &app,
            &progress(
                &job_id,
                "moving",
                Some(cand.path.display().to_string()),
                clusters_moved,
                clusters_total,
                files_processed,
                pct(clusters_moved, clusters_total),
            ),
        );

        // Kaçınılacak aralıklar: dosyanın kendi extent'leri + MFT zone.
        let mut avoid: Vec<(u64, u64)> = cand.extents.iter().map(|e| (e.lcn, e.len)).collect();
        if mft_zone.1 > 0 {
            avoid.push(mft_zone);
        }

        let Some(target) = bm.find_free_run(cand.total_clusters, 0, &avoid) else {
            continue; // yeterli bitişik boş alan yok — atla.
        };

        let Ok(file) = volume::FileRef::open(&cand.path) else {
            continue;
        };
        match move_file(letter, file.raw(), target, cand.total_clusters) {
            Ok(()) => {
                // Bitmap güncelle: eski extent'leri boşalt, hedefi doldur.
                for e in &cand.extents {
                    for c in e.lcn..e.lcn + e.len {
                        bm.set_used(c, false);
                    }
                }
                for c in target..target + cand.total_clusters {
                    bm.set_used(c, true);
                }
                clusters_moved += cand.total_clusters;
            }
            Err(e) => {
                tracing::debug!(path = %cand.path.display(), error = %e, "taşıma atlandı");
            }
        }
    }

    emit(
        &app,
        &progress(&job_id, "done", None, clusters_moved, clusters_total, files_processed, 100.0),
    );
    Ok(())
}

/// FSCTL_MOVE_FILE — dosyanın VCN 0..count aralığını hedef LCN'e taşı.
#[cfg(windows)]
fn move_file(
    letter: char,
    file_handle: windows::Win32::Foundation::HANDLE,
    target_lcn: u64,
    count: u64,
) -> DMedicResult<()> {
    use std::ffi::c_void;
    use windows::Win32::System::Ioctl::{FSCTL_MOVE_FILE, MOVE_FILE_DATA};
    use windows::Win32::System::IO::DeviceIoControl;

    // Taşıma birim (volume) handle'ı üzerinden yapılır — yazma erişimi gerekir.
    let vol = super::volume::VolumeHandle::open(letter, true)?;

    let data = MOVE_FILE_DATA {
        FileHandle: file_handle,
        StartingVcn: 0,
        StartingLcn: target_lcn as i64,
        ClusterCount: count as u32,
    };
    let mut returned: u32 = 0;
    unsafe {
        DeviceIoControl(
            vol.raw(),
            FSCTL_MOVE_FILE,
            Some(&data as *const _ as *const c_void),
            std::mem::size_of::<MOVE_FILE_DATA>() as u32,
            None,
            0,
            Some(&mut returned),
            None,
        )
    }
    .map_err(|e| DMedicError::Other(format!("FSCTL_MOVE_FILE: {e}")))
}

fn pct(moved: u64, total: u64) -> f32 {
    if total == 0 {
        100.0
    } else {
        ((moved as f32 / total as f32) * 100.0).min(100.0)
    }
}

#[allow(clippy::too_many_arguments)]
fn progress(
    job_id: &str,
    phase: &str,
    current_file: Option<String>,
    clusters_moved: u64,
    clusters_total: u64,
    files_processed: u64,
    percent: f32,
) -> DefragProgress {
    DefragProgress {
        job_id: job_id.to_string(),
        phase: phase.to_string(),
        current_file,
        clusters_moved,
        clusters_total,
        files_processed,
        percent,
    }
}

fn emit(app: &AppHandle, p: &DefragProgress) {
    if let Err(e) = app.emit("defrag-progress", p) {
        tracing::warn!(error = %e, "defrag-progress emit failed");
    }
}
