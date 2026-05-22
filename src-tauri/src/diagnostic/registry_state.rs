use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #20 — Windows Recovery Environment devre dışı.
pub struct WindowsReCheck;

#[async_trait]
impl Check for WindowsReCheck {
    fn id(&self) -> &'static str { "windows-re" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: reagentc /info — Windows RE Status: Disabled
        Ok(Vec::new())
    }
}

/// #24 — CPU Win11 resmi destek listesinde yok.
pub struct CpuCompatibilityCheck;

#[async_trait]
impl Check for CpuCompatibilityCheck {
    fn id(&self) -> &'static str { "cpu-compat" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Win32_Processor.Name'i Microsoft destekli listede ara
        Ok(Vec::new())
    }
}

/// #25 — Aktivasyon hatası.
pub struct ActivationCheck;

#[async_trait]
impl Check for ActivationCheck {
    fn id(&self) -> &'static str { "activation" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: cscript slmgr.vbs /xpr  veya  Get-CimInstance SoftwareLicensingProduct
        Ok(Vec::new())
    }
}

/// #26 — Driver güncel değil (kritik).
pub struct DriverFreshnessCheck;

#[async_trait]
impl Check for DriverFreshnessCheck {
    fn id(&self) -> &'static str { "driver-freshness" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 2: pnputil /enum-drivers — son driver tarihleri
        Ok(Vec::new())
    }
}
