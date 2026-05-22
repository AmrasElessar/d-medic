use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #21 — Son 7 günde BSOD tespit edildi.
pub struct BsodHistoryCheck;

#[async_trait]
impl Check for BsodHistoryCheck {
    fn id(&self) -> &'static str { "bsod-history" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Get-WinEvent System Log; Event ID 41 (Kernel-Power)
        // + minidump klasörü taraması
        Ok(Vec::new())
    }
}

/// #28 — Event Log > 50 kritik hata/hafta.
pub struct EventLogCriticalCheck;

#[async_trait]
impl Check for EventLogCriticalCheck {
    fn id(&self) -> &'static str { "event-log-critical" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Get-WinEvent -FilterHashtable Level=1 — son 7 gün
        Ok(Vec::new())
    }
}
