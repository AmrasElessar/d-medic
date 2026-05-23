use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use wmi::WMIConnection;

use super::{wmi as wmi_helper, Check};
use crate::error::{DMedicError, DMedicResult};
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel,
};

#[derive(Deserialize, Clone)]
#[serde(rename = "Win32_Service", rename_all = "PascalCase")]
struct Win32Service {
    name: Option<String>,
    state: Option<String>,
    start_mode: Option<String>,
}

fn query_services(names: &[&str]) -> Vec<Win32Service> {
    let com = wmi_helper::init_com_lib();
    let Ok(cimv2) = WMIConnection::new(com) else {
        return Vec::new();
    };
    let in_list = names
        .iter()
        .map(|n| format!("'{n}'"))
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "SELECT Name, State, StartMode FROM Win32_Service WHERE Name IN ({in_list})"
    );
    cimv2.raw_query::<Win32Service>(&query).unwrap_or_default()
}

fn is_running(svc: &Win32Service) -> bool {
    svc.state.as_deref() == Some("Running")
}

/// SysMain (HDD'de zararlı) + WSearch + arka plan servisleri.
pub struct BloatServicesCheck;

#[async_trait]
impl Check for BloatServicesCheck {
    fn id(&self) -> &'static str {
        "bloat-services"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let snap = wmi_helper::read_snapshot().await?;
        let svcs = tokio::task::spawn_blocking(|| query_services(&["SysMain", "WSearch"]))
            .await
            .map_err(|e| DMedicError::Other(format!("bloat-services join: {e}")))?;

        let mut findings = Vec::new();

        // SysMain HDD'de prefetch için sürekli disk thrash yapar → critical.
        if snap.primary_disk_type == "HDD" {
            if let Some(sysmain) = svcs.iter().find(|s| s.name.as_deref() == Some("SysMain")) {
                if is_running(sysmain) {
                    findings.push(Finding {
                        id: "sysmain-hdd".to_string(),
                        category: Category::Performance,
                        priority: Priority::High,
                        action_type: ActionType::Automatic,
                        title: LocalizedText::new(
                            "SysMain (Superfetch) HDD'de aktif",
                            "SysMain (Superfetch) running on HDD",
                        ),
                        description: LocalizedText::new(
                            "SysMain SSD'de faydalıdır, HDD'de sürekli prefetch yaparak diski \
                             yorar ve genel sistem yavaşlığına neden olur. Kapatılması önerilir."
                                .to_string(),
                            "SysMain helps on SSD; on HDD it constantly prefetches and slows \
                             the whole system. Disabling is recommended."
                                .to_string(),
                        ),
                        estimated_gain: EstimatedGain::CpuPct { value: 10 },
                        risk: RiskLevel::Low,
                        reboot_required: false,
                        action_id: Some("disable-sysmain".to_string()),
                        guide_id: None,
                        evidence: json!({
                            "disk_type": snap.primary_disk_type,
                            "sysmain_state": sysmain.state,
                            "sysmain_start_mode": sysmain.start_mode,
                        }),
                    });
                }
            }
        }

        // WSearch (Windows Search Indexer) — düşük RAM + arka plan yükü.
        if let Some(wsearch) = svcs.iter().find(|s| s.name.as_deref() == Some("WSearch")) {
            if is_running(wsearch) && snap.total_ram_gb < 4.0 {
                findings.push(Finding {
                    id: "wsearch-low-ram".to_string(),
                    category: Category::Performance,
                    priority: Priority::Medium,
                    action_type: ActionType::Automatic,
                    title: LocalizedText::new(
                        "Düşük RAM'de Windows Search aktif",
                        "Windows Search active on low-RAM system",
                    ),
                    description: LocalizedText::new(
                        format!(
                            "Sistemde {:.1} GB RAM var ve Windows Search Indexer arka planda \
                             dosya tarıyor. Düşük RAM'de devre dışı bırakmak hatırı sayılır \
                             rahatlama sağlar (arama yine çalışır, sadece yavaş başlar).",
                            snap.total_ram_gb
                        ),
                        format!(
                            "System has {:.1} GB RAM with Windows Search Indexer scanning. \
                             On low RAM, disabling gives noticeable relief (search still works, \
                             just slower).",
                            snap.total_ram_gb
                        ),
                    ),
                    estimated_gain: EstimatedGain::RamMb { value: 200 },
                    risk: RiskLevel::Low,
                    reboot_required: false,
                    action_id: Some("disable-wsearch".to_string()),
                    guide_id: None,
                    evidence: json!({
                        "total_ram_gb": snap.total_ram_gb,
                        "wsearch_state": wsearch.state,
                    }),
                });
            }
        }

        Ok(findings)
    }
}

/// #10 — DiagTrack (Connected User Experiences) + dmwappushservice telemetri.
pub struct TelemetryCheck;

#[async_trait]
impl Check for TelemetryCheck {
    fn id(&self) -> &'static str {
        "telemetry-services"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let svcs = tokio::task::spawn_blocking(|| {
            query_services(&["DiagTrack", "dmwappushservice"])
        })
        .await
        .map_err(|e| DMedicError::Other(format!("telemetry join: {e}")))?;

        let running: Vec<String> = svcs
            .iter()
            .filter(|s| is_running(s))
            .filter_map(|s| s.name.clone())
            .collect();

        if running.is_empty() {
            return Ok(Vec::new());
        }

        Ok(vec![Finding {
            id: "telemetry-services".to_string(),
            category: Category::Performance,
            priority: Priority::Low,
            action_type: ActionType::Automatic,
            title: LocalizedText::new(
                "Telemetri servisleri çalışıyor",
                "Telemetry services running",
            ),
            description: LocalizedText::new(
                format!(
                    "Microsoft veri toplama servisleri aktif: {}. Devre dışı bırakmak küçük \
                     bir CPU/disk yükü düşürür. (Windows Update için zorunlu değildir.)",
                    running.join(", ")
                ),
                format!(
                    "Microsoft telemetry services active: {}. Disabling reduces a small CPU/disk \
                     load. (Not required by Windows Update.)",
                    running.join(", ")
                ),
            ),
            estimated_gain: EstimatedGain::RamMb { value: 80 },
            risk: RiskLevel::None,
            reboot_required: false,
            action_id: Some("disable-telemetry".to_string()),
            guide_id: None,
            evidence: json!({ "running": running }),
        }])
    }
}
