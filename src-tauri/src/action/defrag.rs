use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::diagnostic::wmi as wmi_helper;
use crate::error::DMedicResult;
use crate::ps;

/// HDD'de defrag, SSD'de TRIM. Optimize-Volume zaten disk türünü doğru tespit eder
/// ve uygun işlemi seçer. Yine de bizim tarafta SSD'de "asla full defrag DEĞİL"
/// invariant'ı korumak için snapshot.primary_disk_type kontrolü yapıyoruz.
pub struct DefragHdd;

#[async_trait]
impl Action for DefragHdd {
    fn id(&self) -> &'static str {
        // Check tarafı bu id ile arıyor.
        "defrag-system"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let snap = wmi_helper::read_snapshot().await?;
        // SSD'de Optimize-Volume -ReTrim, HDD'de -Defrag yeterli.
        let cmd = if snap.primary_disk_type == "HDD" {
            "Optimize-Volume -DriveLetter C -Defrag -Verbose"
        } else if snap.primary_disk_type == "SSD" {
            "Optimize-Volume -DriveLetter C -ReTrim -Verbose"
        } else {
            // Bilinmeyen disk türünde Optimize-Volume kendisi doğruyu seçer.
            "Optimize-Volume -DriveLetter C -Verbose"
        };
        let out = ps::runner::run_script(cmd).await?;
        let success = out.status == 0;
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                format!(
                    "{} üzerinde Optimize-Volume tamamlandı.",
                    snap.primary_disk_type
                )
            } else {
                format!("Optimize-Volume başarısız: {}", out.stderr.trim())
            },
            reboot_required: false,
            details: Some(json!({
                "disk_type": snap.primary_disk_type,
                "command": cmd,
                "exit": out.status,
                "stdout_sample": out.stdout.lines().take(10).collect::<Vec<_>>().join(" | "),
            })),
        })
    }
}
