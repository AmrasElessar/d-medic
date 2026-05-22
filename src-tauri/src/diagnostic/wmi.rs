//! WMI native helper'ları — windows-rs ile doğrudan COM çağrısı.
//!
//! Faz 1 hedefi: PS spawn etmeden RAM/CPU/Disk bilgisi okumak.
//! Şimdilik iskelet — gerçek implementasyon `windows::Win32::System::Wmi` ile.

use crate::error::DMedicResult;

#[derive(Debug, Default)]
pub struct SystemSnapshot {
    pub total_ram_gb: f32,
    pub free_ram_gb: f32,
    pub cpu_name: String,
    pub primary_disk_type: String,
    pub primary_disk_size_gb: f32,
    pub primary_disk_free_gb: f32,
}

/// Sistem genel snapshot'ını WMI üzerinden al.
pub async fn read_snapshot() -> DMedicResult<SystemSnapshot> {
    // TODO Faz 1:
    //   - CoInitializeEx + IWbemLocator::ConnectServer
    //   - Win32_PhysicalMemory.Capacity
    //   - Win32_OperatingSystem.FreePhysicalMemory
    //   - Win32_Processor.Name
    //   - MSFT_PhysicalDisk.MediaType
    //   - Win32_LogicalDisk.Size / FreeSpace
    Ok(SystemSnapshot::default())
}
