use async_trait::async_trait;
use serde::Deserialize;
use serde_json::json;
use wmi::WMIConnection;

use super::{wmi as wmi_helper, Check};
use crate::error::{DMedicError, DMedicResult};
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel,
};

/// #20 — Windows Recovery Environment devre dışı (rollback için kritik).
pub struct WindowsReCheck;

#[async_trait]
impl Check for WindowsReCheck {
    fn id(&self) -> &'static str {
        "windows-re"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // reagentc /info çıktısı locale-bağımlı: "Windows RE status: Enabled" veya
        // "Windows RE durumu: Etkin". İngilizce "Enabled" ve Türkçe "Etkin" anahtar
        // kelimelerine birden bak; başarısızsa devre dışı varsayma.
        let out = crate::ps::runner::run_script("reagentc /info 2>&1").await.ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };
        let stdout = out.stdout.to_lowercase();
        // Çıktıda "enabled" veya "etkin" geçiyorsa RE aktif kabul ediyoruz.
        // "disabled"/"devre dışı" var ama "enabled"/"etkin" yoksa pasif.
        let enabled = stdout.contains("enabled") || stdout.contains("etkin");
        let disabled = stdout.contains("disabled") || stdout.contains("devre");
        if enabled && !disabled {
            return Ok(Vec::new());
        }

        Ok(vec![Finding {
            id: "windows-re".to_string(),
            category: Category::Stability,
            priority: Priority::Medium,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                "Windows Recovery devre dışı",
                "Windows Recovery disabled",
            ),
            description: LocalizedText::new(
                "Windows RE pasif görünüyor. Sistem geri yükleme noktası açma, başlangıç \
                 onarımı ve sıfırlama gibi recovery yetenekleri çalışmaz. \
                 `reagentc /enable` ile yeniden etkinleştirilebilir."
                    .to_string(),
                "Windows RE appears inactive. Restore points, startup repair and reset \
                 features won't work. Re-enable with `reagentc /enable`."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::None,
            reboot_required: false,
            action_id: Some("enable-windows-re".to_string()),
            guide_id: None,
            evidence: json!({
                "reagentc_output_lower": stdout.lines().take(10).collect::<Vec<_>>().join(" | "),
            }),
        }])
    }
}

/// #24 — CPU Win11 resmi destek listesinde yok.
pub struct CpuCompatibilityCheck;

#[async_trait]
impl Check for CpuCompatibilityCheck {
    fn id(&self) -> &'static str {
        "cpu-compat"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let snap = wmi_helper::read_snapshot().await?;
        if snap.cpu_name.is_empty() {
            return Ok(Vec::new());
        }

        let Some(reason) = unsupported_reason(&snap.cpu_name) else {
            return Ok(Vec::new());
        };

        Ok(vec![Finding {
            id: "cpu-compat".to_string(),
            category: Category::Compatibility,
            priority: Priority::High,
            action_type: ActionType::NotPossible,
            title: LocalizedText::new(
                "CPU Windows 11 resmi destek listesinde değil",
                "CPU not on Windows 11 official support list",
            ),
            description: LocalizedText::new(
                format!(
                    "İşlemci \"{}\" Microsoft'un Win11 desteklenen CPU listesinde değil ({}). \
                     Yükseltme installer'ı engelleyebilir; resmi olmayan obnaa atlatma \
                     yöntemleri var ama güvenlik güncellemeleri kesintiye uğrayabilir.",
                    snap.cpu_name, reason
                ),
                format!(
                    "CPU \"{}\" is not on Microsoft's Win11 supported list ({}). The upgrade \
                     installer may block; unsupported workarounds exist but security updates \
                     can be interrupted.",
                    snap.cpu_name, reason
                ),
            ),
            estimated_gain: EstimatedGain::None,
            risk: RiskLevel::Medium,
            reboot_required: false,
            action_id: None,
            guide_id: Some("cpu-incompatible".to_string()),
            evidence: json!({
                "cpu_name": snap.cpu_name,
                "reason": reason,
            }),
        }])
    }
}

