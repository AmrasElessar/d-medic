//! Tanılama motoru.
//!
//! Her tanılama kalemi bir [`Check`] implementasyonudur. Modül yüklenirken
//! `registry()` çağrısı tüm aktif check'lerin listesini döner; quick scan
//! sadece `ScanKind::Quick`'i destekleyenleri, deep scan ise tümünü çalıştırır.

use async_trait::async_trait;

use crate::error::DMedicResult;
use crate::models::{Finding, ScanKind};

pub mod batch;
pub mod bloatware;
pub mod boot;
pub mod events;
pub mod network;
pub mod power;
pub mod ram;
pub mod registry;
pub mod registry_state;
pub mod security;
pub mod services;
pub mod sfc_dism;
pub mod startup;
pub mod storage;
pub mod update;
pub mod wmi;

/// Bir tanılama kontrolü. Run sonucunda 0 veya daha fazla [`Finding`] döner.
/// Birden fazla finding mantıklı olabilir (örn. birden fazla SMART uyarısı).
#[async_trait]
pub trait Check: Send + Sync {
    /// Kararlı, kebab-case kimlik — log/UI/profil eşlemesi için.
    fn id(&self) -> &'static str;

    /// Hangi tarama türlerinde çalışmalı.
    fn applicable_in(&self, kind: ScanKind) -> bool {
        matches!(kind, ScanKind::Quick | ScanKind::Deep)
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>>;
}

/// Aktif kontrol listesi. Yeni bir tanılama eklediğinde buraya kayıt yap.
pub fn registry() -> Vec<Box<dyn Check>> {
    vec![
        Box::new(ram::RamVbsConflictCheck),
        Box::new(storage::SysmainHddCheck),
        Box::new(storage::SmartHealthCheck),
        Box::new(storage::DiskFullCheck),
        Box::new(storage::FragmentationCheck),
        Box::new(services::BloatServicesCheck),
        Box::new(services::TelemetryCheck),
        Box::new(bloatware::UwpBloatwareCheck),
        Box::new(startup::StartupCountCheck),
        Box::new(boot::BcdHealthCheck),
        Box::new(boot::LegacyBiosCheck),
        Box::new(boot::EfiPartitionCheck),
        Box::new(update::WindowsUpdateStuckCheck),
        Box::new(update::PendingRebootCheck),
        Box::new(security::TpmCheck),
        Box::new(security::SecureBootCheck),
        Box::new(security::VbsHvciCheck),
        Box::new(power::PowerPlanCheck),
        Box::new(power::HibernationCheck),
        Box::new(power::PagefileCheck),
        Box::new(power::VisualEffectsCheck),
        Box::new(network::DnsSpeedCheck),
        Box::new(events::BsodHistoryCheck),
        Box::new(events::EventLogCriticalCheck),
        Box::new(registry_state::WindowsReCheck),
        Box::new(registry_state::CpuCompatibilityCheck),
        Box::new(registry_state::ActivationCheck),
        Box::new(registry_state::DriverFreshnessCheck),
        Box::new(sfc_dism::SfcCheck),
        Box::new(sfc_dism::DismCheck),
    ]
}

/// Quick scan: SFC/DISM gibi dakikalar süren işlemler hariç tüm check'leri çalıştır.
pub async fn run_quick() -> DMedicResult<Vec<Finding>> {
    run_filtered(ScanKind::Quick).await
}

/// Deep scan: SFC + DISM dahil tümünü çalıştır.
pub async fn run_deep() -> DMedicResult<Vec<Finding>> {
    run_filtered(ScanKind::Deep).await
}

async fn run_filtered(kind: ScanKind) -> DMedicResult<Vec<Finding>> {
    let mut findings = Vec::new();
    for check in registry() {
        if !check.applicable_in(kind) {
            continue;
        }
        match check.run().await {
            Ok(mut items) => findings.append(&mut items),
            Err(e) => {
                tracing::warn!(check_id = check.id(), error = %e, "Check başarısız");
            }
        }
    }
    Ok(findings)
}
