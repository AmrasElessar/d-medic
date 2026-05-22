//! PowerShell batch dispatch — quick scan'in tüm PS sorgularını tek bir
//! script'te birleştirip JSON olarak dönen yardımcı.
//!
//! Hedef: 28 ayrı spawn yerine tek spawn. Yanıt UTF-16 LE BOM ile dönerse
//! UTF-8'e çevrildikten sonra simd-json parse'a verilir.

use serde::Deserialize;

use crate::error::DMedicResult;

/// Quick scan PS batch çıktısının tek bir typed temsili.
/// Yeni alan eklediğinde hem PS hem buraya ekle.
#[derive(Debug, Deserialize, Default)]
pub struct QuickBatch {
    pub ram_gb: Option<f32>,
    pub disk_type: Option<String>,
    pub disk_free_gb: Option<f32>,
    pub vbs_enabled: Option<u32>,
    pub boot_mode: Option<u32>,
    pub pending_reboot: Option<bool>,
    pub services: Option<Vec<ServiceEntry>>,
    pub uwp_apps: Option<Vec<String>>,
    pub startup_count: Option<u32>,
    pub tpm_version: Option<String>,
    pub secure_boot: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct ServiceEntry {
    pub name: String,
    pub status: String,
    pub startup_type: Option<String>,
}

/// Tek batch çalıştır, JSON parse et.
pub async fn run_quick_batch() -> DMedicResult<QuickBatch> {
    // TODO Faz 1: crate::ps::batch::run_quick_scan_script() çağrısı,
    // çıktıyı UTF-8'e normalize edip simd_json::from_slice ile parse et.
    Ok(QuickBatch::default())
}
