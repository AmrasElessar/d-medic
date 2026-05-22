//! Deep scan kapsamındaki yavaş kontroller (5-30 dk).

use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::{Finding, ScanKind};

/// #3 — SFC: bozuk sistem dosyaları.
pub struct SfcCheck;

#[async_trait]
impl Check for SfcCheck {
    fn id(&self) -> &'static str { "sfc-corrupt" }
    fn applicable_in(&self, kind: ScanKind) -> bool {
        matches!(kind, ScanKind::Deep)
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 2: sfc /scannow + CBS.log SIMD parse (parse::cbs_log)
        Ok(Vec::new())
    }
}

/// #4 — DISM: bozuk image.
pub struct DismCheck;

#[async_trait]
impl Check for DismCheck {
    fn id(&self) -> &'static str { "dism-image" }
    fn applicable_in(&self, kind: ScanKind) -> bool {
        matches!(kind, ScanKind::Deep)
    }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 2: DISM /Online /Cleanup-Image /ScanHealth
        Ok(Vec::new())
    }
}
