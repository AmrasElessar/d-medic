use async_trait::async_trait;
use serde_json::json;

use super::{wmi, Check};
use crate::error::DMedicResult;
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel,
};

/// 3 GB altı: VBS düşük RAM'de ciddi performans kaybı yaratır (genelde 800-1500 MB
/// ek tahsis + Hyper-V katmanı). Eşik dokümandaki rakamla aynı.
const RAM_THRESHOLD_GB: f32 = 3.0;

/// #1 — RAM < 3 GB + VBS aktif.
pub struct RamVbsConflictCheck;

#[async_trait]
impl Check for RamVbsConflictCheck {
    fn id(&self) -> &'static str {
        "ram-vbs-conflict"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let snap = wmi::read_snapshot().await?;
        if snap.total_ram_gb >= RAM_THRESHOLD_GB || !snap.vbs_running {
            return Ok(Vec::new());
        }

        Ok(vec![Finding {
            id: "ram-vbs-conflict".to_string(),
            category: Category::Performance,
            priority: Priority::Critical,
            action_type: ActionType::Automatic,
            title: LocalizedText::new(
                "Düşük RAM'de VBS aktif",
                "VBS active on low-RAM system",
            ),
            description: LocalizedText::new(
                format!(
                    "Sistem {:.1} GB RAM'e sahip ve Sanallaştırma Tabanlı Güvenlik (VBS) çalışıyor. \
                     VBS ~800-1500 MB ek bellek tahsis eder ve düşük RAM'li makinelerde \
                     belirgin yavaşlığa yol açar. Devre dışı bırakılması önerilir.",
                    snap.total_ram_gb
                ),
                format!(
                    "System has {:.1} GB RAM with Virtualization-Based Security (VBS) running. \
                     VBS allocates ~800-1500 MB extra and noticeably slows low-RAM machines. \
                     Disabling is recommended.",
                    snap.total_ram_gb
                ),
            ),
            estimated_gain: EstimatedGain::RamMb { value: 1000 },
            risk: RiskLevel::Low,
            reboot_required: true,
            action_id: Some("disable-vbs".to_string()),
            guide_id: None,
            evidence: json!({
                "total_ram_gb": snap.total_ram_gb,
                "threshold_gb": RAM_THRESHOLD_GB,
                "vbs_running": snap.vbs_running,
            }),
        }])
    }
}
