use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// Temp / Cache temizliği.
pub struct CleanTemp;

#[async_trait]
impl Action for CleanTemp {
    fn id(&self) -> &'static str { "clean-temp" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2:
        //   Remove-Item -Recurse -Force "$env:TEMP\*"
        //   Remove-Item -Recurse -Force "$env:SystemRoot\Temp\*"
        //   Remove-Item -Recurse -Force "$env:LOCALAPPDATA\Microsoft\Windows\INetCache\*"
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "Temp clean stub".into(),
            reboot_required: false,
            details: None,
        })
    }
}
