use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// Görsel efektleri "Best Performance" moduna geçir.
pub struct MinimalVisualEffects;

#[async_trait]
impl Action for MinimalVisualEffects {
    fn id(&self) -> &'static str {
        "set-visual-effects-performance"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // VisualFXSetting = 2 → Best Performance (Windows tüm animasyonları kapatır).
        // HKCU yazımı admin gerektirmez.
        let script = "$key = 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\VisualEffects'\n\
            New-Item -Path $key -Force | Out-Null\n\
            Set-ItemProperty -Path $key -Name VisualFXSetting -Value 2 -Type DWord\n\
            # UserPreferencesMask: animasyon/gölge bitlerini de kapat\n\
            $um = 'HKCU:\\Control Panel\\Desktop'\n\
            Set-ItemProperty -Path $um -Name UserPreferencesMask -Value ([byte[]](0x90,0x12,0x03,0x80,0x10,0x00,0x00,0x00))\n\
            'ok'";
        let out = ps::runner::run_script(script).await?;
        let success = out.status == 0 && out.stdout.trim().ends_with("ok");
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                "Görsel efektler en iyi performans moduna geçirildi. Etkili olması için \
                 oturumu yeniden başlatın (logout/login)."
                    .to_string()
            } else {
                format!("Visual effects ayarlanamadı: {}", out.stderr.trim())
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
