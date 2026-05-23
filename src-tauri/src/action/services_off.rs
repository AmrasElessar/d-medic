use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

/// `Stop-Service` + `Set-Service -StartupType Disabled` ortak adımı.
/// DiagTrack/dmwappushservice gibi telemetry servisleri için liste de alabilir.
async fn disable_services(action_id: &str, services: &[&str]) -> DMedicResult<ActionOutcome> {
    let list = services
        .iter()
        .map(|s| format!("'{s}'"))
        .collect::<Vec<_>>()
        .join(",");
    let script = format!(
        "$svcs = @({list}); $results = @{{}}\n\
         foreach ($s in $svcs) {{\n\
           try {{ Stop-Service $s -Force -ErrorAction Stop; $stop='ok' }} catch {{ $stop=$_.Exception.Message }}\n\
           try {{ Set-Service $s -StartupType Disabled -ErrorAction Stop; $set='ok' }} catch {{ $set=$_.Exception.Message }}\n\
           $results[$s] = @{{ stop=$stop; set=$set }}\n\
         }}\n\
         $results | ConvertTo-Json -Compress"
    );
    let out = ps::runner::run_script(&script).await?;
    let success = out.status == 0 && !out.stderr.to_lowercase().contains("access is denied")
        && !out.stderr.to_lowercase().contains("erişim engellendi");

    let details = serde_json::from_str::<serde_json::Value>(out.stdout.trim()).ok();
    let message = if success {
        format!("{} servis(ler)i durduruldu ve Disabled olarak ayarlandı.", services.len())
    } else {
        format!(
            "Bazı işlemler başarısız oldu — yönetici yetkisi gerek olabilir. stderr: {}",
            out.stderr.trim().chars().take(200).collect::<String>()
        )
    };
    Ok(ActionOutcome {
        action_id: action_id.to_string(),
        success,
        message,
        reboot_required: false,
        details: details.or_else(|| Some(json!({ "raw_stdout": out.stdout }))),
    })
}

pub struct DisableSysmain;
#[async_trait]
impl Action for DisableSysmain {
    fn id(&self) -> &'static str {
        "disable-sysmain"
    }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        disable_services(self.id(), &["SysMain"]).await
    }
}

pub struct DisableSearchIndex;
#[async_trait]
impl Action for DisableSearchIndex {
    fn id(&self) -> &'static str {
        "disable-wsearch"
    }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        disable_services(self.id(), &["WSearch"]).await
    }
}

pub struct DisableTelemetry;
#[async_trait]
impl Action for DisableTelemetry {
    fn id(&self) -> &'static str {
        "disable-telemetry"
    }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        disable_services(self.id(), &["DiagTrack", "dmwappushservice"]).await
    }
}
