use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #14 — Windows Update takılı.
pub struct WindowsUpdateStuckCheck;

#[async_trait]
impl Check for WindowsUpdateStuckCheck {
    fn id(&self) -> &'static str { "wu-stuck" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: SoftwareDistribution\Download boyutu + wuauserv durumu
        // + son başarılı update tarihi
        Ok(Vec::new())
    }
}

/// #15 — Pending reboot > 7 gün.
pub struct PendingRebootCheck;

#[async_trait]
impl Check for PendingRebootCheck {
    fn id(&self) -> &'static str { "pending-reboot" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Component Based Servicing\RebootPending key var mı?
        Ok(Vec::new())
    }
}
