//! Düzeltici işlemler (action) — her biri bir [`Action`] implementasyonudur.
//!
//! Plan yürütücüsü `registry()`'den ilgili `action_id` ile arar ve `apply()` çağırır.
//! Çalıştırmadan önce snapshot oluşturulur, hata durumunda rollback önerilir.

use async_trait::async_trait;
use serde::Serialize;

use crate::error::DMedicResult;

pub mod bcd;
pub mod debloat;
pub mod defrag;
pub mod dism;
pub mod dns;
pub mod hibernation;
pub mod pagefile;
pub mod power_plan;
pub mod recovery;
pub mod services_off;
pub mod sfc;
pub mod startup_clean;
pub mod temp_clean;
pub mod vbs;
pub mod visual_effects;
pub mod wu_reset;

#[derive(Debug, Clone, Serialize)]
pub struct ActionOutcome {
    pub action_id: String,
    pub success: bool,
    pub message: String,
    pub reboot_required: bool,
    pub details: Option<serde_json::Value>,
}

#[async_trait]
pub trait Action: Send + Sync {
    fn id(&self) -> &'static str;
    fn reboot_required(&self) -> bool {
        false
    }
    async fn apply(&self) -> DMedicResult<ActionOutcome>;
}

pub fn registry() -> Vec<Box<dyn Action>> {
    vec![
        Box::new(sfc::SfcRepair),
        Box::new(dism::DismRestoreHealth),
        Box::new(bcd::BcdRebuild),
        Box::new(wu_reset::WindowsUpdateReset),
        Box::new(services_off::DisableSysmain),
        Box::new(services_off::DisableSearchIndex),
        Box::new(services_off::DisableTelemetry),
        Box::new(debloat::RemoveBloatware),
        Box::new(startup_clean::StartupCleanup),
        Box::new(visual_effects::MinimalVisualEffects),
        Box::new(power_plan::UltimatePerformance),
        Box::new(pagefile::OptimizePagefile),
        Box::new(hibernation::DisableHibernation),
        Box::new(dns::SwitchDns),
        Box::new(recovery::EnableWindowsRe),
        Box::new(vbs::DisableVbs),
        Box::new(temp_clean::CleanTemp),
        Box::new(defrag::DefragHdd),
    ]
}

pub fn by_id(action_id: &str) -> Option<Box<dyn Action>> {
    registry().into_iter().find(|a| a.id() == action_id)
}
