use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// Temp / cache temizliği. Hatalar (kilitli dosya) sessizce atlanır.
pub struct CleanTemp;

#[async_trait]
impl Action for CleanTemp {
    fn id(&self) -> &'static str {
        "clean-temp"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let script = "$paths = @(\"$env:TEMP\\*\", \"$env:SystemRoot\\Temp\\*\", \"$env:LOCALAPPDATA\\Microsoft\\Windows\\INetCache\\*\")\n\
            $before = 0; $after = 0\n\
            foreach ($p in $paths) {\n\
              try { $before += (Get-ChildItem $p -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum } catch {}\n\
            }\n\
            foreach ($p in $paths) {\n\
              try { Remove-Item $p -Recurse -Force -ErrorAction SilentlyContinue } catch {}\n\
            }\n\
            foreach ($p in $paths) {\n\
              try { $after += (Get-ChildItem $p -Recurse -Force -ErrorAction SilentlyContinue | Measure-Object -Property Length -Sum).Sum } catch {}\n\
            }\n\
            [PSCustomObject]@{ freed_mb = [int](($before - $after) / 1MB); before_mb = [int]($before/1MB); after_mb = [int]($after/1MB) } | ConvertTo-Json -Compress";
        let out = ps::runner::run_script(script).await?;
        let success = out.status == 0;

        let details = serde_json::from_str::<serde_json::Value>(out.stdout.trim()).ok();
        let freed_mb = details
            .as_ref()
            .and_then(|v| v.get("freed_mb"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                format!("Geçici dosyalar temizlendi: ~{} MB serbest bırakıldı.", freed_mb)
            } else {
                format!("Temizlik başarısız: {}", out.stderr.trim())
            },
            reboot_required: false,
            details: details.or_else(|| Some(json!({ "raw_stdout": out.stdout }))),
        })
    }
}
