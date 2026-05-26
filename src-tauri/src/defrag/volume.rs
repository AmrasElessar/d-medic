//! Birim açma + geometri + SSD tespiti.
//!
//! NTFS olup olmadığını `FSCTL_GET_NTFS_VOLUME_DATA`'nın başarısına bakarak
//! anlarız (ayrı `GetVolumeInformationW` gerekmez). SSD tespiti seek-penalty
//! IOCTL'i ile yapılır — SSD'de full defrag engellenir.

#![allow(clippy::missing_safety_doc)]

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

use crate::error::{DMedicError, DMedicResult};
use crate::models::VolumeInfo;

#[cfg(windows)]
use windows::core::PCWSTR;
#[cfg(windows)]
use windows::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};
#[cfg(windows)]
use windows::Win32::Storage::FileSystem::{
    CreateFileW, GetDiskFreeSpaceExW, GetDriveTypeW, GetLogicalDrives, FILE_FLAGS_AND_ATTRIBUTES,
    FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
};
#[cfg(windows)]
use windows::Win32::System::Ioctl::{
    PropertyStandardQuery, StorageDeviceSeekPenaltyProperty, DEVICE_SEEK_PENALTY_DESCRIPTOR,
    FSCTL_GET_NTFS_VOLUME_DATA, IOCTL_STORAGE_QUERY_PROPERTY, NTFS_VOLUME_DATA_BUFFER,
    STORAGE_PROPERTY_QUERY,
};
#[cfg(windows)]
use windows::Win32::System::IO::DeviceIoControl;

// CreateFileW dwDesiredAccess için ham erişim hakları (typed-wrapper sürüm
// farklarından kaçınmak için u32 literal).
#[cfg(windows)]
const GENERIC_READ_U32: u32 = 0x8000_0000;
#[cfg(windows)]
const GENERIC_WRITE_U32: u32 = 0x4000_0000;
/// GetDriveTypeW dönüşü: sabit disk (windows-rs bu sabiti dışa vermiyor).
#[cfg(windows)]
const DRIVE_FIXED: u32 = 3;

/// Birim handle'ı — Drop'ta kapanır.
#[cfg(windows)]
pub struct VolumeHandle(HANDLE);

#[cfg(windows)]
impl VolumeHandle {
    /// "\\.\C:" aç. `write=true` taşıma (MOVE_FILE) için GENERIC_WRITE de ister.
    pub fn open(letter: char, write: bool) -> DMedicResult<Self> {
        let path = format!(r"\\.\{}:", letter.to_ascii_uppercase());
        let wide: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
        let access = if write {
            GENERIC_READ_U32 | GENERIC_WRITE_U32
        } else {
            GENERIC_READ_U32
        };
        let handle = unsafe {
            CreateFileW(
                PCWSTR(wide.as_ptr()),
                access,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(0),
                HANDLE::default(),
            )
        }
        .map_err(|e| DMedicError::Other(format!("{path} açılamadı: {e}")))?;

        if handle == INVALID_HANDLE_VALUE {
            return Err(DMedicError::Other(format!("{path}: geçersiz handle")));
        }
        Ok(Self(handle))
    }

    pub fn raw(&self) -> HANDLE {
        self.0
    }
}

#[cfg(windows)]
impl Drop for VolumeHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Tek bir birimin NTFS geometrisi (cluster sayıları + boyut).
#[cfg(windows)]
pub struct NtfsGeometry {
    pub bytes_per_cluster: u32,
    pub total_clusters: u64,
    pub free_clusters: u64,
    /// MFT zone — taşınamaz bölge sınırları (LCN).
    pub mft_zone_start: i64,
    pub mft_zone_end: i64,
}

