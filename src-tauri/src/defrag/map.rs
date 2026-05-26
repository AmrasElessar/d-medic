//! Cluster bitmap → UI grid downsample'ı.
//!
//! Birim milyonlarca küme içerir; UI'da her hücre binlerce kümeyi temsil eder.
//! Her hücre, kapsadığı kümeleri örnekleyip baskın duruma göre renklenir
//! (boş / dolu / parçalı(kısmen dolu) / taşınamaz=MFT zone).

use crate::error::DMedicResult;
use crate::models::{CellState, ClusterMap};

/// Hücre başına azami örnek (büyük aralıklarda hız için).
const MAX_SAMPLES_PER_CELL: u64 = 256;
/// Varsayılan grid genişliği (sütun).
const COLS: u32 = 64;

pub fn cluster_map(letter: char, target_cells: u32) -> DMedicResult<ClusterMap> {
    #[cfg(windows)]
    {
        build(letter, target_cells)
    }
    #[cfg(not(windows))]
    {
        let _ = target_cells;
        Ok(ClusterMap {
            letter: letter.to_string(),
            ..Default::default()
        })
    }
}

#[cfg(windows)]
fn build(letter: char, target_cells: u32) -> DMedicResult<ClusterMap> {
    use super::{bitmap, volume};

    let bm = bitmap::read(letter)?;
    let total = bm.bit_count.max(1);

    // MFT zone (taşınamaz) sınırları — geometri başarısızsa yok say.
    let (mft_start, mft_end) = match volume::geometry(letter) {
        Ok(g) => (g.mft_zone_start.max(0) as u64, g.mft_zone_end.max(0) as u64),
        Err(_) => (0, 0),
    };

    let cols = COLS;
    let rows = (target_cells / cols).max(1);
    let n_cells = (cols * rows) as u64;
    let cpc = (total + n_cells - 1) / n_cells; // ceil
    let cpc = cpc.max(1);

    let mut cells = Vec::with_capacity(n_cells as usize);
    for cell in 0..n_cells {
        let begin = cell * cpc;
        let end = (begin + cpc).min(total);
        if begin >= total {
            cells.push(CellState::Free);
            continue;
        }

        // MFT zone ile örtüşüyor mu (çoğunlukla)?
        let mid = (begin + end) / 2;
        if mft_end > mft_start && mid >= mft_start && mid < mft_end {
            cells.push(CellState::Unmovable);
            continue;
        }

        // Aralığı örnekle.
        let span = end - begin;
        let stride = (span / MAX_SAMPLES_PER_CELL).max(1);
        let mut samples = 0u64;
        let mut used = 0u64;
        let mut lcn = begin;
        while lcn < end {
            samples += 1;
            if bm.is_used(lcn) {
                used += 1;
            }
            lcn += stride;
        }
        let ratio = if samples > 0 {
            used as f32 / samples as f32
        } else {
            0.0
        };
        let state = if ratio == 0.0 {
            CellState::Free
        } else if ratio >= 0.9 {
            CellState::Used
        } else {
            // Kısmen dolu bölge — görsel olarak "parçalı" tonu.
            CellState::Fragmented
        };
        cells.push(state);
    }

    Ok(ClusterMap {
        letter: letter.to_string(),
        cols,
        rows,
        cells,
        clusters_per_cell: cpc,
    })
}
