//! WMI native helper'ları — windows-rs üzerine wmi crate ile native COM çağrısı.
//!
//! PowerShell spawn etmeden RAM/CPU/Disk bilgisi okur. WMI çağrıları senkron
//! olduğu için `tokio::task::spawn_blocking` içinde sarmalanır.
//!
//! Namespace haritası:
//! - `ROOT\CIMV2` — Win32_PhysicalMemory, Win32_OperatingSystem, Win32_Processor,
//!   Win32_LogicalDisk
//! - `ROOT\Microsoft\Windows\Storage` — MSFT_PhysicalDisk (MediaType: HDD/SSD)

use serde::{Deserialize, Serialize};
use wmi::{COMLibrary, WMIConnection};

use crate::error::{DMedicError, DMedicResult};

#[derive(Debug, Default, Clone, Serialize)]
pub struct SystemSnapshot {
    pub total_ram_gb: f32,
    pub free_ram_gb: f32,
    pub cpu_name: String,
    pub primary_disk_type: String,
    pub primary_disk_size_gb: f32,
    pub primary_disk_free_gb: f32,
    /// Win32_DeviceGuard.VirtualizationBasedSecurityStatus == 2 → çalışıyor.
    pub vbs_running: bool,
    /// EFI System Partition boyutu (MB). 0 = bilinmiyor / Legacy BIOS sistem.
    pub efi_size_mb: u64,
    pub efi_free_mb: u64,
    /// Get-AppxPackage sayısı (mevcut kullanıcı için). 0 = sayım yapılamadı.
    pub uwp_package_count: u32,
    /// HKLM + HKCU Uninstall altında kayıtlı program sayısı (yaklaşık).
    pub installed_app_count: u32,
}

// wmi crate, query<T>() yaparken T'nin Rust adını WMI sınıf adı olarak kullanır.
// `Win32_PhysicalMemory` gibi underscore'lu WMI isimlerini eşleyebilmek için
// `#[serde(rename = "...")]` ile gerçek sınıf adını veriyoruz; aksi halde
// WBEM_E_INVALID_CLASS (0x80041010) hatası.

#[derive(Deserialize)]
#[serde(rename = "Win32_PhysicalMemory", rename_all = "PascalCase")]
struct Win32PhysicalMemory {
    capacity: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename = "Win32_OperatingSystem", rename_all = "PascalCase")]
struct Win32OperatingSystem {
    free_physical_memory: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename = "Win32_Processor", rename_all = "PascalCase")]
struct Win32Processor {
    name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename = "Win32_LogicalDisk", rename_all = "PascalCase")]
struct Win32LogicalDisk {
    device_id: Option<String>,
    drive_type: Option<u32>,
    size: Option<u64>,
    free_space: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename = "MSFT_PhysicalDisk", rename_all = "PascalCase")]
struct MsftPhysicalDisk {
    media_type: Option<u16>,
}

#[derive(Deserialize)]
#[serde(rename = "Win32_DeviceGuard", rename_all = "PascalCase")]
struct Win32DeviceGuard {
    /// 0: off, 1: configured but not running, 2: running.
    virtualization_based_security_status: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename = "MSFT_Partition", rename_all = "PascalCase")]
struct MsftPartition {
    /// 1=Unknown, 2=Logical, ..., "System" tipli partition ESP'dir.
    /// MSFT_Partition.Type enum'da string; ama wmi crate sadece numeric döner —
    /// GptType GUID alanını kullanıyoruz (ESP GUID sabit).
    gpt_type: Option<String>,
    size: Option<u64>,
}

const BYTES_PER_GB: f64 = 1024.0 * 1024.0 * 1024.0;

/// COMLibrary helper. tokio'nun spawn_blocking thread havuzundaki bir thread
/// daha önce başka bir check tarafından init edilmiş olabilir; bu durumda
/// `new()` "already initialized" hatası verir ve `assume_initialized` ile
/// devam etmek güvenlidir. İlk çağrıdaki gerçek init de bu yol üzerinden
/// geçer.
pub(crate) fn init_com_lib() -> COMLibrary {
    match COMLibrary::new() {
        Ok(c) => c,
        // SAFETY: COMLibrary::new() failure → bu thread'de COM zaten init edildi
        // (idempotent CoInitializeEx call). Marker döndürmek güvenli.
        Err(_) => unsafe { COMLibrary::assume_initialized() },
    }
}

