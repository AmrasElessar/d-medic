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
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32PhysicalMemory {
    capacity: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32OperatingSystem {
    free_physical_memory: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32Processor {
    name: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32LogicalDisk {
    device_id: Option<String>,
    drive_type: Option<u32>,
    size: Option<u64>,
    free_space: Option<u64>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MsftPhysicalDisk {
    media_type: Option<u16>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32DeviceGuard {
    /// 0: off, 1: configured but not running, 2: running.
    virtualization_based_security_status: Option<u32>,
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

/// Sistem genel snapshot'ını WMI üzerinden al.
pub async fn read_snapshot() -> DMedicResult<SystemSnapshot> {
    tokio::task::spawn_blocking(read_snapshot_blocking)
        .await
        .map_err(|e| DMedicError::Other(format!("wmi spawn_blocking join: {e}")))?
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

    Ok(snap)
}
