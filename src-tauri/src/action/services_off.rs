use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

macro_rules! service_action {
    ($name:ident, $id:literal, $svc:literal, $reason:literal) => {
        pub struct $name;
        #[async_trait]
        impl Action for $name {
            fn id(&self) -> &'static str { $id }
            async fn apply(&self) -> DMedicResult<ActionOutcome> {
                // TODO Faz 2:
                //   snapshot::service_state::capture($svc)
                //   Set-Service $svc -StartupType Disabled
                //   Stop-Service $svc -Force
                let _reason = $reason;
                Ok(ActionOutcome {
                    action_id: self.id().to_string(),
                    success: true,
                    message: format!("{} devre dışı bırakıldı (stub)", $svc),
                    reboot_required: false,
                    details: None,
                })
            }
        }
    };
}

service_action!(DisableSysmain,     "disable-sysmain",     "SysMain",   "HDD thrash");
service_action!(DisableSearchIndex, "limit-search-index",  "WSearch",   "RAM tasarrufu");
service_action!(DisableTelemetry,   "disable-telemetry",   "DiagTrack", "Telemetry");
