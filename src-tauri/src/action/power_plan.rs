use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// "Yüksek Performans" (veya yoksa duplicate Ultimate) plan'ını aktif et.
pub struct UltimatePerformance;

#[async_trait]
impl Action for UltimatePerformance {
    fn id(&self) -> &'static str {
        // Check tarafı (power.rs) bu id ile arıyor.
        "set-high-performance-plan"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // Strateji:
        //   1) High Performance GUID'i varsa direkt aktive et (8c5e7fda-...).
        //   2) Yoksa Ultimate Performance'ı duplicate et ve aktive et (e9a42b02-...).
        let script = "$hp = '8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c'\n\
            $ult = 'e9a42b02-d5df-448d-aa00-03f14749eb61'\n\
            $existing = (powercfg /list) -join \"`n\"\n\
            if ($existing -match $hp) {\n\
              powercfg /setactive $hp; 'used:high-performance'\n\
            } else {\n\
              $dup = powercfg /duplicatescheme $ult 2>&1\n\
              $newGuid = ($dup | Select-String -Pattern '([0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})').Matches[0].Groups[1].Value\n\
              if ($newGuid) { powercfg /setactive $newGuid; \"used:ultimate-duplicate:$newGuid\" }\n\
              else { 'failed:no-guid' }\n\
            }";
        let out = ps::runner::run_script(script).await?;
        let success = out.status == 0 && out.stdout.trim().starts_with("used:");
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                format!("Aktif power plan değiştirildi: {}", out.stdout.trim())
            } else {
                "Power plan değiştirilemedi (yönetici yetkisi gerek olabilir).".to_string()
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
