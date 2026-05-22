use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// Hibernation'ı kapat — hiberfil.sys (RAM kadar disk) silinir.
pub struct DisableHibernation;

#[async_trait]
impl Action for DisableHibernation {
    fn id(&self) -> &'static str { "disable-hibernation" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2: powercfg /hibernate off
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "Hibernation off (stub)".into(),
            reboot_required: false,
            details: None,
        })
    }
}
