use async_trait::async_trait;
use serde_json::json;

use super::{registry, Check};
use crate::error::{DMedicError, DMedicResult};
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel,
};

/// #11 — BCD store hatası — bcdedit /enum çıktısında {default} veya {bootmgr} eksik.
pub struct BcdHealthCheck;

#[async_trait]
impl Check for BcdHealthCheck {
    fn id(&self) -> &'static str {
        "bcd-health"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // bcdedit elevated olmayan kullanıcıda kısıtlı çalışır ama enum'a izin verir.
        let out = crate::ps::runner::run_script("bcdedit /enum 2>&1").await.ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };
        let stdout = out.stdout.to_lowercase();

        // Sağlıklı bir Windows BCD'de hem "windows boot manager" hem en az bir
        // "windows boot loader" girdisi olur. Eksikse Critical bulgu.
        let has_bootmgr = stdout.contains("windows boot manager");
        let has_loader = stdout.contains("windows boot loader");
        if has_bootmgr && has_loader {
            return Ok(Vec::new());
        }

        Ok(vec![Finding {
            id: "bcd-health".to_string(),
            category: Category::Stability,
            priority: Priority::Critical,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                "BCD store eksik kayıt içeriyor",
                "BCD store missing entries",
            ),
            description: LocalizedText::new(
                "bcdedit /enum çıktısında Windows Boot Manager veya Boot Loader girdisi \
                 eksik görünüyor. Sistemin sonraki açılışı başarısız olabilir. \
                 `bootrec /rebuildbcd` ile onarım önerilir."
                    .to_string(),
                "bcdedit /enum is missing Windows Boot Manager or Boot Loader entries. \
                 Next boot may fail. Recovery: `bootrec /rebuildbcd`."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::High,
            reboot_required: true,
            action_id: None,
            guide_id: None,
            evidence: json!({
                "has_bootmgr": has_bootmgr,
                "has_loader": has_loader,
            }),
        }])
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

/// #13 — EFI System partition < 300 MB (Windows 11 feature update için dar).
pub struct EfiPartitionCheck;

#[async_trait]
impl Check for EfiPartitionCheck {
    fn id(&self) -> &'static str {
        "efi-partition"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let script = "try { (Get-Partition -ErrorAction Stop | Where-Object { $_.Type -eq 'System' } | Select-Object -First 1 -ExpandProperty Size) / 1MB | ForEach-Object { [int]$_ } } catch { '' }";
        let out = crate::ps::runner::run_script(script).await.ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };
        let Ok(size_mb) = out.stdout.trim().parse::<u32>() else {
            return Ok(Vec::new());
        };
        if size_mb >= 300 {
            return Ok(Vec::new());
        }

        let (priority, label_tr, label_en) = if size_mb < 100 {
            (Priority::Critical, "çok küçük", "very small")
        } else {
            (Priority::High, "küçük", "small")
        };

        Ok(vec![Finding {
            id: "efi-partition".to_string(),
            category: Category::Storage,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                format!("EFI System partition {label_tr}: {size_mb} MB"),
                format!("EFI System partition {label_en}: {size_mb} MB"),
            ),
            description: LocalizedText::new(
                "Win11 feature update'leri EFI/MSR partition'a yeni boot dosyaları yazıyor. \
                 100 MB altı kurulum tamamen başarısız olur; 100-300 MB arası gelecek \
                 güncellemelerde sorun çıkarır. Önerilen: ~500 MB. Genişletme için DiskGenius \
                 veya MiniTool Partition Wizard."
                    .to_string(),
                "Win11 feature updates write new boot files to EFI/MSR. Under 100 MB the \
                 update fails; 100-300 MB causes future issues. Recommended: ~500 MB. Use \
                 DiskGenius or MiniTool Partition Wizard to expand."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::Medium,
            reboot_required: false,
            action_id: None,
            guide_id: Some("efi-partition-resize".to_string()),
            evidence: json!({ "efi_size_mb": size_mb }),
        }])
    }
}
