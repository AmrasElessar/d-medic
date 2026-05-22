use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// Bilinen UWP bloatware'i kaldır. KALICIDIR — kullanıcı uyarılmalı.
pub struct RemoveBloatware;

#[async_trait]
impl Action for RemoveBloatware {
    fn id(&self) -> &'static str { "remove-bloatware" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 3:
        //   for pkg in KNOWN_BLOATWARE:
        //     Get-AppxPackage $pkg | Remove-AppxPackage -AllUsers
        //     Get-AppxProvisionedPackage -Online | ? DisplayName -eq $pkg | Remove-AppxProvisionedPackage -Online
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "Debloat stub".into(),
            reboot_required: false,
            details: None,
        })
    }
}
