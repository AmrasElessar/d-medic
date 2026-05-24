use async_trait::async_trait;
use serde_json::json;

use super::Check;
use crate::error::DMedicResult;
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel, ScanKind,
};
use crate::ps;

/// Get-WinEvent FilterHashtable ile log+level+(opsiyonel id)+gün sayısı için event sayısını döner.
async fn count_events(log: &str, level: u32, ids: Option<&str>, days: u32) -> Option<u32> {
    let filter = match ids {
        Some(i) => format!(
            "@{{LogName='{log}'; Level={level}; Id={i}; StartTime=(Get-Date).AddDays(-{days})}}"
        ),
        None => format!(
            "@{{LogName='{log}'; Level={level}; StartTime=(Get-Date).AddDays(-{days})}}"
        ),
    };
    let script = format!(
        "try {{ $e = Get-WinEvent -FilterHashtable {filter} -ErrorAction Stop; \
         [string](@($e).Count) }} catch {{ '0' }}"
    );
    let out = ps::runner::run_script(&script).await.ok()?;
    out.stdout.trim().parse::<u32>().ok()
}

/// #21 — Son 30 günde Kernel-Power 41 (beklenmedik kapanma/BSOD sonrası restart).
pub struct BsodHistoryCheck;

#[async_trait]
impl Check for BsodHistoryCheck {
    fn id(&self) -> &'static str {
        "bsod-history"
    }
    fn applicable_in(&self, kind: ScanKind) -> bool {
        matches!(kind, ScanKind::Deep)
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // Level=2 (Error) + Id=41 (Kernel-Power "kritik" power loss); bazı sistemlerde
        // Level=4 (Information) görünür ama Id=41 her zaman aynı anlamı taşır.
        let count = count_events("System", 2, Some("41"), 30).await.unwrap_or(0);
        if count == 0 {
            return Ok(Vec::new());
        }
        let (priority, label_tr, label_en) = if count >= 10 {
            (Priority::High, "çok sık", "very frequent")
        } else if count >= 3 {
            (Priority::Medium, "sık", "frequent")
        } else {
            (Priority::Low, "ara sıra", "occasional")
        };

        Ok(vec![Finding {
            id: "bsod-history".to_string(),
            category: Category::Stability,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                format!("Son 30 günde {count} beklenmedik kapanma ({label_tr})"),
                format!(
                    "{count} unexpected shutdowns in last 30 days ({label_en})"
                ),
            ),
            description: LocalizedText::new(
                "Kernel-Power Event 41 — sistem donma/BSOD/güç kesintisi sonrası yeniden \
                 başlamış. Driver güncellemesi, RAM testi, sıcaklık takibi ve PSU sağlığı \
                 sırayla incelenmeli."
                    .to_string(),
                "Kernel-Power Event 41 — system rebooted after freeze/BSOD/power loss. \
                 Investigate drivers, RAM (memtest), thermals and PSU health."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::None,
            reboot_required: false,
            action_id: None,
            guide_id: Some("bsod-analysis".to_string()),
            evidence: json!({ "kernel_power_41_count_30d": count }),
        }])
    }
}

/// #28 — Son 7 günde Application log'unda Critical seviye event sayısı.
pub struct EventLogCriticalCheck;

#[async_trait]
impl Check for EventLogCriticalCheck {
    fn id(&self) -> &'static str {
        "event-log-critical"
    }
    fn applicable_in(&self, kind: ScanKind) -> bool {
        matches!(kind, ScanKind::Deep)
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let count = count_events("Application", 1, None, 7).await.unwrap_or(0);
        if count < 5 {
            return Ok(Vec::new());
        }
        let priority = if count >= 30 {
            Priority::High
        } else {
            Priority::Medium
        };
        Ok(vec![Finding {
            id: "event-log-critical".to_string(),
            category: Category::Stability,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                format!("Son 7 günde {count} kritik uygulama olayı"),
                format!("{count} critical app events in last 7 days"),
            ),
            description: LocalizedText::new(
                "Application Event Log'unda Critical seviye olaylar birikmiş. Event Viewer'ı \
                 açıp en sık hataları üreten uygulamayı tespit edin — genellikle bozuk bir \
                 plugin, eski driver veya çakışan güvenlik yazılımı söz konusudur."
                    .to_string(),
                "Critical events piling up in Application log. Open Event Viewer and find \
                 the most frequent source — usually a broken plugin, outdated driver or \
                 conflicting security software."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::None,
            reboot_required: false,
            action_id: None,
            guide_id: None,
            evidence: json!({ "critical_event_count_7d": count }),
        }])
    }
}
