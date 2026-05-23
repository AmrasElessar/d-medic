use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::diagnostic::bloatware::{KNOWN_BLOATWARE, THIRD_PARTY_BLOAT_CONTAINS};
use crate::error::DMedicResult;
use crate::ps;

/// Bilinen UWP bloatware'i kaldır. KALICI işlem — Microsoft Store ile geri yüklenebilir
/// ama bazı paketler (Edge gibi) için ek adımlar gerekir. KNOWN_BLOATWARE prefix listesi
/// + THIRD_PARTY_BLOAT_CONTAINS contains pattern'ları diagnostic/bloatware.rs ile aynı.
pub struct RemoveBloatware;

#[async_trait]
impl Action for RemoveBloatware {
    fn id(&self) -> &'static str {
        // Check tarafı bu id ile arıyor.
        "uninstall-uwp-bloat"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // PS array literal'ı oluştur: 'Microsoft.BingNews','Microsoft.Xbox',...
        let prefixes = KNOWN_BLOATWARE
            .iter()
            .map(|p| format!("'{p}'"))
            .collect::<Vec<_>>()
            .join(",");
        let contains = THIRD_PARTY_BLOAT_CONTAINS
            .iter()
            .map(|p| format!("'{p}'"))
            .collect::<Vec<_>>()
            .join(",");

        let script = format!(
            "$prefixes = @({prefixes})\n\
             $contains = @({contains})\n\
             $removed = @(); $failed = @()\n\
             $pkgs = Get-AppxPackage -ErrorAction SilentlyContinue\n\
             foreach ($p in $pkgs) {{\n\
               $name = $p.Name\n\
               $hit = $false\n\
               foreach ($pref in $prefixes) {{ if ($name.StartsWith($pref)) {{ $hit = $true; break }} }}\n\
               if (-not $hit) {{ foreach ($c in $contains) {{ if ($name -like \"*$c*\") {{ $hit = $true; break }} }} }}\n\
               if ($hit) {{\n\
                 try {{ Remove-AppxPackage -Package $p.PackageFullName -ErrorAction Stop; $removed += $name }}\n\
                 catch {{ $failed += \"$name: $($_.Exception.Message.Substring(0, [Math]::Min(80, $_.Exception.Message.Length)))\" }}\n\
               }}\n\
             }}\n\
             [PSCustomObject]@{{ removed = $removed; failed = $failed }} | ConvertTo-Json -Compress -Depth 3"
        );
        let out = ps::runner::run_script(&script).await?;

        let parsed: Option<serde_json::Value> =
            serde_json::from_str(out.stdout.trim()).ok();
        let removed_count = parsed
            .as_ref()
            .and_then(|v| v.get("removed"))
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let failed_count = parsed
            .as_ref()
            .and_then(|v| v.get("failed"))
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        let success = out.status == 0 && removed_count > 0;
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if removed_count > 0 {
                format!(
                    "{} bloatware paketi kaldırıldı, {} başarısız (provisioned veya kilitli).",
                    removed_count, failed_count
                )
            } else if failed_count > 0 {
                format!("{} paket kaldırılamadı, hiçbiri silinmedi.", failed_count)
            } else {
                "Eşleşen bloatware paketi bulunamadı.".to_string()
            },
            reboot_required: false,
            details: parsed.or_else(|| Some(json!({ "raw_stdout": out.stdout }))),
        })
    }
}
