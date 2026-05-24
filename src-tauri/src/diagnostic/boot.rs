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

/// #13 — EFI System Partition Microsoft minimum boyutuna uyumsuz.
///
/// Microsoft Learn 'UEFI/GPT-based hard drive partitions' (2026-05-18 güncel):
/// "The EFI system partition must meet the following minimum size requirements
/// for the storage device type: 512 native/512e byte sector size: minimum 200 MB;
/// 4K native sector size: minimum 300 MB."
/// Ref: https://learn.microsoft.com/en-us/windows-hardware/manufacture/desktop/configure-uefigpt-based-hard-drive-partitions?view=windows-11
///
/// Eskiden 100 MB önerilirdi (Win7/8 dönemi); Microsoft Win11 25H2'den itibaren
/// 200 MB minimum talep ediyor — bazı sistemlerde 100 MB ESP'de feature update
/// başarısız oluyor (Microsoft Q&A'larda belgelendi). Bu sebeple D-Medic eşik
/// 200 MB; sektör boyutuna göre 4K detected ise 300 MB.
pub struct EfiPartitionCheck;

#[async_trait]
impl Check for EfiPartitionCheck {
    fn id(&self) -> &'static str {
        "efi-partition"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // Partition boyutu ve sektör boyutu (LogicalSectorSize) — 4K disk ise 4096.
        let script = "try { \
            $p = Get-Partition -ErrorAction Stop | Where-Object { $_.Type -eq 'System' } | Select-Object -First 1; \
            $disk = Get-Disk -Number $p.DiskNumber -ErrorAction Stop; \
            $size_mb = [int]($p.Size / 1MB); \
            $sector = [int]$disk.LogicalSectorSize; \
            \"$size_mb,$sector\" \
        } catch { '' }";
        let out = crate::ps::runner::run_script(script).await.ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };
        let trimmed = out.stdout.trim();
        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() != 2 {
            return Ok(Vec::new());
        }
        let Ok(size_mb) = parts[0].parse::<u32>() else {
            return Ok(Vec::new());
        };
        let sector_bytes = parts[1].parse::<u32>().unwrap_or(512);
        // Microsoft minimum: 200 MB (512-byte sector), 300 MB (4K).
        let ms_minimum = if sector_bytes >= 4096 { 300 } else { 200 };
        if size_mb >= ms_minimum {
            return Ok(Vec::new());
        }

        let priority = if size_mb < 100 {
            Priority::Critical
        } else {
            Priority::High
        };

        Ok(vec![Finding {
            id: "efi-partition".to_string(),
            category: Category::Storage,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                format!("EFI System Partition Microsoft minimumunun altında: {size_mb} MB"),
                format!("EFI System Partition below Microsoft minimum: {size_mb} MB"),
            ),
            description: LocalizedText::new(
                format!(
                    "Microsoft Learn 'UEFI/GPT-based hard drive partitions' \
                     (2026-05-18 güncel): {sector_bytes}-byte sektör için ESP \
                     minimum {ms_minimum} MB olmalıdır. Mevcut: {size_mb} MB. \
                     Win11 25H2 feature update'leri 100 MB ESP'lerde başarısız \
                     olabiliyor (Microsoft Q&A). Çözüm: temiz kurulum (kurulum \
                     standart ESP oluşturur)."
                ),
                format!(
                    "Microsoft Learn 'UEFI/GPT-based hard drive partitions' \
                     (updated 2026-05-18): ESP minimum is {ms_minimum} MB for \
                     {sector_bytes}-byte sector disks. Current: {size_mb} MB. \
                     Win11 25H2 feature updates can fail on 100 MB ESPs \
                     (Microsoft Q&A). Resolution: clean install (setup creates \
                     a standard ESP)."
                ),
            ),
            estimated_gain: EstimatedGain::Stability,
            risk: RiskLevel::Medium,
            reboot_required: false,
            action_id: None,
            guide_id: Some("efi-partition-resize".to_string()),
            evidence: json!({
                "efi_size_mb": size_mb,
                "sector_bytes": sector_bytes,
                "ms_minimum_mb": ms_minimum
            }),
        }])
    }
}
