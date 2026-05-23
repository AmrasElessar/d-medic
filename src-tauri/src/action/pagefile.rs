use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::diagnostic::wmi as wmi_helper;
use crate::error::DMedicResult;
use crate::ps;

/// Pagefile'ı RAM*1.5 sabit boyuta sabitle (otomatik yönetimden çıkar).
pub struct OptimizePagefile;

#[async_trait]
impl Action for OptimizePagefile {
    fn id(&self) -> &'static str {
        "pagefile-optimize"
    }
    fn reboot_required(&self) -> bool {
        true
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // Önerilen boyut için anlık RAM'i al.
        let snap = wmi_helper::read_snapshot().await?;
        let size_mb = ((snap.total_ram_gb * 1024.0) * 1.5).round() as u32;
        if size_mb == 0 {
            return Ok(ActionOutcome {
                action_id: self.id().to_string(),
                success: false,
                message: "RAM bilgisi okunamadı, pagefile boyutu hesaplanamadı.".into(),
                reboot_required: false,
                details: None,
            });
        }

        // wmic deprecated ama mevcut; bazı build'lerde Get-CimInstance + Set-CimInstance
        // tercih edilebilir. wmic daha basit ve garantili çalışır.
        let script = format!(
            "$cs = (wmic computersystem set AutomaticManagedPagefile=False) 2>&1\n\
             $exists = wmic pagefileset where \"name='C:\\\\\\\\pagefile.sys'\" get name 2>&1\n\
             if (-not ($exists -match 'pagefile.sys')) {{\n\
               $cr = (wmic pagefileset create name=\"C:\\\\\\\\pagefile.sys\") 2>&1\n\
             }} else {{ $cr = 'exists' }}\n\
             $sz = (wmic pagefileset where \"name='C:\\\\\\\\pagefile.sys'\" set InitialSize={size_mb},MaximumSize={size_mb}) 2>&1\n\
             [PSCustomObject]@{{ cs=$cs; cr=$cr; sz=$sz; size_mb={size_mb} }} | ConvertTo-Json -Compress"
        );
        let out = ps::runner::run_script(&script).await?;
        let success = out.status == 0
            && (out.stdout.contains("successful")
                || out.stdout.to_lowercase().contains("başarıyla"));

        let details: Option<serde_json::Value> =
            serde_json::from_str(out.stdout.trim()).ok();

        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                format!(
                    "Pagefile {size_mb} MB sabit boyuta ayarlandı. Yeniden başlatma sonrası etkili."
                )
            } else {
                "Pagefile ayarlanamadı (yönetici yetkisi gerekir).".to_string()
            },
            reboot_required: true,
            details: details.or_else(|| Some(json!({ "raw_stdout": out.stdout }))),
        })
    }
}
