use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// Ultimate Performance power plan'ını aktifleştir/oluştur.
pub struct UltimatePerformance;

#[async_trait]
impl Action for UltimatePerformance {
    fn id(&self) -> &'static str { "ultimate-performance" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2:
        //   powercfg /duplicatescheme e9a42b02-d5df-448d-aa00-03f14749eb61
        //   powercfg /setactive <new-guid>
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "Ultimate Performance stub".into(),
            reboot_required: false,
            details: None,
        })
    }
}
