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
#[serde(rename = "Win32_Tpm", rename_all = "PascalCase")]
struct Win32Tpm {
    spec_version: Option<String>,
    is_enabled_initial_value: Option<bool>,
    is_activated_initial_value: Option<bool>,
}

/// #18 — TPM 2.0 yok / devre dışı (Win11 update + BitLocker engeli).
pub struct TpmCheck;

#[async_trait]
impl Check for TpmCheck {
    fn id(&self) -> &'static str {
        "tpm-status"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        tokio::task::spawn_blocking(tpm_blocking)
            .await
            .map_err(|e| DMedicError::Other(format!("tpm spawn_blocking join: {e}")))?
    }
}

fn tpm_blocking() -> DMedicResult<Vec<Finding>> {
    let com = wmi_helper::init_com_lib();
    let tpm_ns = match WMIConnection::with_namespace_path(
        "ROOT\\CIMV2\\Security\\MicrosoftTpm",
        com,
    ) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(error = %e, "MicrosoftTpm namespace bulunamadı");
            // Namespace yoksa TPM yok varsayalım.
            return Ok(vec![tpm_missing_finding()]);
        }
    };

    let tpms: Vec<Win32Tpm> = tpm_ns.query().unwrap_or_default();
    let Some(tpm) = tpms.into_iter().next() else {
        return Ok(vec![tpm_missing_finding()]);
    };

    let spec = tpm.spec_version.unwrap_or_default();
    let enabled = tpm.is_enabled_initial_value.unwrap_or(false);
    let activated = tpm.is_activated_initial_value.unwrap_or(false);
    // SpecVersion "2.0, 0, 1.59" gibi — ilk versiyon parçasını al.
    let major = spec
        .split(',')
        .next()
        .and_then(|s| s.trim().split('.').next())
        .unwrap_or("");

    if major == "2" && enabled && activated {
        return Ok(Vec::new());
    }

    let (title_tr, title_en, body_tr, body_en) = if major != "2" {
        (
            "TPM 2.0 bulunamadı",
            "TPM 2.0 not available",
            "Sistem TPM 2.0 desteklemiyor (mevcut sürüm: \"".to_string()
                + &spec
                + "\"). Windows 11 güncellemesi, BitLocker ve Windows Hello için TPM 2.0 zorunlu.",
            "System does not report TPM 2.0 (current: \"".to_string()
                + &spec
                + "\"). Required by Windows 11 update, BitLocker and Windows Hello.",
        )
    } else {
        (
            "TPM devre dışı",
            "TPM disabled",
            "TPM 2.0 mevcut ama UEFI/BIOS'tan devre dışı bırakılmış. UEFI ayarlarından \
             etkinleştirin (Security → TPM/PTT/fTPM)."
                .to_string(),
            "TPM 2.0 present but disabled in firmware. Enable it from UEFI \
             (Security → TPM/PTT/fTPM)."
                .to_string(),
        )
    };

    Ok(vec![Finding {
        id: "tpm-status".to_string(),
        category: Category::Compatibility,
        priority: Priority::High,
        action_type: ActionType::Guided,
        title: LocalizedText::new(title_tr, title_en),
        description: LocalizedText::new(body_tr, body_en),
        estimated_gain: EstimatedGain::None,
        risk: RiskLevel::None,
        reboot_required: false,
        action_id: None,
        guide_id: Some("tpm-enable".to_string()),
        evidence: json!({
            "spec_version": spec,
            "enabled": enabled,
            "activated": activated,
        }),
    }])
}

fn tpm_missing_finding() -> Finding {
    Finding {
        id: "tpm-status".to_string(),
        category: Category::Compatibility,
        priority: Priority::High,
        action_type: ActionType::Guided,
        title: LocalizedText::new("TPM bulunamadı", "TPM not detected"),
        description: LocalizedText::new(
            "Sistemde TPM çipi tespit edilemedi. Windows 11 ve BitLocker için TPM 2.0 gerekir.",
            "No TPM chip detected. Windows 11 and BitLocker require TPM 2.0.",
        ),
        estimated_gain: EstimatedGain::None,
        risk: RiskLevel::None,
        reboot_required: false,
        action_id: None,
        guide_id: Some("tpm-enable".to_string()),
        evidence: json!({ "tpm_present": false }),
    }
}

/// #19 — Secure Boot kapalı.
pub struct SecureBootCheck;

#[async_trait]
impl Check for SecureBootCheck {
    fn id(&self) -> &'static str {
        "secure-boot"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        tokio::task::spawn_blocking(secure_boot_blocking)
            .await
            .map_err(|e| DMedicError::Other(format!("secure_boot spawn_blocking join: {e}")))?
    }
}

fn secure_boot_blocking() -> DMedicResult<Vec<Finding>> {
    let enabled = registry::read_dword(
        registry::HKLM,
        r"SYSTEM\CurrentControlSet\Control\SecureBoot\State",
        "UEFISecureBootEnabled",
    );
    if enabled == Some(1) {
        return Ok(Vec::new());
    }

    let (title_tr, title_en, body_tr, body_en) = if enabled.is_none() {
        (
            "Secure Boot bilgisi alınamadı (legacy BIOS?)",
            "Secure Boot state unavailable (legacy BIOS?)",
            "Secure Boot State kayıt anahtarı bulunamadı; sistem büyük olasılıkla \
             Legacy BIOS modunda. UEFI moduna geçiş için MBR→GPT dönüşümü gerekir.",
            "Secure Boot State key not found; system is likely in Legacy BIOS mode. \
             UEFI requires MBR→GPT conversion.",
        )
    } else {
        (
            "Secure Boot kapalı",
            "Secure Boot disabled",
            "Secure Boot UEFI'de etkin değil. Windows 11 uyumluluğu ve rootkit \
             korumasını sağlamak için açmanız önerilir.",
            "Secure Boot is disabled in UEFI. Enable it for Windows 11 compatibility \
             and rootkit protection.",
        )
    };

    Ok(vec![Finding {
        id: "secure-boot".to_string(),
        category: Category::Security,
        priority: Priority::Medium,
        action_type: ActionType::Guided,
        title: LocalizedText::new(title_tr, title_en),
        description: LocalizedText::new(body_tr, body_en),
        estimated_gain: EstimatedGain::None,
        risk: RiskLevel::None,
        reboot_required: false,
        action_id: None,
        guide_id: Some("secure-boot".to_string()),
        evidence: json!({ "uefi_secure_boot_enabled": enabled }),
    }])
}

/// VBS / HVCI durumu — RamVbsConflict zaten kritik durumu yakalıyor, burası bilgi seviyesi.
pub struct VbsHvciCheck;

#[async_trait]
impl Check for VbsHvciCheck {
    fn id(&self) -> &'static str {
        "vbs-hvci"
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // Bilgi seviyesi check — ileride security öneri (RAM yeterli + VBS kapalı → açma öneri)
        // olarak genişletilecek. Şu an no-op.
        Ok(Vec::new())
    }
}
