use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use wmi::WMIConnection;

use super::{wmi as wmi_helper, Check};
use crate::error::{DMedicError, DMedicResult};
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel, ScanKind,
};

/// #2 — SysMain (Superfetch) + HDD kombinasyonu = sürekli disk thrash.
pub struct SysmainHddCheck;

#[async_trait]
impl Check for SysmainHddCheck {
    fn id(&self) -> &'static str {
        "sysmain-hdd"
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO: snapshot.primary_disk_type == "HDD" && services.SysMain.status == "Running"
        Ok(Vec::new())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct MsftPhysicalDiskHealth {
    friendly_name: Option<String>,
    /// 0=Healthy, 1=Warning, 2=Unhealthy, 3=Unknown.
    health_status: Option<u16>,
    /// 3=HDD, 4=SSD, 5=SCM.
    media_type: Option<u16>,
    size: Option<u64>,
}

/// #6 — SMART kritik uyarı: MSFT_PhysicalDisk.HealthStatus != Healthy.
pub struct SmartHealthCheck;

#[async_trait]
impl Check for SmartHealthCheck {
    fn id(&self) -> &'static str {
        "smart-health"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        tokio::task::spawn_blocking(smart_health_blocking)
            .await
            .map_err(|e| DMedicError::Other(format!("smart spawn_blocking join: {e}")))?
    }
}

fn smart_health_blocking() -> DMedicResult<Vec<Finding>> {
    let com = wmi_helper::init_com_lib();
    let storage = match WMIConnection::with_namespace_path(
        "ROOT\\Microsoft\\Windows\\Storage",
        com,
    ) {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!(error = %e, "Storage namespace connect failed");
            return Ok(Vec::new());
        }
    };

    let disks: Vec<MsftPhysicalDiskHealth> = storage.query().unwrap_or_default();
    let mut findings = Vec::new();
    for disk in disks {
        let status = match disk.health_status {
            Some(0) | None => continue, // Healthy veya bilgi yok
            Some(s) => s,
        };
        let (priority, label_tr, label_en) = match status {
            1 => (Priority::High, "Uyarı", "Warning"),
            2 => (Priority::Critical, "Sağlıksız", "Unhealthy"),
            _ => continue, // Unknown — bulgu üretme
        };
        let name = disk.friendly_name.unwrap_or_else(|| "Bilinmeyen disk".into());
        let size_gb = disk
            .size
            .map(|s| s as f64 / (1024.0 * 1024.0 * 1024.0))
            .unwrap_or(0.0);
        findings.push(Finding {
            id: format!("smart-health:{}", name),
            category: Category::Data,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                format!("Disk sağlığı: {label_tr} — {name}"),
                format!("Disk health: {label_en} — {name}"),
            ),
            description: LocalizedText::new(
                format!(
                    "{name} ({:.0} GB) SMART HealthStatus={status}. Veri kaybı riski var; \
                     yedek alıp diski en kısa sürede değiştirin.",
                    size_gb
                ),
                format!(
                    "{name} ({:.0} GB) reports SMART HealthStatus={status}. Data loss risk; \
                     back up and replace as soon as possible.",
                    size_gb
                ),
            ),
            estimated_gain: EstimatedGain::DataSafety,
            risk: RiskLevel::None,
            reboot_required: false,
            action_id: None,
            guide_id: Some("smart-disk-replace".to_string()),
            evidence: json!({
                "friendly_name": name,
                "health_status": status,
                "media_type": disk.media_type,
                "size_bytes": disk.size,
            }),
        });
    }
    Ok(findings)
}

const DISK_FREE_CRITICAL_PCT: f32 = 5.0;
const DISK_FREE_HIGH_PCT: f32 = 15.0;
const DISK_FREE_MEDIUM_PCT: f32 = 25.0;

/// #22 — Sistem disk doluluk yüzdesi.
pub struct DiskFullCheck;

#[async_trait]
impl Check for DiskFullCheck {
    fn id(&self) -> &'static str {
        "disk-full"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let snap = wmi_helper::read_snapshot().await?;
        if snap.primary_disk_size_gb <= 0.0 {
            return Ok(Vec::new());
        }
        let free_pct = (snap.primary_disk_free_gb / snap.primary_disk_size_gb) * 100.0;
        let (priority, label_tr, label_en) = if free_pct < DISK_FREE_CRITICAL_PCT {
            (Priority::Critical, "kritik", "critical")
        } else if free_pct < DISK_FREE_HIGH_PCT {
            (Priority::High, "yüksek", "high")
        } else if free_pct < DISK_FREE_MEDIUM_PCT {
            (Priority::Medium, "orta", "medium")
        } else {
            return Ok(Vec::new());
        };