/// Sistem genel snapshot'ını WMI üzerinden al. WMI sorgularını sync thread'de
/// yapar, PS-bağımlı sayımları (UWP) ayrı bir async spawn ile ekler. Toplam
/// 1-3 saniye; ilk Dashboard yüklemesinde non-blocking.
pub async fn read_snapshot() -> DMedicResult<SystemSnapshot> {
    let mut snap = tokio::task::spawn_blocking(read_snapshot_blocking)
        .await
        .map_err(|e| DMedicError::Other(format!("wmi spawn_blocking join: {e}")))??;

    // UWP package count — PS spawn'lı sayım, fail olursa 0 kalır.
    snap.uwp_package_count = count_uwp_packages().await.unwrap_or(0);

    // Installed app count — sync registry enum, hızlı.
    snap.installed_app_count = count_installed_apps_blocking().unwrap_or(0);

    Ok(snap)
}

/// Get-AppxPackage'i çağırıp satır sayısını döner. Mevcut kullanıcı kapsamı —
/// `-AllUsers` admin gerektirir, dev'de fail eder; basit non-elevated sayım.
async fn count_uwp_packages() -> Option<u32> {
    let out = crate::ps::runner::run_script(
        "(Get-AppxPackage -ErrorAction SilentlyContinue | Measure-Object).Count",
    )
    .await
    .ok()?;
    out.stdout.trim().parse::<u32>().ok()
}