/// FSCTL_GET_NTFS_VOLUME_DATA. NTFS değilse Err.
#[cfg(windows)]
pub fn ntfs_geometry(handle: &VolumeHandle) -> DMedicResult<NtfsGeometry> {
    let mut data = NTFS_VOLUME_DATA_BUFFER::default();
    let mut returned: u32 = 0;
    let ok = unsafe {
        DeviceIoControl(
            handle.raw(),
            FSCTL_GET_NTFS_VOLUME_DATA,
            None,
            0,
            Some(&mut data as *mut _ as *mut std::ffi::c_void),
            std::mem::size_of::<NTFS_VOLUME_DATA_BUFFER>() as u32,
            Some(&mut returned),
            None,
        )
    };
    ok.map_err(|e| DMedicError::Other(format!("FSCTL_GET_NTFS_VOLUME_DATA: {e}")))?;
    Ok(NtfsGeometry {
        bytes_per_cluster: data.BytesPerCluster,
        total_clusters: data.TotalClusters.max(0) as u64,
        free_clusters: data.FreeClusters.max(0) as u64,
        mft_zone_start: data.MftZoneStart,
        mft_zone_end: data.MftZoneEnd,
    })
}

/// Birimin NTFS geometrisini tek çağrıda al (handle aç + sorgula).
#[cfg(windows)]
pub fn geometry(letter: char) -> DMedicResult<NtfsGeometry> {
    let handle = VolumeHandle::open(letter, false)?;
    ntfs_geometry(&handle)
}

/// Tek dosya/dizin handle'ı (retrieval pointers + MOVE_FILE için). Drop'ta kapanır.
#[cfg(windows)]
pub struct FileRef(HANDLE);

#[cfg(windows)]
impl FileRef {
    /// Dosyayı GENERIC_READ ile aç (FSCTL retrieval + MOVE_FILE yeterli).
    /// Paylaşımlı (read/write/delete) açılır ki kilitli dosyalarda da dene­yebilelim.
    pub fn open(path: &std::path::Path) -> DMedicResult<Self> {
        const FILE_FLAG_BACKUP_SEMANTICS: u32 = 0x0200_0000;
        const FILE_SHARE_DELETE_U: u32 = 0x0000_0004;
        let wide: Vec<u16> = path
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();
        let handle = unsafe {
            CreateFileW(
                PCWSTR(wide.as_ptr()),
                GENERIC_READ_U32,
                FILE_SHARE_READ | FILE_SHARE_WRITE | windows::Win32::Storage::FileSystem::FILE_SHARE_MODE(FILE_SHARE_DELETE_U),
                None,
                OPEN_EXISTING,
                FILE_FLAGS_AND_ATTRIBUTES(FILE_FLAG_BACKUP_SEMANTICS),
                HANDLE::default(),
            )
        }
        .map_err(|e| DMedicError::Other(format!("dosya açılamadı: {e}")))?;
        if handle == INVALID_HANDLE_VALUE {
            return Err(DMedicError::Other("geçersiz dosya handle".into()));
        }
        Ok(Self(handle))
    }

    pub fn raw(&self) -> HANDLE {
        self.0
    }
}