        Ok(vec![Finding {
            id: "disk-full".to_string(),
            category: Category::Storage,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                format!("Sistem diski {label_tr} doluluk: %{:.1} boş", free_pct),
                format!("System disk {label_en} usage: {:.1}% free", free_pct),
            ),
            description: LocalizedText::new(
                format!(
                    "C: sürücüsünde {:.1} GB boş alan kaldı ({:.1} GB toplam). \
                     Geçici dosyaları temizleme, gereksiz uygulamaları kaldırma veya \
                     pagefile/hiberfil ayarı önerilir.",
                    snap.primary_disk_free_gb, snap.primary_disk_size_gb
                ),
                format!(
                    "C: drive has {:.1} GB free ({:.1} GB total). Recommended: clear temp \
                     files, remove unneeded apps, tune pagefile/hiberfil.",
                    snap.primary_disk_free_gb, snap.primary_disk_size_gb
                ),
            ),
            estimated_gain: EstimatedGain::DiskMb {
                value: ((snap.primary_disk_size_gb * 0.1) * 1024.0) as u32,
            },
            risk: RiskLevel::Low,
            reboot_required: false,
            action_id: None,
            guide_id: Some("disk-cleanup".to_string()),
            evidence: json!({
                "free_gb": snap.primary_disk_free_gb,
                "size_gb": snap.primary_disk_size_gb,
                "free_pct": free_pct,
            }),
        }])
    }
}

/// #27 — HDD parçalanma > %10 (deep scan, sadece HDD'de).
pub struct FragmentationCheck;

#[async_trait]
impl Check for FragmentationCheck {
    fn id(&self) -> &'static str {
        "fragmentation"
    }
    fn applicable_in(&self, kind: ScanKind) -> bool {
        matches!(kind, ScanKind::Deep)
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let snap = wmi_helper::read_snapshot().await?;
        // SSD'de defrag zararlı + parçalanma anlamsız. Sadece HDD'de çalıştır.
        if snap.primary_disk_type != "HDD" {
            return Ok(Vec::new());
        }

        // defrag /A /V çıktısı: "Total fragmented space = X%" veya "Toplam parçalanmış alan".
        let out = crate::ps::runner::run_script("defrag C: /A /V")
            .await
            .ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };
        let stdout = out.stdout;

        // En sağlam parsing: "%" karakteri içeren satırı bul, sayıyı çıkar.
        let pct: Option<u32> = stdout
            .lines()
            .find(|l| {
                let lc = l.to_lowercase();
                (lc.contains("fragmented") || lc.contains("parçalanmış")) && lc.contains('%')
            })
            .and_then(|l| {
                let digits: String = l
                    .chars()
                    .skip_while(|c| !c.is_ascii_digit())
                    .take_while(|c| c.is_ascii_digit())
                    .collect();
                digits.parse().ok()
            });

        let Some(pct) = pct else {
            return Ok(Vec::new());
        };
        if pct < 10 {
            return Ok(Vec::new());
        }

        let priority = if pct >= 30 {
            Priority::High
        } else {
            Priority::Medium
        };

        Ok(vec![Finding {
            id: "fragmentation".to_string(),
            category: Category::Performance,
            priority,
            action_type: ActionType::Automatic,
            title: LocalizedText::new(
                format!("HDD parçalanması %{pct}"),
                format!("HDD fragmentation {pct}%"),
            ),
            description: LocalizedText::new(
                "Sistem diski HDD ve parçalanma seviyesi yüksek; dosya açma süreleri \
                 belirgin yavaşlar. Birleştirme önerilir (`defrag C: /U /V`)."
                    .to_string(),
                "System disk is HDD with significant fragmentation; file access slows down. \
                 Run `defrag C: /U /V`."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::CpuPct { value: 5 },
            risk: RiskLevel::None,
            reboot_required: false,
            action_id: Some("defrag-system".to_string()),
            guide_id: None,
            evidence: json!({
                "fragmentation_pct": pct,
                "disk_type": snap.primary_disk_type,
            }),
        }])
    }
}
