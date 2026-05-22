use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #2 — SysMain (Superfetch) + HDD kombinasyonu = sürekli disk thrash.
pub struct SysmainHddCheck;

#[async_trait]
impl Check for SysmainHddCheck {
    fn id(&self) -> &'static str { "sysmain-hdd" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Get-PhysicalDisk MediaType == "HDD" && SysMain Running
        Ok(Vec::new())
    }
}

/// #6 — SMART kritik uyarı (Reallocated/Pending/Uncorrectable Sectors).
pub struct SmartHealthCheck;

#[async_trait]
impl Check for SmartHealthCheck {
    fn id(&self) -> &'static str { "smart-health" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Get-StorageReliabilityCounter — kritik eşik aşıldıysa
        // Finding(Critical, ActionType::Guided, guide_id="smart-disk-replace")
        Ok(Vec::new())
    }
}

/// #22 — Disk doluluk > %85.
pub struct DiskFullCheck;

#[async_trait]
impl Check for DiskFullCheck {
    fn id(&self) -> &'static str { "disk-full" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Win32_LogicalDisk.FreeSpace / Size
        Ok(Vec::new())
    }
}

/// #27 — HDD parçalanma > %10.
pub struct FragmentationCheck;

#[async_trait]
impl Check for FragmentationCheck {
    fn id(&self) -> &'static str { "fragmentation" }
    fn applicable_in(&self, kind: crate::models::ScanKind) -> bool {
        matches!(kind, crate::models::ScanKind::Deep)
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 2: defrag /A /C
        Ok(Vec::new())
    }
}
