use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// HDD defrag. SSD'de TRIM, asla full defrag DEĞİL.
pub struct DefragHdd;

#[async_trait]
impl Action for DefragHdd {
    fn id(&self) -> &'static str { "defrag-hdd" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 3:
        //   Optimize-Volume -DriveLetter C  (otomatik HDD->defrag, SSD->retrim)
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "Defrag stub".into(),
            reboot_required: false,
            details: None,
        })
    }
}
