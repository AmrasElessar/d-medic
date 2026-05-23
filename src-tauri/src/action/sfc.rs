use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// SFC /scannow — bozuk sistem dosyalarını WinSxS önbelleğinden onarır.
/// 5-15 dk sürer; PS spawn'ı blocking değildir (tokio process).
pub struct SfcRepair;

#[async_trait]
impl Action for SfcRepair {
    fn id(&self) -> &'static str {
        "sfc-repair"
    }
    fn reboot_required(&self) -> bool {
        true
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let out = ps::runner::run_script("sfc /scannow").await?;
        let stdout_lc = out.stdout.to_lowercase();

        let no_corrupt = stdout_lc.contains("did not find any integrity")
            || stdout_lc.contains("bütünlük ihlali bulamadı");
        let fixed = stdout_lc.contains("successfully repaired")
            || stdout_lc.contains("başarıyla onardı");
        let cant_fix = stdout_lc.contains("was unable to fix")
            || stdout_lc.contains("onaramadı");

        let (success, message) = if no_corrupt {
            (true, "SFC: bütünlük ihlali bulamadı, sistem dosyaları sağlıklı.")
        } else if fixed {
            (
                true,
                "SFC: bozuk dosyalar başarıyla onarıldı. Yeniden başlatma önerilir.",
            )
        } else if cant_fix {
            (
                false,
                "SFC bazı dosyaları onaramadı — DISM /RestoreHealth çalıştırıp tekrar deneyin.",
            )
        } else {
            (
                false,
                "SFC çalışmadı (yönetici yetkisi gerekir veya başka bir hata oldu).",
            )
        };

        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: message.to_string(),
            reboot_required: fixed,
            details: Some(json!({
                "stdout_sample": stdout_lc.lines().take(20).collect::<Vec<_>>().join(" | "),
                "exit": out.status,
            })),
        })
    }
}
