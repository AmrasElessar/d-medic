use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use wmi::WMIConnection;

use super::{wmi as wmi_helper, Check};
use crate::error::{DMedicError, DMedicResult};
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel,
};

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32StartupCommand {
    name: Option<String>,
    location: Option<String>,
    user: Option<String>,
}

const THRESHOLD_HIGH: usize = 12;
const THRESHOLD_MEDIUM: usize = 8;

/// #9 — Çok sayıda başlangıç uygulaması.
pub struct StartupCountCheck;

#[async_trait]
impl Check for StartupCountCheck {
    fn id(&self) -> &'static str {
        "startup-count"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        tokio::task::spawn_blocking(startup_blocking)
            .await
            .map_err(|e| DMedicError::Other(format!("startup spawn_blocking join: {e}")))?
    }
}

fn startup_blocking() -> DMedicResult<Vec<Finding>> {
    let com = wmi_helper::init_com_lib();
    let cimv2 = match WMIConnection::new(com) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "CIMV2 connect failed");
            return Ok(Vec::new());
        }
    };

    let items: Vec<Win32StartupCommand> = cimv2.query().unwrap_or_default();
    let count = items.len();
    let (priority, est_boot_pct) = if count >= THRESHOLD_HIGH {
        (Priority::High, 25u8)
    } else if count >= THRESHOLD_MEDIUM {
        (Priority::Medium, 12u8)
    } else {
        return Ok(Vec::new());
    };

    // Evidence için ilk 10 girdiyi {name, location, user} olarak topla.
    let sample: Vec<serde_json::Value> = items
        .iter()
        .take(10)
        .map(|c| {
            json!({
                "name": c.name,
                "location": c.location,
                "user": c.user,
            })
        })
        .collect();

    Ok(vec![Finding {
        id: "startup-count".to_string(),
        category: Category::Performance,
        priority,
        action_type: ActionType::Guided,
        title: LocalizedText::new(
            format!("Açılışta {count} uygulama başlıyor"),
            format!("{count} startup applications"),
        ),
        description: LocalizedText::new(
            "Açılışta çalışan uygulamalar oturum açma süresini ve RAM kullanımını \
             artırır. Görev Yöneticisi → Başlangıç sekmesinden kullanılmayanları \
             devre dışı bırakın."
                .to_string(),
            "Startup apps increase login time and RAM usage. Disable unused ones from \
             Task Manager → Startup tab."
                .to_string(),
        ),
        estimated_gain: EstimatedGain::BootPct {
            value: est_boot_pct,
        },
        risk: RiskLevel::Low,
        reboot_required: false,
        action_id: None,
        guide_id: None,
        evidence: json!({
            "count": count,
            "sample": sample,
        }),
    }])
}
