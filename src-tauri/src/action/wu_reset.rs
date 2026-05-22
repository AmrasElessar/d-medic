use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// Windows Update sıfırlama — SoftwareDistribution + catroot2 rename.
/// File lock için Wait-ServiceStopped pattern'i ZORUNLU.
pub struct WindowsUpdateReset;

#[async_trait]
impl Action for WindowsUpdateReset {
    fn id(&self) -> &'static str { "wu-reset" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2:
        //   ps::wait::stop_and_wait(["wuauserv","bits","cryptSvc","msiserver"])
        //   Rename-Item SoftwareDistribution -> SoftwareDistribution.old
        //   Rename-Item catroot2 -> catroot2.old
        //   Start-Service [...]
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "WU reset stub".into(),
            reboot_required: false,
            details: None,
        })
    }
}
