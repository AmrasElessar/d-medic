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

/// Tek check'in başlangıç/bitiş anını dış dünyaya bildiren event.
/// commands::scan bunu Tauri `emit("scan-progress", ...)`'a çevirir; diagnostic
/// modülü Tauri'den bağımsız kalsın diye saf callback API.
#[derive(Debug)]
pub struct ScanEvent {
    pub index: usize,
    pub total: usize,
    pub check_id: &'static str,
    pub kind: ScanEventKind,
}

#[derive(Debug)]
pub enum ScanEventKind {
    Started,
    Finished { success: bool, finding_count: usize },
}

/// Quick scan: SFC/DISM gibi dakikalar süren işlemler hariç tüm check'leri çalıştır.
pub async fn run_quick() -> DMedicResult<Vec<Finding>> {
    run_with_progress(ScanKind::Quick, |_| {}).await
}

/// Deep scan: SFC + DISM dahil tümünü çalıştır.
pub async fn run_deep() -> DMedicResult<Vec<Finding>> {
    run_with_progress(ScanKind::Deep, |_| {}).await
}

/// Progress callback'li çekirdek tarama. `on_event` her check öncesi (Started)
/// ve sonrası (Finished) çağrılır — UI progress bar / step listesi için.
pub async fn run_with_progress<F>(kind: ScanKind, mut on_event: F) -> DMedicResult<Vec<Finding>>
where
    F: FnMut(ScanEvent) + Send,
{
    let applicable: Vec<Box<dyn Check>> = registry()
        .into_iter()
        .filter(|c| c.applicable_in(kind))
        .collect();
    let total = applicable.len();

    let mut findings = Vec::new();
    for (idx, check) in applicable.iter().enumerate() {
        on_event(ScanEvent {
            index: idx,
            total,
            check_id: check.id(),
            kind: ScanEventKind::Started,
        });
        let (success, count) = match check.run().await {
            Ok(mut items) => {
                let n = items.len();
                findings.append(&mut items);
                (true, n)
            }
            Err(e) => {
                tracing::warn!(check_id = check.id(), error = %e, "Check başarısız");
                (false, 0)
            }
        };
        on_event(ScanEvent {
            index: idx,
            total,
            check_id: check.id(),
            kind: ScanEventKind::Finished { success, finding_count: count },
        });
    }
    Ok(findings)
}
