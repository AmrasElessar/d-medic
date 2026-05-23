//! PowerShell batch dispatch — quick scan'in tüm PS sorgularını tek bir
//! script'te birleştirip JSON olarak dönen yardımcı.
//!
//! Hedef: 28 ayrı spawn yerine tek spawn. Yanıt UTF-16 LE BOM ile dönerse
//! UTF-8'e çevrildikten sonra simd-json parse'a verilir.

use serde::Deserialize;

use crate::error::{DMedicError, DMedicResult};
use crate::ps;

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
///
/// simd-json mutable buffer ister — string'i `into_bytes()` ile owned `Vec<u8>`
/// olarak veriyoruz. UTF-16 BOM normalize işi `ps::runner::run_script`
/// içinde yapıldığı için burada saf UTF-8 bekliyoruz.
pub async fn run_quick_batch() -> DMedicResult<QuickBatch> {
    let raw = ps::batch::run_quick_scan_script().await?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(QuickBatch::default());
    }

    let mut buf = trimmed.to_owned().into_bytes();
    let parsed: QuickBatch = simd_json::serde::from_slice(&mut buf).map_err(|e| {
        // Tanı için ilk 200 karakteri logla — büyük JSON akışında debug pratik.
        let preview: String = trimmed.chars().take(200).collect();
        tracing::warn!(error = %e, preview = %preview, "QuickBatch parse failed");
        DMedicError::SimdJson(e)
    })?;
    Ok(parsed)
}
