//! Birim cluster tahsis bitmap'i (FSCTL_GET_VOLUME_BITMAP).
//!
//! FSCTL tek çağrıda tüm bitmap'i vermez: tampon dolunca `ERROR_MORE_DATA` ile
//! döner ama tamponu geçerli veriyle doldurur. Bu yüzden `StartingLcn`'i her
//! turda ilerleterek döngüyle tüm birimi okuruz.

use crate::error::DMedicResult;

#[cfg(windows)]
use crate::error::DMedicError;

/// Birimin küme tahsis durumu — bit başına bir küme (1=dolu).
#[derive(Default)]
pub struct VolumeBitmap {
    pub start_lcn: u64,
    pub bit_count: u64,
    pub bits: Vec<u8>,
}

impl VolumeBitmap {
    /// LCN dolu mu (sınır dışı → false).
    pub fn is_used(&self, lcn: u64) -> bool {
        if lcn < self.start_lcn {
            return false;
        }
        let idx = lcn - self.start_lcn;
        if idx >= self.bit_count {
            return false;
        }
        let byte = (idx / 8) as usize;
        let bit = (idx % 8) as u8;
        self.bits
            .get(byte)
            .map(|b| (b >> bit) & 1 == 1)
            .unwrap_or(false)
    }

    /// LCN'i dolu/boş işaretle (engine taşıma sonrası in-memory günceller).
    pub fn set_used(&mut self, lcn: u64, used: bool) {
        if lcn < self.start_lcn {
            return;
        }
        let idx = lcn - self.start_lcn;
        if idx >= self.bit_count {
            return;
        }
        let byte = (idx / 8) as usize;
        let bit = (idx % 8) as u8;
        if let Some(b) = self.bits.get_mut(byte) {
            if used {
                *b |= 1 << bit;
            } else {
                *b &= !(1 << bit);
            }
        }
    }

    /// `len` uzunlukta, `from_lcn`'den itibaren ilk boş bitişik aralığı bul.
    /// Hiçbiri yoksa None. `avoid` aralıklarıyla çakışanları atlar.
    pub fn find_free_run(&self, len: u64, from_lcn: u64, avoid: &[(u64, u64)]) -> Option<u64> {
        if len == 0 {
            return None;
        }
        let end = self.start_lcn + self.bit_count;
        let mut lcn = from_lcn.max(self.start_lcn);
        while lcn + len <= end {
            // lcn'den len boyunca boş mu?
            let mut ok = true;
            let mut probe = lcn;
            while probe < lcn + len {
                if self.is_used(probe) || in_any(probe, avoid) {
                    ok = false;
                    break;
                }
                probe += 1;
            }
            if ok {
                return Some(lcn);
            }
            // Engellenen kümeden sonrasına atla.
            lcn = probe + 1;
        }
        None
    }
}

fn in_any(lcn: u64, ranges: &[(u64, u64)]) -> bool {
    ranges.iter().any(|&(start, len)| lcn >= start && lcn < start + len)
}

#[cfg(windows)]
pub fn read(letter: char) -> DMedicResult<VolumeBitmap> {
    use std::ffi::c_void;
    use windows::core::HRESULT;
    use windows::Win32::Foundation::ERROR_MORE_DATA;
    use windows::Win32::System::Ioctl::{FSCTL_GET_VOLUME_BITMAP, STARTING_LCN_INPUT_BUFFER};
    use windows::Win32::System::IO::DeviceIoControl;

    use super::volume::VolumeHandle;

    let handle = VolumeHandle::open(letter, false)?;

    // 1 MB tampon → çağrı başına ~8M küme.
    const BUF_BYTES: usize = 1 << 20;
    const HDR: usize = 16; // StartingLcn(i64) + BitmapSize(i64)

    let mut all_bits: Vec<u8> = Vec::new();
    let mut start_lcn: i64 = 0;
    let mut total_clusters: u64 = 0;
    let more_data = HRESULT::from_win32(ERROR_MORE_DATA.0);
    let mut guard = 0u32;

    loop {
        guard += 1;
        if guard > 100_000 {
            break; // güvenlik freni
        }
        let input = STARTING_LCN_INPUT_BUFFER {
            StartingLcn: start_lcn,
        };
        let mut out = vec![0u8; BUF_BYTES];
        let mut returned: u32 = 0;
        let res = unsafe {
            DeviceIoControl(
                handle.raw(),
                FSCTL_GET_VOLUME_BITMAP,
                Some(&input as *const _ as *const c_void),
                std::mem::size_of::<STARTING_LCN_INPUT_BUFFER>() as u32,
                Some(out.as_mut_ptr() as *mut c_void),
                BUF_BYTES as u32,
                Some(&mut returned),
                None,
            )
        };

        let is_more = match &res {
            Ok(()) => false,
            Err(e) if e.code() == more_data => true,
            Err(e) => return Err(DMedicError::Other(format!("FSCTL_GET_VOLUME_BITMAP: {e}"))),
        };

        if (returned as usize) < HDR {
            break;
        }
        let hdr_start = i64::from_le_bytes(out[0..8].try_into().unwrap());
        let bitmap_size = i64::from_le_bytes(out[8..16].try_into().unwrap()).max(0) as u64;
        if total_clusters == 0 {
            // İlk çağrı: hdr_start=0 → BitmapSize = toplam küme sayısı.
            total_clusters = hdr_start.max(0) as u64 + bitmap_size;
        }

        let avail_bytes = (returned as usize) - HDR;
        all_bits.extend_from_slice(&out[HDR..HDR + avail_bytes]);

        // Bu turda kapsanan küme = bit sayısı (byte*8), kalanla sınırlı.
        let covered = (avail_bytes as u64) * 8;
        start_lcn = hdr_start + covered as i64;

        if !is_more {
            break;
        }
    }

    Ok(VolumeBitmap {
        start_lcn: 0,
        bit_count: total_clusters,
        bits: all_bits,
    })
}

#[cfg(not(windows))]
pub fn read(_letter: char) -> DMedicResult<VolumeBitmap> {
    Ok(VolumeBitmap::default())
}