/// Bilinen Win11 desteklemeyen CPU pattern'ları. Yanlış pozitiften kaçınmak
/// için sadece NET tanınan eski jenerasyonlar — şüpheli olanı atlıyoruz.
fn unsupported_reason(name: &str) -> Option<&'static str> {
    let n = name.to_ascii_lowercase();

    // Intel — Core 2, Core 1-7. gen, eski Atom/Pentium.
    if n.contains("core 2") || n.contains("core(tm) 2") {
        return Some("Intel Core 2 ailesi (2006-2011)");
    }
    // i3/i5/i7/i9-1xxx ... -7xxx (1.-7. nesil) — desteklenmiyor.
    // 8. nesil ve üstü destekleniyor.
    for gen in ['1', '2', '3', '4', '5', '6', '7'] {
        let prefixes = [
            format!("i3-{gen}"),
            format!("i5-{gen}"),
            format!("i7-{gen}"),
            format!("i9-{gen}"),
        ];
        for p in &prefixes {
            // i7-7700 evet, i7-7920 evet, ama i7-10700 (10. nesil) HAYIR.
            // 4 haneli kontrol: i7-7700 → "i7-7" sonrası ilk char rakam değilse 5 haneli
            // gibi yanıltıcı olabilir. Daha güvenli: "i*-Xddd" pattern (4 haneli).
            if let Some(idx) = n.find(p.as_str()) {
                let after = &n[idx + p.len()..];
                let rest_digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
                // i7-7700 → "i7-7" + "700" = 3 rakam sonra → toplam 4 hane → match
                // i7-10700 → "i7-1" + "0700" = 4 rakam sonra → toplam 5 hane → 10. nesil
                if rest_digits.len() == 3 {
                    return Some("Intel Core 1.-7. nesil");
                }
            }
        }
    }

    // AMD — FX, Phenom, eski A-series.
    if n.contains("fx-") && n.contains("amd") {
        return Some("AMD FX serisi (Bulldozer/Piledriver)");
    }
    if n.contains("phenom") {
        return Some("AMD Phenom serisi");
    }
    // AMD Ryzen 1xxx (1st gen, Zen) — sadece 4 haneli ve 1 ile başlayan.
    if let Some(idx) = n.find("ryzen ") {
        let after = &n[idx + "ryzen ".len()..];
        // "ryzen 7 1700" → tier "7", boşluk, model "1700"
        let parts: Vec<&str> = after.split_whitespace().collect();
        if parts.len() >= 2 {
            let model = parts[1];
            if model.len() == 4 && model.starts_with('1') && model.chars().all(|c| c.is_ascii_digit()) {
                return Some("AMD Ryzen 1. nesil (Zen)");
            }
        }
    }

    None
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct SoftwareLicensingProduct {
    name: Option<String>,
    license_status: Option<u32>,
    partial_product_key: Option<String>,
    application_id: Option<String>,
}

const WINDOWS_APP_ID: &str = "55c92734-d682-4d71-983e-d6ec3f16059f";

/// #25 — Windows aktif değil.
pub struct ActivationCheck;

#[async_trait]
impl Check for ActivationCheck {
    fn id(&self) -> &'static str {
        "activation"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        tokio::task::spawn_blocking(activation_blocking)
            .await
            .map_err(|e| DMedicError::Other(format!("activation join: {e}")))?
    }
}

