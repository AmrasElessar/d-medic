use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// Windows Recovery Environment'ı aktifleştir (reagentc /enable).
pub struct EnableWindowsRe;

#[async_trait]
impl Action for EnableWindowsRe {
    fn id(&self) -> &'static str {
        "enable-windows-re"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let out = ps::runner::run_script("reagentc /enable 2>&1").await?;
        let stdout_lc = out.stdout.to_lowercase();
        let success = out.status == 0
            && (stdout_lc.contains("operation successful")
                || stdout_lc.contains("işlem başarılı"));
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                "Windows RE etkinleştirildi.".to_string()
            } else {
                format!(
                    "reagentc başarısız (yönetici yetkisi veya WinRE.wim dosyası gerekir): {}",
                    out.stdout.trim().chars().take(200).collect::<String>()
                )
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
