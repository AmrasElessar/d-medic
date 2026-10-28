use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// Temp / cache temizliği. Hatalar (kilitli dosya) sessizce atlanır.
///
/// Yalnız 24 saatten eski dosyalar silinir — çalışan kurulumların / uygulamaların
/// o an kullandığı geçici dosyalara dokunulmaz. Junction / symlink'lerin içine
/// inilmez (TEMP altındaki bir junction ağaç dışını işaret edebilir).
pub struct CleanTemp;

#[async_trait]
impl Action for CleanTemp {
    fn id(&self) -> &'static str {
        "clean-temp"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let script = "$cutoff = (Get-Date).AddHours(-24)\n\
            $script:freed = [int64]0\n\
            function Clear-Old($dir) {\n\
              Get-ChildItem -LiteralPath $dir -Force -ErrorAction SilentlyContinue | ForEach-Object {\n\
                if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { return }\n\
                if ($_.PSIsContainer) {\n\
                  Clear-Old $_.FullName\n\
                  if ($_.CreationTime -lt $cutoff -and -not (Get-ChildItem -LiteralPath $_.FullName -Force -ErrorAction SilentlyContinue)) {\n\
                    Remove-Item -LiteralPath $_.FullName -Force -ErrorAction SilentlyContinue\n\
                  }\n\
                } elseif ($_.LastWriteTime -lt $cutoff) {\n\
                  $len = $_.Length\n\
                  Remove-Item -LiteralPath $_.FullName -Force -ErrorAction SilentlyContinue\n\
                  if (-not (Test-Path -LiteralPath $_.FullName)) { $script:freed += $len }\n\
                }\n\
              }\n\
            }\n\
            $roots = @($env:TEMP, \"$env:SystemRoot\\Temp\", \"$env:LOCALAPPDATA\\Microsoft\\Windows\\INetCache\")\n\
            foreach ($r in $roots) { if ($r -and (Test-Path -LiteralPath $r)) { Clear-Old $r } }\n\
            [PSCustomObject]@{ freed_mb = [int]($script:freed / 1MB) } | ConvertTo-Json -Compress";
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
                format!(
                    "Geçici dosyalar temizlendi (24 saatten eski): ~{} MB serbest bırakıldı.",
                    freed_mb
                )
            } else {
                format!("Temizlik başarısız: {}", out.stderr.trim())
            },
            reboot_required: false,
            details: details.or_else(|| Some(json!({ "raw_stdout": out.stdout }))),
        })
    }
}
