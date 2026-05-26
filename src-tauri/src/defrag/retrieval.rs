//! Dosya extent (parça) analizi — FSCTL_GET_RETRIEVAL_POINTERS.
//!
//! Bir dosyanın VCN→LCN eşlemesini okur; extent (bitişik küme bloğu) sayısı
//! parçalanmanın ölçüsüdür (1 extent = bütünleşik). [`analyze`] birimi yürüyüp
//! parçalanma raporu üretir; [`get_layout`] tek dosyanın extent'lerini verir
//! (engine taşıma için kullanır).

use std::time::Instant;

use crate::error::DMedicResult;
use crate::models::{FileFrag, FragmentationReport};

/// Bitişik bir küme bloğu.
#[derive(Debug, Clone, Copy)]
pub struct Extent {
    pub lcn: u64,
    pub len: u64,
}

/// Bir dosyanın küme yerleşimi.
pub struct FileLayout {
    pub extents: Vec<Extent>,
    pub total_clusters: u64,
}

impl FileLayout {
    pub fn fragment_count(&self) -> u32 {
        self.extents.len() as u32
    }
}

/// Analiz sırasında dolaşılacak azami dosya sayısı (yanıt süresini sınırlar).
const MAX_FILES: u64 = 300_000;
/// "En parçalı" listesinde tutulacak öğe sayısı.
const TOP_N: usize = 50;

#[cfg(windows)]
pub fn get_layout(handle: windows::Win32::Foundation::HANDLE) -> Option<FileLayout> {
    use std::ffi::c_void;
    use windows::core::HRESULT;
    use windows::Win32::Foundation::{ERROR_HANDLE_EOF, ERROR_MORE_DATA};
    use windows::Win32::System::Ioctl::{
        FSCTL_GET_RETRIEVAL_POINTERS, STARTING_VCN_INPUT_BUFFER,
    };
    use windows::Win32::System::IO::DeviceIoControl;

    const BUF: usize = 64 * 1024;
    let more_data = HRESULT::from_win32(ERROR_MORE_DATA.0);
    let eof = HRESULT::from_win32(ERROR_HANDLE_EOF.0);

    let mut extents: Vec<Extent> = Vec::new();
    let mut starting_vcn: i64 = 0;
    let mut prev_vcn: i64 = 0;
    let mut guard = 0u32;

    loop {
        guard += 1;
        if guard > 10_000 {
            break;
        }
        let input = STARTING_VCN_INPUT_BUFFER {
            StartingVcn: starting_vcn,
        };
        let mut out = vec![0u8; BUF];
        let mut returned: u32 = 0;
        let res = unsafe {
            DeviceIoControl(
                handle,
                FSCTL_GET_RETRIEVAL_POINTERS,
                Some(&input as *const _ as *const c_void),
                std::mem::size_of::<STARTING_VCN_INPUT_BUFFER>() as u32,
                Some(out.as_mut_ptr() as *mut c_void),
                BUF as u32,
                Some(&mut returned),
                None,
            )
        };

        let is_more = match &res {
            Ok(()) => false,
            Err(e) if e.code() == more_data => true,
            // Resident (MFT içi) veya boş dosya — extent yok, atla.
            Err(e) if e.code() == eof => return None,
            Err(_) => return None,
        };

        if (returned as usize) < 16 {
            break;
        }
        let extent_count = u32::from_le_bytes(out[0..4].try_into().unwrap());
        let base_vcn = i64::from_le_bytes(out[8..16].try_into().unwrap());
        prev_vcn = base_vcn;

        for i in 0..extent_count as usize {
            let off = 16 + i * 16;
            if off + 16 > returned as usize {
                break;
            }
            let next_vcn = i64::from_le_bytes(out[off..off + 8].try_into().unwrap());
            let lcn = i64::from_le_bytes(out[off + 8..off + 16].try_into().unwrap());
            let len = (next_vcn - prev_vcn).max(0) as u64;
            // lcn = -1 → seyrek/sanal blok, taşınmaz; saymayız.
            if lcn >= 0 && len > 0 {
                extents.push(Extent {
                    lcn: lcn as u64,
                    len,
                });
            }
            prev_vcn = next_vcn;
        }

        if is_more {
            starting_vcn = prev_vcn;
        } else {
            break;
        }
    }

    Some(FileLayout {
        extents,
        total_clusters: prev_vcn.max(0) as u64,
    })
}

/// İlerleme bildirimi sıklığı (dosya sayısı).
const PROGRESS_EVERY: u64 = 1500;

/// Birimi yürü, parçalanma raporu üret. `on_progress(taranan_dosya, son_yol)`
/// periyodik olarak çağrılır — UI canlı ilerleme gösterir.
#[cfg(windows)]
pub fn analyze<F>(letter: char, on_progress: F) -> DMedicResult<FragmentationReport>
where
    F: Fn(u64, Option<&str>),
{
    use super::volume::{self, FileRef};

    let started = Instant::now();
    let root = format!("{}:\\", letter.to_ascii_uppercase());

    let mut total_files: u64 = 0;
    let mut fragmented_files: u64 = 0;
    let mut frag_list: Vec<FileFrag> = Vec::new();

    for entry in walkdir::WalkDir::new(&root)
        .follow_links(false)
        .into_iter()
        .filter_map(Result::ok)
    {
        if total_files >= MAX_FILES {
            break;
        }
        // Reparse noktalarını (junction/symlink) atla.
        if entry.path_is_symlink() {
            continue;
        }
        let ftype = entry.file_type();
        if !ftype.is_file() {
            continue;
        }
        total_files += 1;

        let path = entry.path();
        if total_files % PROGRESS_EVERY == 0 {
            on_progress(total_files, path.to_str());
        }

        let Ok(file) = FileRef::open(path) else {
            continue;
        };
        let Some(layout) = get_layout(file.raw()) else {
            continue;
        };
        let frags = layout.fragment_count();
        if frags > 1 {
            fragmented_files += 1;
            let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
            frag_list.push(FileFrag {
                path: path.display().to_string(),
                fragments: frags,
                size_bytes: size,
            });
        }
    }
    on_progress(total_files, None);

    frag_list.sort_by(|a, b| b.fragments.cmp(&a.fragments));
    frag_list.truncate(TOP_N);

    let pct = if total_files > 0 {
        (fragmented_files as f32 / total_files as f32) * 100.0
    } else {
        0.0
    };

    let vol = volume::describe(letter);
    let recommendation = if vol.media_type == "SSD" {
        "SSD algılandı — full defrag önerilmez (yalnız analiz/TRIM).".to_string()
    } else if pct >= 10.0 {
        "Parçalanma yüksek — defrag önerilir.".to_string()
    } else if pct >= 3.0 {
        "Orta düzey parçalanma — hızlı defrag yararlı olabilir.".to_string()
    } else {
        "Parçalanma düşük — defrag gerekmez.".to_string()
    };

    Ok(FragmentationReport {
        letter: letter.to_string(),
        fragmentation_percent: (pct * 10.0).round() / 10.0,
        total_files,
        fragmented_files,
        most_fragmented: frag_list,
        elapsed_ms: started.elapsed().as_millis() as u64,
        recommendation,
    })
}

#[cfg(not(windows))]
pub fn analyze<F>(letter: char, _on_progress: F) -> DMedicResult<FragmentationReport>
where
    F: Fn(u64, Option<&str>),
{
    Ok(FragmentationReport {
        letter: letter.to_string(),
        ..Default::default()
    })
}
