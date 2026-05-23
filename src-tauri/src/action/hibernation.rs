use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// Hibernation'ı kapat — hiberfil.sys silinir, RAM kadar disk boşluğu açılır.
pub struct DisableHibernation;

#[async_trait]
impl Action for DisableHibernation {
    fn id(&self) -> &'static str {
        "disable-hibernation"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let out = ps::runner::run_script("powercfg /hibernate off 2>&1").await?;
        let stderr_lc = out.stderr.to_lowercase();
        let needs_admin = stderr_lc.contains("access is denied")
            || stderr_lc.contains("erişim engellendi")
            || out.stdout.to_lowercase().contains("yönetici");
        let success = out.status == 0 && !needs_admin;

        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                "Hibernation kapatıldı, hiberfil.sys silindi.".to_string()
            } else {
                "powercfg yönetici yetkisi gerektirir veya başka bir hata oldu.".to_string()
            },
            reboot_required: false,
            details: Some(json!({
                "stdout": out.stdout.trim(),
                "stderr": out.stderr.trim(),
                "exit": out.status,
            })),
        })
    }
}
