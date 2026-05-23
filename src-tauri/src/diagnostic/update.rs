use async_trait::async_trait;
use serde_json::json;

use super::{registry, Check};
use crate::error::{DMedicError, DMedicResult};
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel,
};

/// #14 — Windows Update takılı.
pub struct WindowsUpdateStuckCheck;

#[async_trait]
impl Check for WindowsUpdateStuckCheck {
    fn id(&self) -> &'static str {
        "wu-stuck"
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO: SoftwareDistribution\Download boyutu + wuauserv durumu + son
        // başarılı update tarihi. Bu Faz 2 PS batch ile gelecek.
        Ok(Vec::new())
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
