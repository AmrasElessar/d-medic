use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// BCD store onarımı — bootrec /scanos + /rebuildbcd. Tam rebuildbcd interaktif
/// onay (Y/N) ister; PS `echo Y |` ile besleyerek non-interaktif yapıyoruz.
/// Normal Windows içinde bootrec /fixmbr çoğu zaman engellenir (system reserved
/// disk locked) — kullanıcı recovery ortamına gitmek zorunda kalabilir.
pub struct BcdRebuild;

#[async_trait]
impl Action for BcdRebuild {
    fn id(&self) -> &'static str {
        "bcd-rebuild"
    }
    fn reboot_required(&self) -> bool {
        true
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let script = "$scan = bootrec /scanos 2>&1\n\
            $rebuild = (echo Y | bootrec /rebuildbcd) 2>&1\n\
            [PSCustomObject]@{ scan = ($scan | Out-String); rebuild = ($rebuild | Out-String); exit = $LASTEXITCODE } | ConvertTo-Json -Compress";
        let out = ps::runner::run_script(script).await?;

        let parsed: Option<serde_json::Value> =
            serde_json::from_str(out.stdout.trim()).ok();
        let rebuild_text = parsed
            .as_ref()
            .and_then(|v| v.get("rebuild"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();
        let success = rebuild_text.contains("successfully")
            || rebuild_text.contains("başarıyla")
            || rebuild_text.contains("operation completed");

        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                "BCD store yeniden oluşturuldu. Yeniden başlatma sonrası test edin."
                    .to_string()
            } else {
                "bootrec /rebuildbcd başarısız — sistem partition kilitli olabilir. \
                 Recovery ortamından (WinRE) tekrar deneyin."
                    .to_string()
            },
            reboot_required: true,
            details: parsed.or_else(|| Some(json!({ "raw_stdout": out.stdout }))),
        })
    }
}
