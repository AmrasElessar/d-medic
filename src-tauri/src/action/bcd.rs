use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// BCD store onarımı — bootrec /fixmbr + /fixboot + /rebuildbcd.
/// Normal Windows içinden çağrılır; tam onarım WinPE gerektirir.
pub struct BcdRebuild;

#[async_trait]
impl Action for BcdRebuild {
    fn id(&self) -> &'static str { "bcd-rebuild" }
    fn reboot_required(&self) -> bool { true }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2: bootrec sırayla; çıktı parse + UI'a aktar
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "BCD stub".into(),
            reboot_required: true,
            details: None,
        })
    }
}
