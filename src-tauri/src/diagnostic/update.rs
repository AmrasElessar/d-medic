use async_trait::async_trait;
use serde_json::json;

use super::{registry, Check};
use crate::error::{DMedicError, DMedicResult};
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel,
};

/// #14 — Windows Update takılı (SoftwareDistribution\Download büyük + wuauserv yok).
pub struct WindowsUpdateStuckCheck;

#[async_trait]
impl Check for WindowsUpdateStuckCheck {
    fn id(&self) -> &'static str {
        "wu-stuck"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let script = "$r = @{}\n\
            try { $s = Get-Service wuauserv -ErrorAction Stop; $r.state = [string]$s.Status } catch { $r.state = 'unknown' }\n\
            try { $sd = Get-ChildItem 'C:\\Windows\\SoftwareDistribution\\Download' -Recurse -ErrorAction Stop -Force | Measure-Object -Property Length -Sum; $r.size_mb = [int]($sd.Sum / 1MB) } catch { $r.size_mb = 0 }\n\
            $r | ConvertTo-Json -Compress";
        let out = crate::ps::runner::run_script(script).await.ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };

        #[derive(serde::Deserialize)]
        struct WuStuckData {
            state: String,
            size_mb: u32,
        }
        let trimmed = out.stdout.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }
        let mut buf = trimmed.to_owned().into_bytes();
        let parsed: WuStuckData = match simd_json::serde::from_slice(&mut buf) {
            Ok(p) => p,
            Err(_) => return Ok(Vec::new()),
        };

        // Heuristik: download klasörü > 3 GB takılı kalmış indirme anlamına gelir.
        // wuauserv "Stopped" + büyük cache → klasik "takılı" tablosu.
        let big_cache = parsed.size_mb >= 3000;
        let svc_stopped = parsed.state == "Stopped";
        if !big_cache && !svc_stopped {
            return Ok(Vec::new());
        }

        let priority = if big_cache && svc_stopped {
            Priority::High
        } else {
            Priority::Medium
        };

        Ok(vec![Finding {
            id: "wu-stuck".to_string(),
            category: Category::Stability,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                "Windows Update takılı görünüyor",
                "Windows Update appears stuck",
            ),
            description: LocalizedText::new(
                format!(
                    "wuauserv durumu: {}, SoftwareDistribution\\Download boyutu: {} MB. \
                     Reset için: servisi durdur → Download klasörünü temizle → servisi başlat.",
                    parsed.state, parsed.size_mb
                ),
                format!(
                    "wuauserv state: {}, SoftwareDistribution\\Download size: {} MB. \
                     Reset by: stop service → clear Download folder → start service.",
                    parsed.state, parsed.size_mb
                ),
            ),
            estimated_gain: EstimatedGain::DiskMb { value: parsed.size_mb },
            risk: RiskLevel::Low,
            reboot_required: false,
            action_id: Some("reset-windows-update".to_string()),
            guide_id: None,
            evidence: json!({
                "wuauserv_state": parsed.state,
                "softdist_download_mb": parsed.size_mb,
            }),
        }])
    }
}

/// #15 — Pending reboot bekliyor.
pub struct PendingRebootCheck;

#[async_trait]
impl Check for PendingRebootCheck {
    fn id(&self) -> &'static str {
        "pending-reboot"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        tokio::task::spawn_blocking(pending_reboot_blocking)
            .await
            .map_err(|e| DMedicError::Other(format!("pending_reboot spawn_blocking join: {e}")))?
    }
}

fn pending_reboot_blocking() -> DMedicResult<Vec<Finding>> {
    // Standart pending reboot işaretçileri — birden fazla yerden okunabilir:
    let cbs = registry::key_exists(
        registry::HKLM,
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Component Based Servicing\RebootPending",
    );
    let wu = registry::key_exists(
        registry::HKLM,
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\WindowsUpdate\Auto Update\RebootRequired",
    );
    let pending_renames = registry::read_string(
        registry::HKLM,
        r"SYSTEM\CurrentControlSet\Control\Session Manager",
        "PendingFileRenameOperations",
    )
    .is_some();

    if !cbs && !wu && !pending_renames {
        return Ok(Vec::new());
    }

    Ok(vec![Finding {
        id: "pending-reboot".to_string(),
        category: Category::Stability,
        priority: Priority::Medium,
        action_type: ActionType::Reboot,
        title: LocalizedText::new(
            "Yeniden başlatma bekliyor",
            "Reboot pending",
        ),
        description: LocalizedText::new(
            "Bir güncelleme, kurulum veya dosya değişikliği yeniden başlatmayı bekliyor. \
             Bekletmek SFC/DISM ve sonraki kurulumları başarısız kılabilir."
                .to_string(),
            "An update, install, or file change is awaiting reboot. Postponing may cause \
             subsequent SFC/DISM and installations to fail."
                .to_string(),
        ),
        estimated_gain: EstimatedGain::Stability,
        risk: RiskLevel::None,
        reboot_required: true,
        action_id: None,
        guide_id: None,
        evidence: json!({
            "cbs_reboot_pending": cbs,
            "wu_reboot_required": wu,
            "pending_file_renames": pending_renames,
        }),
    }])
}
