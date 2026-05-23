use async_trait::async_trait;
use serde_json::json;

use super::{registry, Check};
use crate::error::{DMedicError, DMedicResult};
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel,
};

/// #11 — BCD store hatası.
pub struct BcdHealthCheck;

#[async_trait]
impl Check for BcdHealthCheck {
    fn id(&self) -> &'static str {
        "bcd-health"
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 2: bcdedit /enum all — kayıp identifier var mı?
        Ok(Vec::new())
    }
}

/// #12 — Legacy BIOS (Win11 UEFI gerektiriyor).
pub struct LegacyBiosCheck;

#[async_trait]
impl Check for LegacyBiosCheck {
    fn id(&self) -> &'static str {
        "legacy-bios"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        tokio::task::spawn_blocking(legacy_bios_blocking)
            .await
            .map_err(|e| DMedicError::Other(format!("legacy_bios spawn_blocking join: {e}")))?
    }
}

fn legacy_bios_blocking() -> DMedicResult<Vec<Finding>> {
    // PEFirmwareType: 1 = Legacy BIOS, 2 = UEFI. Yoksa Windows boot loader
    // kayıt yapmamış demektir — kararsız sistem, atla.
    let firmware = registry::read_dword(
        registry::HKLM,
        r"SYSTEM\CurrentControlSet\Control",
        "PEFirmwareType",
    );
    match firmware {
        Some(2) | None => return Ok(Vec::new()),
        _ => {}
    }

    Ok(vec![Finding {
        id: "legacy-bios".to_string(),
        category: Category::Compatibility,
        priority: Priority::High,
        action_type: ActionType::Guided,
        title: LocalizedText::new(
            "Legacy BIOS modu (UEFI önerilir)",
            "Legacy BIOS mode (UEFI recommended)",
        ),
        description: LocalizedText::new(
            "Sistem Legacy BIOS modunda çalışıyor. Windows 11, Secure Boot ve TPM 2.0 için \
             UEFI gerekir. Mevcut kuruluma dokunmadan MBR→GPT dönüşümü ve UEFI'ye geçiş \
             yapılabilir (mbr2gpt aracı)."
                .to_string(),
            "System is in Legacy BIOS mode. Windows 11 / Secure Boot / TPM 2.0 require UEFI. \
             MBR→GPT conversion and UEFI switch is possible without reinstall (mbr2gpt)."
                .to_string(),
        ),
        estimated_gain: EstimatedGain::None,
        risk: RiskLevel::Medium,
        reboot_required: true,
        action_id: None,
        guide_id: Some("mbr2gpt".to_string()),
        evidence: json!({ "pe_firmware_type": firmware }),
    }])
}

/// #13 — EFI partition < 100 MB.
pub struct EfiPartitionCheck;

#[async_trait]
impl Check for EfiPartitionCheck {
    fn id(&self) -> &'static str {
        "efi-partition"
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 2: Get-Partition Type=System . Size < 100MB — PS batch ile
        Ok(Vec::new())
    }
}
