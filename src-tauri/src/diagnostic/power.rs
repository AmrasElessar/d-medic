use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use wmi::WMIConnection;

use super::{registry, wmi as wmi_helper, Check};
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

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Win32ComputerSystem {
    automatic_managed_pagefile: Option<bool>,
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

/// #8 — Hibernation açık + sistem diski dar (< 50 GB boş).
pub struct HibernationCheck;

#[async_trait]
impl Check for HibernationCheck {
    fn id(&self) -> &'static str {
        "hibernation"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let snap = wmi_helper::read_snapshot().await?;
        let enabled = tokio::task::spawn_blocking(|| {
            registry::read_dword(
                registry::HKLM,
                r"SYSTEM\CurrentControlSet\Control\Power",
                "HibernateEnabled",
            )
        })
        .await
        .map_err(|e| DMedicError::Other(format!("hib spawn_blocking join: {e}")))?
        .unwrap_or(0);

        if enabled == 0 || snap.primary_disk_free_gb >= 50.0 {
            return Ok(Vec::new());
        }

        // Hiberfil ~RAM kadar yer kaplar.
        let hiberfil_est_mb = (snap.total_ram_gb * 1024.0) as u32;
        Ok(vec![Finding {
            id: "hibernation".to_string(),
            category: Category::Storage,
            priority: Priority::Medium,
            action_type: ActionType::Automatic,
            title: LocalizedText::new(
                "Hibernation aktif + disk dar",
                "Hibernation enabled with low disk",
            ),
            description: LocalizedText::new(
                format!(
                    "C: sürücüsünde {:.1} GB boş alan ve hiberfil.sys ~{:.1} GB tahsis ediyor. \
                     Masaüstü sistemlerde hibernation'a genelde ihtiyaç yoktur — kapatınca \
                     disk alanı geri kazanılır.",
                    snap.primary_disk_free_gb,
                    snap.total_ram_gb
                ),
                format!(
                    "C: has {:.1} GB free and hiberfil.sys takes ~{:.1} GB. Hibernation is \
                     usually unnecessary on desktops — disabling reclaims the space.",
                    snap.primary_disk_free_gb, snap.total_ram_gb
                ),
            ),
            estimated_gain: EstimatedGain::DiskMb {
                value: hiberfil_est_mb,
            },
            risk: RiskLevel::Low,
            reboot_required: false,
            action_id: Some("disable-hibernation".to_string()),
            guide_id: None,
            evidence: json!({
                "hibernate_enabled": enabled,
                "free_gb": snap.primary_disk_free_gb,
                "total_ram_gb": snap.total_ram_gb,
            }),
        }])
    }
}

/// #7 — RAM az + pagefile otomatik yönetilen.
pub struct PagefileCheck;

#[async_trait]
impl Check for PagefileCheck {
    fn id(&self) -> &'static str {
        "pagefile"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let snap = wmi_helper::read_snapshot().await?;
        if snap.total_ram_gb >= 6.0 {
            return Ok(Vec::new());
        }

        let auto_managed = tokio::task::spawn_blocking(pagefile_auto_managed)
            .await
            .map_err(|e| DMedicError::Other(format!("pagefile spawn_blocking join: {e}")))?;

        if !auto_managed.unwrap_or(true) {
            // Manuel yönetimde — kullanıcı bilinçli ayarlamış varsayılır.
            return Ok(Vec::new());
        }

        let recommended_mb = (snap.total_ram_gb * 1024.0 * 1.5) as u32;
        Ok(vec![Finding {
            id: "pagefile".to_string(),
            category: Category::Performance,
            priority: Priority::High,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                "Düşük RAM'de pagefile otomatik",
                "Auto-managed pagefile on low-RAM system",
            ),
            description: LocalizedText::new(
                format!(
                    "Sistemde {:.1} GB RAM var ve pagefile boyutu Windows tarafından \
                     otomatik yönetiliyor. Düşük RAM'li sistemlerde sabit boyutlu pagefile \
                     (önerilen: ~{} MB) dosya parçalanmasını azaltır ve OOM riskini düşürür.",
                    snap.total_ram_gb, recommended_mb
                ),
                format!(
                    "System has {:.1} GB RAM with auto-managed pagefile. On low-RAM systems \
                     a fixed-size pagefile (recommended ~{} MB) reduces file fragmentation \
                     and lowers OOM risk.",
                    snap.total_ram_gb, recommended_mb
                ),
            ),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::Low,
            reboot_required: true,
            action_id: None,
            guide_id: Some("pagefile-tune".to_string()),
            evidence: json!({
                "total_ram_gb": snap.total_ram_gb,
                "auto_managed": true,
                "recommended_size_mb": recommended_mb,
            }),
        }])
    }
}

fn pagefile_auto_managed() -> Option<bool> {
    let com = wmi_helper::init_com_lib();
    let cimv2 = WMIConnection::new(com).ok()?;
    let rows: Vec<Win32ComputerSystem> = cimv2.query().ok()?;
    rows.into_iter()
        .next()
        .and_then(|s| s.automatic_managed_pagefile)
}

/// #16 — Görsel efektler tam açık + RAM az.
pub struct VisualEffectsCheck;

#[async_trait]
impl Check for VisualEffectsCheck {
    fn id(&self) -> &'static str {
        "visual-effects"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let snap = wmi_helper::read_snapshot().await?;
        if snap.total_ram_gb >= 6.0 {
            return Ok(Vec::new());
        }

        // VisualFXSetting: 0=Let Windows choose, 1=Best appearance,
        // 2=Best performance, 3=Custom. 1 (best appearance) = en pahalı.
        let setting = tokio::task::spawn_blocking(|| {
            registry::read_dword(
                registry::HKCU,
                r"Software\Microsoft\Windows\CurrentVersion\Explorer\VisualEffects",
                "VisualFXSetting",
            )
        })
        .await
        .map_err(|e| DMedicError::Other(format!("vfx spawn_blocking join: {e}")))?;

        // 0 = Windows seçer (genelde best appearance), 1 = best appearance
        let needs_action = matches!(setting, Some(0) | Some(1) | None);
        if !needs_action {
            return Ok(Vec::new());
        }

        Ok(vec![Finding {
            id: "visual-effects".to_string(),
            category: Category::Performance,
            priority: Priority::Medium,
            action_type: ActionType::Automatic,
            title: LocalizedText::new(
                "Görsel efektler performans için kısılabilir",
                "Visual effects can be tuned for performance",
            ),
            description: LocalizedText::new(
                format!(
                    "Sistemde {:.1} GB RAM var ve görsel efektler tam açık. \"En iyi performans\" \
                     moduna geçmek animasyonları/gölgeleri kapatır, eski makinelerde UI \
                     gözle görülür hızlanır.",
                    snap.total_ram_gb
                ),
                format!(
                    "System has {:.1} GB RAM with full visual effects. Switching to \"Best \
                     performance\" disables animations/shadows; UI is noticeably faster on \
                     older machines.",
                    snap.total_ram_gb
                ),
            ),
            estimated_gain: EstimatedGain::CpuPct { value: 5 },
            risk: RiskLevel::None,
            reboot_required: false,
            action_id: Some("set-visual-effects-performance".to_string()),
            guide_id: None,
            evidence: json!({
                "total_ram_gb": snap.total_ram_gb,
                "visual_fx_setting": setting,
            }),
        }])
    }
}