fn activation_blocking() -> DMedicResult<Vec<Finding>> {
    let com = wmi_helper::init_com_lib();
    let Ok(cimv2) = WMIConnection::new(com) else {
        return Ok(Vec::new());
    };
    // SoftwareLicensingProduct çok kayıt döner; sadece Windows ApplicationId + key olanları al.
    let products: Vec<SoftwareLicensingProduct> = cimv2
        .raw_query(
            "SELECT Name, LicenseStatus, PartialProductKey, ApplicationID \
             FROM SoftwareLicensingProduct WHERE PartialProductKey <> NULL",
        )
        .unwrap_or_default();

    // Windows ürünü olan ve LicenseStatus != 1 (Licensed) olan ilk kayıt.
    let problem = products.into_iter().find(|p| {
        p.application_id.as_deref() == Some(WINDOWS_APP_ID)
            && p.license_status.unwrap_or(0) != 1
    });

    let Some(p) = problem else {
        return Ok(Vec::new());
    };

    let status_label = match p.license_status.unwrap_or(0) {
        0 => "Lisanssız",
        2 => "OOB Grace (kısıtlı süre)",
        3 => "OOT Grace",
        4 => "Bağlantı kesilmiş Grace",
        5 => "Bildirim modu",
        6 => "Genişletilmiş Grace",
        _ => "Bilinmiyor",
    };

    Ok(vec![Finding {
        id: "activation".to_string(),
        category: Category::Compatibility,
        priority: Priority::High,
        action_type: ActionType::Guided,
        title: LocalizedText::new(
            format!("Windows aktif değil ({status_label})"),
            format!("Windows not activated ({status_label})"),
        ),
        description: LocalizedText::new(
            format!(
                "Ürün: {}. Lisans durumu={}. Aktive edilmemiş Windows belirli kişiselleştirme \
                 ayarlarını kilitler ve uyarı suluyolu gösterir.",
                p.name.clone().unwrap_or_default(),
                p.license_status.unwrap_or(0)
            ),
            format!(
                "Product: {}. License status={}. Unactivated Windows locks personalization \
                 and shows nag watermarks.",
                p.name.unwrap_or_default(),
                p.license_status.unwrap_or(0)
            ),
        ),
        estimated_gain: EstimatedGain::None,
        risk: RiskLevel::None,
        reboot_required: false,
        action_id: None,
        guide_id: Some("activation-error".to_string()),
        evidence: json!({
            "license_status": p.license_status,
            "partial_key": p.partial_product_key,
        }),
    }])
}

/// #26 — Driver güncel değil — 2 yıldan eski signed driver sayısı.
pub struct DriverFreshnessCheck;

#[async_trait]
impl Check for DriverFreshnessCheck {
    fn id(&self) -> &'static str {
        "driver-freshness"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // CIM standart query, admin gerektirmez. Get-WindowsDriver daha detaylı
        // ama elevated ister; CIM ile yetiniyoruz.
        let script = "$cutoff = (Get-Date).AddYears(-2)\n\
            $drv = Get-CimInstance Win32_PnPSignedDriver -ErrorAction SilentlyContinue\n\
            $total = @($drv).Count\n\
            $old = @($drv | Where-Object { $_.DriverDate -and $_.DriverDate -lt $cutoff }).Count\n\
            \"$total,$old\"";
        let out = crate::ps::runner::run_script(script).await.ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };
        let parts: Vec<&str> = out.stdout.trim().split(',').collect();
        if parts.len() != 2 {
            return Ok(Vec::new());
        }
        let total: u32 = parts[0].parse().unwrap_or(0);
        let old: u32 = parts[1].parse().unwrap_or(0);
        if total == 0 || old < 5 {
            return Ok(Vec::new());
        }

        let pct = (old as f32 / total as f32) * 100.0;
        let priority = if pct >= 30.0 {
            Priority::High
        } else if pct >= 15.0 {
            Priority::Medium
        } else {
            Priority::Low
        };

        Ok(vec![Finding {
            id: "driver-freshness".to_string(),
            category: Category::Stability,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                format!("{old} adet 2 yıldan eski driver (toplam {total})"),
                format!("{old} drivers older than 2 years (of {total})"),
            ),
            description: LocalizedText::new(
                "Eski driver'lar BSOD, donanım performans kaybı ve güvenlik açığı kaynağıdır. \
                 Anakart/üretici sitesinden veya Windows Update Catalog'dan güncel sürümlere \
                 bakın; özellikle chipset/GPU/SSD driver'ları öncelikli."
                    .to_string(),
                "Outdated drivers cause BSODs, hardware perf loss and security holes. Check \
                 vendor sites or Windows Update Catalog; chipset/GPU/SSD drivers first."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::Low,
            reboot_required: true,
            action_id: None,
            guide_id: Some("driver-update".to_string()),
            evidence: json!({
                "total_drivers": total,
                "older_than_2y": old,
                "old_pct": pct,
            }),
        }])
    }
}
