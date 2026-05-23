use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// Startup uygulamalarını temizle — kullanıcı seçimi kritik olduğu için
/// otomatik silme yerine Task Manager'ın Startup sekmesini açıp kullanıcıya
/// devrediyoruz. Manual seçim olmadan otomatik kaldırma yanlış uygulamayı
/// devre dışı bırakma riski taşır.
pub struct StartupCleanup;

#[async_trait]
impl Action for StartupCleanup {
    fn id(&self) -> &'static str {
        "startup-cleanup"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // /0 /startup Win11'de Task Manager → Startup apps sekmesini direkt açar.
        let out = ps::runner::run_script("Start-Process taskmgr.exe -ArgumentList '/0','/startup'")
            .await?;
        let success = out.status == 0;
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                "Task Manager → Startup sekmesi açıldı. Devre dışı bırakmak istediğiniz \
                 uygulamalara sağ tıklayıp \"Disable\" diyebilirsiniz."
                    .to_string()
            } else {
                format!("Task Manager açılamadı: {}", out.stderr.trim())
            },
            reboot_required: false,
            details: Some(json!({
                "interactive": true,
                "exit": out.status,
            })),
        })
    }
}
