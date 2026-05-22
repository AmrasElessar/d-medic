use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #1 — RAM < 3 GB + VBS aktif.
/// Düşük RAM'li sistemde VBS (Virtualization Based Security) ciddi CPU tüketir.
pub struct RamVbsConflictCheck;

#[async_trait]
impl Check for RamVbsConflictCheck {
    fn id(&self) -> &'static str {
        "ram-vbs-conflict"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1:
        //   ram = wmi::read_snapshot().total_ram_gb
        //   vbs = HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\EnableVirtualizationBasedSecurity
        //   if ram < 3.0 && vbs == 1 => Finding(Critical, action_id="disable-vbs")
        Ok(Vec::new())
    }
}
