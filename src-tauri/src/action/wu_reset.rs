use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// Windows Update sıfırlama — wuauserv+bits+cryptSvc+msiserver durdur, SoftwareDistribution
/// + catroot2 rename, servisleri yeniden başlat.
pub struct WindowsUpdateReset;

#[async_trait]
impl Action for WindowsUpdateReset {
    fn id(&self) -> &'static str {
        // Check tarafı bu id ile arıyor.
        "reset-windows-update"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // Tek script — Wait-ServiceStopped pattern (Stop-Service -Force sonrası
        // 5 sn'ye kadar bekle, file lock için).
        let script = "$svcs = @('wuauserv','bits','cryptSvc','msiserver')\n\
            foreach ($s in $svcs) {\n\
              try { Stop-Service $s -Force -ErrorAction SilentlyContinue\n\
                $deadline = (Get-Date).AddSeconds(5)\n\
                while ((Get-Service $s).Status -ne 'Stopped' -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 200 }\n\
              } catch {}\n\
            }\n\
            $errors = @()\n\
            try {\n\
              if (Test-Path 'C:\\Windows\\SoftwareDistribution') {\n\
                $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'\n\
                Rename-Item 'C:\\Windows\\SoftwareDistribution' \"SoftwareDistribution.old.$stamp\" -ErrorAction Stop\n\
              }\n\
            } catch { $errors += \"sd: $($_.Exception.Message)\" }\n\
            try {\n\
              if (Test-Path 'C:\\Windows\\System32\\catroot2') {\n\
                $stamp = Get-Date -Format 'yyyyMMdd-HHmmss'\n\
                Rename-Item 'C:\\Windows\\System32\\catroot2' \"catroot2.old.$stamp\" -ErrorAction Stop\n\
              }\n\
            } catch { $errors += \"catroot: $($_.Exception.Message)\" }\n\
            foreach ($s in $svcs) { try { Start-Service $s -ErrorAction SilentlyContinue } catch {} }\n\
            if ($errors.Count -eq 0) { 'ok' } else { 'partial:' + ($errors -join ' | ') }";
        let out = ps::runner::run_script(script).await?;
        let success = out.status == 0 && out.stdout.trim().starts_with("ok");
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                "Windows Update bileşenleri sıfırlandı; SoftwareDistribution + catroot2 yedeklendi."
                    .to_string()
            } else {
                format!("Reset kısmen başarılı: {}", out.stdout.trim())
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
