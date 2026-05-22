use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #18 — TPM 2.0 yok veya devre dışı.
pub struct TpmCheck;

#[async_trait]
impl Check for TpmCheck {
    fn id(&self) -> &'static str { "tpm-status" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Get-Tpm — TpmPresent, TpmReady, SpecVersion
        Ok(Vec::new())
    }
}

/// #19 — Secure Boot devre dışı.
pub struct SecureBootCheck;

#[async_trait]
impl Check for SecureBootCheck {
    fn id(&self) -> &'static str { "secure-boot" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Confirm-SecureBootUEFI
        Ok(Vec::new())
    }
}

/// VBS / HVCI durumu (RAM düşükse kapatılmalı).
pub struct VbsHvciCheck;

#[async_trait]
impl Check for VbsHvciCheck {
    fn id(&self) -> &'static str { "vbs-hvci" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: HKLM\SYSTEM\...\DeviceGuard\EnableVirtualizationBasedSecurity
        // + HypervisorEnforcedCodeIntegrity
        Ok(Vec::new())
    }
}
