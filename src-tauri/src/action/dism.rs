use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// DISM /Online /Cleanup-Image /RestoreHealth — Windows image onarımı.
pub struct DismRestoreHealth;

#[async_trait]
impl Action for DismRestoreHealth {
    fn id(&self) -> &'static str { "dism-restore-health" }
    fn reboot_required(&self) -> bool { true }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2: ps::runner — uzun sürer (10-30 dk), progress event yayınla
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "DISM stub".into(),
            reboot_required: true,
            details: None,
        })
    }
}
