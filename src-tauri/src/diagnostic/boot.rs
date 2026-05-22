use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #11 — BCD store hatası.
pub struct BcdHealthCheck;

#[async_trait]
impl Check for BcdHealthCheck {
    fn id(&self) -> &'static str { "bcd-health" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 2: bcdedit /enum all — kayıp identifier var mı?
        Ok(Vec::new())
    }
}

/// #12 — Legacy BIOS + MBR disk (Win11 UEFI gerektiriyor).
pub struct LegacyBiosCheck;

#[async_trait]
impl Check for LegacyBiosCheck {
    fn id(&self) -> &'static str { "legacy-bios" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1:
        //   PEFirmwareType (HKLM\SYSTEM\CurrentControlSet\Control) — 1=BIOS, 2=UEFI
        //   Get-Disk system disk.PartitionStyle — MBR vs GPT
        //   ikisi de legacy ise Finding(Guided, guide_id="mbr2gpt")
        Ok(Vec::new())
    }
}

/// #13 — EFI partition < 100 MB.
pub struct EfiPartitionCheck;

#[async_trait]
impl Check for EfiPartitionCheck {
    fn id(&self) -> &'static str { "efi-partition" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Get-Partition Type=System . Size < 100MB
        Ok(Vec::new())
    }
}
