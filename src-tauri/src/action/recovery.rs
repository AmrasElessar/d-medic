use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// Windows Recovery Environment'ı aktifleştir.
pub struct EnableWindowsRe;

#[async_trait]
impl Action for EnableWindowsRe {
    fn id(&self) -> &'static str { "enable-windows-re" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2: reagentc /enable
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "Windows RE enabled (stub)".into(),
            reboot_required: false,
            details: None,
        })
    }
}