#[cfg(windows)]
impl Drop for FileRef {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// SSD tespiti: seek penalty yoksa SSD. None → belirlenemedi.
#[cfg(windows)]
fn incurs_seek_penalty(handle: &VolumeHandle) -> Option<bool> {
    let query = STORAGE_PROPERTY_QUERY {
        PropertyId: StorageDeviceSeekPenaltyProperty,
        QueryType: PropertyStandardQuery,
        AdditionalParameters: [0],
    };
    let mut desc = DEVICE_SEEK_PENALTY_DESCRIPTOR::default();
    let mut returned: u32 = 0;
    let ok = unsafe {
        DeviceIoControl(
            handle.raw(),
            IOCTL_STORAGE_QUERY_PROPERTY,
            Some(&query as *const _ as *const std::ffi::c_void),
            std::mem::size_of::<STORAGE_PROPERTY_QUERY>() as u32,
            Some(&mut desc as *mut _ as *mut std::ffi::c_void),
            std::mem::size_of::<DEVICE_SEEK_PENALTY_DESCRIPTOR>() as u32,
            Some(&mut returned),
            None,
        )
    };
    if ok.is_err() || returned == 0 {
        return None;
    }
    Some(desc.IncursSeekPenalty.as_bool())
}

/// Birimin toplam/boş byte değerleri (her dosya sistemi için çalışır).
#[cfg(windows)]
fn free_space_bytes(letter: char) -> (u64, u64) {
    let root = format!("{}:\\", letter.to_ascii_uppercase());
    let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
    let mut free_avail: u64 = 0;
    let mut total: u64 = 0;
    let mut total_free: u64 = 0;
    let ok = unsafe {
        GetDiskFreeSpaceExW(
            PCWSTR(wide.as_ptr()),
            Some(&mut free_avail),
            Some(&mut total),
            Some(&mut total_free),
        )
    };
    if ok.is_ok() {
        (total, total_free)
    } else {
        (0, 0)
    }
}

/// Tüm sabit (DRIVE_FIXED) birimleri listele.
#[cfg(windows)]
pub fn list() -> DMedicResult<Vec<VolumeInfo>> {
    let mask = unsafe { GetLogicalDrives() };
    let mut out = Vec::new();
    for i in 0..26u32 {
        if mask & (1 << i) == 0 {
            continue;
        }
        let letter = (b'A' + i as u8) as char;
        let root = format!("{letter}:\\");
        let wide: Vec<u16> = root.encode_utf16().chain(std::iter::once(0)).collect();
        let dtype = unsafe { GetDriveTypeW(PCWSTR(wide.as_ptr())) };
        if dtype != DRIVE_FIXED {
            continue;
        }
        out.push(describe(letter));
    }
    Ok(out)
}

/// Tek bir birimi VolumeInfo'ya çevir (analiz/komut için public).
#[cfg(windows)]
pub fn describe(letter: char) -> VolumeInfo {
    let (total_bytes, free_bytes) = free_space_bytes(letter);

    // NTFS geometrisi + SSD — handle açılabilirse.
    let (file_system, cluster_bytes, total_clusters, free_clusters, media_type) =
        match VolumeHandle::open(letter, false) {
            Ok(h) => {
                let seek = incurs_seek_penalty(&h);
                let media = match seek {
                    Some(true) => "HDD",
                    Some(false) => "SSD",
                    None => "Unknown",
                };
                match ntfs_geometry(&h) {
                    Ok(g) => (
                        "NTFS".to_string(),
                        g.bytes_per_cluster,
                        g.total_clusters,
                        g.free_clusters,
                        media.to_string(),
                    ),
                    Err(_) => ("Diğer".to_string(), 0, 0, 0, media.to_string()),
                }
            }
            Err(_) => ("Bilinmiyor".to_string(), 0, 0, 0, "Unknown".to_string()),
        };

    let is_ntfs = file_system == "NTFS";
    let defrag_supported = is_ntfs && media_type != "SSD";

    VolumeInfo {
        letter: letter.to_string(),
        file_system,
        media_type,
        total_bytes,
        free_bytes,
        cluster_bytes,
        total_clusters,
        free_clusters,
        defrag_supported,
    }
}

// ---- non-windows stub'ları -------------------------------------------------

#[cfg(not(windows))]
pub fn list() -> DMedicResult<Vec<VolumeInfo>> {
    Ok(Vec::new())
}

#[cfg(not(windows))]
pub fn describe(_letter: char) -> VolumeInfo {
    VolumeInfo {
        letter: String::new(),
        file_system: "N/A".into(),
        media_type: "N/A".into(),
        total_bytes: 0,
        free_bytes: 0,
        cluster_bytes: 0,
        total_clusters: 0,
        free_clusters: 0,
        defrag_supported: false,
    }
}
