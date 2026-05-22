use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// SFC /scannow — bozuk sistem dosyalarını WinSxS önbelleğinden onarır.
pub struct SfcRepair;

#[async_trait]
impl Action for SfcRepair {
    fn id(&self) -> &'static str { "sfc-repair" }
    fn reboot_required(&self) -> bool { true }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2: crate::ps::runner::run("sfc /scannow") + CBS.log parse
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "SFC stub".into(),
            reboot_required: true,
            details: None,
        })
    }
}
