use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #9 — Startup > 8 uygulama.
pub struct StartupCountCheck;

#[async_trait]
impl Check for StartupCountCheck {
    fn id(&self) -> &'static str { "startup-count" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Get-CimInstance Win32_StartupCommand
        //  + HKCU\Software\Microsoft\Windows\CurrentVersion\Run
        //  + HKLM\Software\Microsoft\Windows\CurrentVersion\Run
        Ok(Vec::new())
    }
}