/// HKLM + HKCU Uninstall altındaki alt anahtar sayısı = yüklü program sayısı.
/// Win32_Product YAVAŞ (MSI repair tetikler), bu yöntem ms cinsinden döner.
fn count_installed_apps_blocking() -> Option<u32> {
    use windows::core::{PCWSTR, PWSTR};
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{
        RegCloseKey, RegOpenKeyExW, RegQueryInfoKeyW, HKEY, KEY_READ,
    };
    use crate::diagnostic::registry::{HKCU, HKLM};

    fn count_subkeys(hive: HKEY, subkey: &str) -> u32 {
        let wide: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
        let mut h: HKEY = HKEY::default();
        if unsafe { RegOpenKeyExW(hive, PCWSTR(wide.as_ptr()), 0, KEY_READ, &mut h) } != ERROR_SUCCESS
        {
            return 0;
        }
        let mut count: u32 = 0;
        // PWSTR(null_mut) → class adı yazılmasın; sadece subkey sayısı isteniyor.
        unsafe {
            let _ = RegQueryInfoKeyW(
                h,
                PWSTR(std::ptr::null_mut()),
                None,
                None,
                Some(&mut count),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );
            let _ = RegCloseKey(h);
        }
        count
    }

    let n = count_subkeys(HKLM, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
        + count_subkeys(HKLM, "Software\\WOW6432Node\\Microsoft\\Windows\\CurrentVersion\\Uninstall")
        + count_subkeys(HKCU, "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall");
    Some(n)
}

fn read_snapshot_blocking() -> DMedicResult<SystemSnapshot> {
    let com = init_com_lib();
    let cimv2 =
        WMIConnection::new(com).map_err(|e| DMedicError::Wmi(format!("CIMV2 connect: {e}")))?;

    let mut snap = SystemSnapshot::default();

    match cimv2.query::<Win32PhysicalMemory>() {
        Ok(modules) => {
            let total_bytes: u64 = modules.iter().filter_map(|m| m.capacity).sum();
            snap.total_ram_gb = (total_bytes as f64 / BYTES_PER_GB) as f32;
        }
        Err(e) => tracing::warn!(error = %e, "Win32_PhysicalMemory query failed"),
    }

    match cimv2.query::<Win32OperatingSystem>() {
        Ok(rows) => {
            if let Some(os) = rows.into_iter().next() {
                if let Some(free_kb) = os.free_physical_memory {
                    snap.free_ram_gb = (free_kb as f64 / (1024.0 * 1024.0)) as f32;
                }
            }
        }
        Err(e) => tracing::warn!(error = %e, "Win32_OperatingSystem query failed"),
    }

    match cimv2.query::<Win32Processor>() {
        Ok(cpus) => {
            snap.cpu_name = cpus
                .into_iter()
                .filter_map(|c| c.name)
                .next()
                .unwrap_or_default()
                .trim()
                .to_string();
        }
        Err(e) => tracing::warn!(error = %e, "Win32_Processor query failed"),
    }

    match cimv2.query::<Win32LogicalDisk>() {
        Ok(disks) => {
            let primary = disks
                .into_iter()
                .filter(|d| d.drive_type == Some(3))
                .find(|d| d.device_id.as_deref() == Some("C:"));
            if let Some(d) = primary {
                if let Some(s) = d.size {
                    snap.primary_disk_size_gb = (s as f64 / BYTES_PER_GB) as f32;
                }
                if let Some(f) = d.free_space {
                    snap.primary_disk_free_gb = (f as f64 / BYTES_PER_GB) as f32;
                }
            }
        }
        Err(e) => tracing::warn!(error = %e, "Win32_LogicalDisk query failed"),
    }

    let storage_com = init_com_lib();
    match WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\Storage", storage_com) {
        Ok(storage) => match storage.query::<MsftPhysicalDisk>() {
            Ok(disks) => {
                if let Some(first) = disks.into_iter().next() {
                    snap.primary_disk_type = match first.media_type {
                        Some(3) => "HDD",
                        Some(4) => "SSD",
                        Some(5) => "SCM",
                        _ => "Unknown",
                    }
                    .to_string();
                }
            }
            Err(e) => tracing::warn!(error = %e, "MSFT_PhysicalDisk query failed"),
        },
        Err(e) => tracing::warn!(error = %e, "Storage namespace connect failed"),
    }

    let dg_com = init_com_lib();
    match WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\DeviceGuard", dg_com) {
        Ok(dg) => match dg.query::<Win32DeviceGuard>() {
            Ok(rows) => {
                if let Some(first) = rows.into_iter().next() {
                    snap.vbs_running = first.virtualization_based_security_status == Some(2);
                }
            }
            Err(e) => tracing::warn!(error = %e, "Win32_DeviceGuard query failed"),
        },
        Err(e) => tracing::warn!(error = %e, "DeviceGuard namespace connect failed"),
    }

    // EFI System Partition — Storage namespace'i tekrar kullan (yukarıdaki COM
    // init aynı thread'de). GptType "{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}"
    // sabit ESP GUID'i. Bulamazsa 0 kalır (Legacy BIOS, küçük olasılık).
    let esp_com = init_com_lib();
    if let Ok(storage) =
        WMIConnection::with_namespace_path("ROOT\\Microsoft\\Windows\\Storage", esp_com)
    {
        match storage.query::<MsftPartition>() {
            Ok(parts) => {
                const ESP_GUID: &str = "{c12a7328-f81f-11d2-ba4b-00a0c93ec93b}";
                for p in parts {
                    if p.gpt_type.as_deref().map(|g| g.eq_ignore_ascii_case(ESP_GUID))
                        == Some(true)
                    {
                        let bytes = p.size.unwrap_or(0);
                        snap.efi_size_mb = bytes / (1024 * 1024);
                        // FAT32 free space WMI'dan doğrudan gelmiyor — Get-Volume
                        // çağrısı PS spawn ister. ESP genelde %30-70 dolu olduğu
                        // için Total - 50MB rough estimate yerine 0 bırakıyoruz;
                        // ileride PS query eklenebilir.
                        snap.efi_free_mb = 0;
                        break;
                    }
                }
            }
            Err(e) => tracing::warn!(error = %e, "MSFT_Partition query failed"),
        }
    }

    Ok(snap)
}
