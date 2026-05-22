use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// SysMain / WSearch / DiagTrack vb. arka plan servisleri.
pub struct BloatServicesCheck;

#[async_trait]
impl Check for BloatServicesCheck {
    fn id(&self) -> &'static str { "bloat-services" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Get-Service SysMain,WSearch,DiagTrack | Where Status=Running
        Ok(Vec::new())
    }
}

/// #10 — Telemetry servisleri çalışıyor.
pub struct TelemetryCheck;

#[async_trait]
impl Check for TelemetryCheck {
    fn id(&self) -> &'static str { "telemetry-services" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: DiagTrack, dmwappushservice çalışıyor mu?
        Ok(Vec::new())
    }
}
