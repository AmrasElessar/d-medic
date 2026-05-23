use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// DISM /Online /Cleanup-Image /RestoreHealth — Windows image onarımı.
/// 10-30 dk sürebilir; internet bağlantısı (Windows Update kaynağı) ister.
pub struct DismRestoreHealth;

#[async_trait]
impl Action for DismRestoreHealth {
    fn id(&self) -> &'static str {
        "dism-restore-health"
    }
    fn reboot_required(&self) -> bool {
        true
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let out = ps::runner::run_script("DISM /Online /Cleanup-Image /RestoreHealth").await?;
        let stdout_lc = out.stdout.to_lowercase();
        let success = out.status == 0
            && (stdout_lc.contains("the restore operation completed successfully")
                || stdout_lc.contains("geri yükleme işlemi başarıyla tamamlandı")
                || stdout_lc.contains("operation completed successfully"));

        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                "DISM image onarımı başarılı. Şimdi `sfc /scannow` çalıştırılabilir."
                    .to_string()
            } else {
                "DISM onarımı başarısız — internet bağlantısı veya WSUS kaynak sorunu olabilir."
                    .to_string()
            },
            reboot_required: success,
            details: Some(json!({
                "stdout_sample": stdout_lc.lines().take(30).collect::<Vec<_>>().join(" | "),
                "exit": out.status,
            })),
        })
    }
}
