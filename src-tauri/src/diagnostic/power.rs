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
struct Win32PowerPlan {
    element_name: Option<String>,
    instance_id: Option<String>,
    is_active: Option<bool>,
}

/// Plan GUID'leri locale'den bağımsız stabil — ElementName farklı dilde değişir.
const GUID_BALANCED: &str = "381b4222-f694-41f0-9685-ff5bb260df2e";
const GUID_POWER_SAVER: &str = "a1841308-3541-4fab-bc81-f71556f20b4a";
const GUID_HIGH_PERFORMANCE: &str = "8c5e7fda-e8bf-4a96-9a85-a6e23a8c635c";
const GUID_ULTIMATE: &str = "e9a42b02-d5df-448d-aa00-03f14749eb61";

/// #17 — Aktif power plan Balanced/PowerSaver ise performans bulgusu.
pub struct PowerPlanCheck;

#[async_trait]
impl Check for PowerPlanCheck {
    fn id(&self) -> &'static str {
        "power-plan"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        tokio::task::spawn_blocking(power_plan_blocking)
            .await
            .map_err(|e| DMedicError::Other(format!("power_plan spawn_blocking join: {e}")))?
    }
}

fn power_plan_blocking() -> DMedicResult<Vec<Finding>> {
    let com = wmi_helper::init_com_lib();
    let power = match WMIConnection::with_namespace_path("ROOT\\CIMV2\\power", com) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "ROOT\\CIMV2\\power connect failed");
            return Ok(Vec::new());
        }
    };

    let plans: Vec<Win32PowerPlan> = power.query().unwrap_or_default();
    let active = plans.into_iter().find(|p| p.is_active == Some(true));
    let Some(plan) = active else {
        return Ok(Vec::new());
    };

    let guid = plan
        .instance_id
        .as_deref()
        .and_then(|s| s.rsplit(['\\', ':']).next())
        .map(|s| s.trim_matches(['{', '}']).to_ascii_lowercase())
        .unwrap_or_default();

    let (priority, est_cpu) = match guid.as_str() {
        g if g == GUID_HIGH_PERFORMANCE || g == GUID_ULTIMATE => return Ok(Vec::new()),
        g if g == GUID_POWER_SAVER => (Priority::High, 15u8),
        g if g == GUID_BALANCED => (Priority::Medium, 8u8),
        _ => return Ok(Vec::new()),
    };

    let name = plan.element_name.unwrap_or_else(|| "Bilinmeyen".into());
    Ok(vec![Finding {
        id: "power-plan".to_string(),
        category: Category::Performance,
        priority,
        action_type: ActionType::Automatic,
        title: LocalizedText::new(
            format!("Güç planı: {name}"),
            format!("Power plan: {name}"),
        ),
        description: LocalizedText::new(
            "Aktif güç planı CPU performansını sınırlandırıyor. \"Yüksek Performans\" \
             planına geçmek özellikle masaüstü sistemlerde belirgin hızlanma sağlar."
                .to_string(),
            "Active power plan throttles CPU. Switching to \"High Performance\" yields \
             noticeable speed-up, especially on desktops."
                .to_string(),
        ),
        estimated_gain: EstimatedGain::CpuPct { value: est_cpu },
        risk: RiskLevel::Low,
        reboot_required: false,
        action_id: Some("set-high-performance-plan".to_string()),
        guide_id: None,
        evidence: json!({
            "active_plan": name,
            "guid": guid,
        }),
    }])
}

/// #8 — Hibernation açık + disk < 50 GB boş.
pub struct HibernationCheck;

#[async_trait]
impl Check for HibernationCheck {
    fn id(&self) -> &'static str {
        "hibernation"
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO: hiberfil.sys + snapshot.primary_disk_free_gb
        Ok(Vec::new())
    }
}

/// #7 — Pagefile otomatik + RAM < 6 GB.
pub struct PagefileCheck;

#[async_trait]
impl Check for PagefileCheck {
    fn id(&self) -> &'static str {
        "pagefile"
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO: Win32_PageFileSetting
        Ok(Vec::new())
    }
}

/// #16 — Görsel efektler tam açık + RAM < 6 GB.
pub struct VisualEffectsCheck;

#[async_trait]
impl Check for VisualEffectsCheck {
    fn id(&self) -> &'static str {
        "visual-effects"
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO: HKCU\Control Panel\Desktop\VisualFXSetting
        Ok(Vec::new())
    }
}
