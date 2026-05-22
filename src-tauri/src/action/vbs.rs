use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// VBS / HVCI'yı kapat (düşük RAM'li sistemlerde CPU verir).
pub struct DisableVbs;

#[async_trait]
impl Action for DisableVbs {
    fn id(&self) -> &'static str { "disable-vbs" }
    fn reboot_required(&self) -> bool { true }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 3:
        //   HKLM\SYSTEM\CurrentControlSet\Control\DeviceGuard\
        //     EnableVirtualizationBasedSecurity = 0
        //   HKLM\SYSTEM\...\DeviceGuard\Scenarios\HypervisorEnforcedCodeIntegrity\Enabled = 0
        //   bcdedit /set hypervisorlaunchtype off
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "VBS disabled (stub)".into(),
            reboot_required: true,
            details: None,
        })
    }
}
