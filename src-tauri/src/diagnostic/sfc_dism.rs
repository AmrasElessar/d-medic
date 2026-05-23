//! Deep scan kapsamındaki yavaş kontroller (5-30 dk).

use async_trait::async_trait;
use serde_json::json;

use super::Check;
use crate::error::DMedicResult;
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel, ScanKind,
};
use crate::ps;

/// #3 — SFC: bozuk sistem dosyaları (5-15 dk).
pub struct SfcCheck;

#[async_trait]
impl Check for SfcCheck {
    fn id(&self) -> &'static str {
        "sfc-corrupt"
    }
    fn applicable_in(&self, kind: ScanKind) -> bool {
        matches!(kind, ScanKind::Deep)
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // sfc /verifyonly: yalnızca tarama, onarım yapmaz. Çıktı stdout'a yazar.
        // Bilinen sonuç ifadeleri (TR/EN):
        //   "did not find any integrity violations" / "bütünlük ihlali bulamadı"
        //   "found corrupt files and successfully repaired"
        //   "found corrupt files but was unable to repair"
        //   "found integrity violations" / "bütünlük ihlalleri buldu"
        let out = ps::runner::run_script("sfc /verifyonly").await.ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };
        let stdout = out.stdout.to_lowercase();

        let clean = stdout.contains("did not find any integrity")
            || stdout.contains("bütünlük ihlali bulamadı")
            || stdout.contains("herhangi bir bütünlük ihlali");
        let dirty = stdout.contains("found integrity violations")
            || stdout.contains("bütünlük ihlal");
        if clean && !dirty {
            return Ok(Vec::new());
        }
        if !dirty {
            // sonuç belirsiz — sessiz kal, false positive vermeyelim.
            return Ok(Vec::new());
        }

        Ok(vec![Finding {
            id: "sfc-corrupt".to_string(),
            category: Category::Stability,
            priority: Priority::High,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                "SFC: sistem dosyalarında bütünlük ihlali",
                "SFC: system file integrity violations",
            ),
            description: LocalizedText::new(
                "sfc /verifyonly bozuk sistem dosyası tespit etti. Onarım için elevated \
                 terminal'de `sfc /scannow` çalıştırın; sonuç \"bazılarını onaramadı\" derse \
                 DISM ile image onarımı sonra tekrar SFC."
                    .to_string(),
                "sfc /verifyonly reported corrupted system files. Repair: run `sfc /scannow` \
                 elevated; if result is \"could not fix some\", run DISM image repair then \
                 SFC again."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::Low,
            reboot_required: false,
            action_id: None,
            guide_id: None,
            evidence: json!({
                "sfc_output_sample": stdout.lines().take(20).collect::<Vec<_>>().join(" | "),
            }),
        }])
    }
}

/// #4 — DISM image health (3-10 dk).
pub struct DismCheck;

#[async_trait]
impl Check for DismCheck {
    fn id(&self) -> &'static str {
        "dism-image"
    }
    fn applicable_in(&self, kind: ScanKind) -> bool {
        matches!(kind, ScanKind::Deep)
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // /ScanHealth onarım yapmaz, sadece WindowsImage cabinet'ında bozulma var mı bakar.
        let out = ps::runner::run_script("DISM /Online /Cleanup-Image /ScanHealth")
            .await
            .ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };
        let stdout = out.stdout.to_lowercase();

        // Bilinen ifadeler:
        //   "no component store corruption" / "bileşen deposunda bozulma yok"
        //   "the component store is repairable" / "onarılabilir"
        //   "the component store cannot be repaired" / "onarılamaz"
        let healthy = stdout.contains("no component store corruption")
            || stdout.contains("bozulma algılanmadı");
        let repairable = stdout.contains("component store is repairable")
            || stdout.contains("onarılabilir");
        let not_repairable = stdout.contains("cannot be repaired")
            || stdout.contains("onarılamaz");
        if healthy {
            return Ok(Vec::new());
        }
        if !repairable && !not_repairable {
            return Ok(Vec::new());
        }

        let (priority, body_tr, body_en, action) = if not_repairable {
            (
                Priority::Critical,
                "DISM bileşen deposunun onarılamaz olduğunu raporladı. Tek çıkış: yerinde \
                 yeniden yükleme (in-place upgrade) veya temiz kurulum.",
                "DISM reports component store cannot be repaired. Options: in-place upgrade \
                 or clean install.",
                None,
            )
        } else {
            (
                Priority::High,
                "DISM bileşen deposunda onarılabilir bozulma tespit etti. Çalıştırma: \
                 `DISM /Online /Cleanup-Image /RestoreHealth` ardından `sfc /scannow`.",
                "DISM detected repairable corruption. Run: `DISM /Online /Cleanup-Image \
                 /RestoreHealth` then `sfc /scannow`.",
                Some("dism-restore-health".to_string()),
            )
        };

        Ok(vec![Finding {
            id: "dism-image".to_string(),
            category: Category::Stability,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                "DISM: Windows image bozulması",
                "DISM: Windows image corruption",
            ),
            description: LocalizedText::new(body_tr, body_en),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::Low,
            reboot_required: false,
            action_id: action,
            guide_id: None,
            evidence: json!({
                "dism_output_sample": stdout.lines().take(20).collect::<Vec<_>>().join(" | "),
            }),
        }])
    }
}
