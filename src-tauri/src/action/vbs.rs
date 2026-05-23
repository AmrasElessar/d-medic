use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// VBS + HVCI'yı kapat — düşük RAM'de ~800-1500 MB ve CPU geri kazandırır.
/// 3 noktada değişiklik: HKLM\...\DeviceGuard root, HVCI scenario, BCD hypervisorlaunchtype.
pub struct DisableVbs;

#[async_trait]
impl Action for DisableVbs {
    fn id(&self) -> &'static str {
        "disable-vbs"
    }
    fn reboot_required(&self) -> bool {
        true
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let script = "$dg = 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\DeviceGuard'\n\
            $hvci = 'HKLM:\\SYSTEM\\CurrentControlSet\\Control\\DeviceGuard\\Scenarios\\HypervisorEnforcedCodeIntegrity'\n\
            $errors = @()\n\
            try { New-Item -Path $dg -Force -ErrorAction Stop | Out-Null; Set-ItemProperty -Path $dg -Name EnableVirtualizationBasedSecurity -Value 0 -Type DWord -ErrorAction Stop } catch { $errors += \"dg: $($_.Exception.Message)\" }\n\
            try { New-Item -Path $hvci -Force -ErrorAction Stop | Out-Null; Set-ItemProperty -Path $hvci -Name Enabled -Value 0 -Type DWord -ErrorAction Stop } catch { $errors += \"hvci: $($_.Exception.Message)\" }\n\
            $bcd = bcdedit /set hypervisorlaunchtype off 2>&1\n\
            if ($LASTEXITCODE -ne 0) { $errors += \"bcd: $bcd\" }\n\
            if ($errors.Count -eq 0) { 'ok' } else { 'fail:' + ($errors -join ' | ') }";
        let out = ps::runner::run_script(script).await?;
        let success = out.status == 0 && out.stdout.trim().ends_with("ok");
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                "VBS + HVCI devre dışı bırakıldı. Yeniden başlatma sonrası etkin olur.".to_string()
            } else {
                format!(
                    "VBS kapatma başarısız (yönetici yetkisi gerekir): {}",
                    out.stdout.trim()
                )
            },
            reboot_required: true,
            details: Some(json!({
                "stdout": out.stdout.trim(),
                "stderr": out.stderr.trim(),
                "exit": out.status,
            })),
        })
    }
}
