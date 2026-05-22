use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #23 — DNS yavaş (> 100 ms).
pub struct DnsSpeedCheck;

#[async_trait]
impl Check for DnsSpeedCheck {
    fn id(&self) -> &'static str { "dns-speed" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Resolve-DnsName ile birkaç hedef sorgula, ms ölç.
        Ok(Vec::new())
    }
}
