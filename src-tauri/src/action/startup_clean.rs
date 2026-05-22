use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// Startup uygulamalarını devre dışı bırak.
pub struct StartupCleanup;

#[async_trait]
impl Action for StartupCleanup {
    fn id(&self) -> &'static str { "startup-cleanup" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2:
        //   HKCU\Software\Microsoft\Windows\CurrentVersion\Run değerleri
        //   Win32_StartupCommand kayıtları
        //   Task Scheduler startup task'leri
        //   — kullanıcı seçimini al, seçilenleri kaldır
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "Startup cleanup stub".into(),
            reboot_required: false,
            details: None,
        })
    }
}
