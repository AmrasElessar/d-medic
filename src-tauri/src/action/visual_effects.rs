use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// Görsel efektleri minimum performans ayarına geçir.
pub struct MinimalVisualEffects;

#[async_trait]
impl Action for MinimalVisualEffects {
    fn id(&self) -> &'static str { "minimal-visual-effects" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2:
        //   HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects
        //   VisualFXSetting = 2 (Best Performance)
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "Visual effects minimum (stub)".into(),
            reboot_required: false,
            details: None,
        })
    }
}
