use async_trait::async_trait;

use super::Check;
use crate::error::DMedicResult;
use crate::models::Finding;

/// #17 — Power plan: Dengeli (Balanced) — Ultimate öneriliyor.
pub struct PowerPlanCheck;

#[async_trait]
impl Check for PowerPlanCheck {
    fn id(&self) -> &'static str { "power-plan" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: powercfg /getactivescheme
        Ok(Vec::new())
    }
}

/// #8 — Hibernation açık + disk < 50 GB boş.
pub struct HibernationCheck;

#[async_trait]
impl Check for HibernationCheck {
    fn id(&self) -> &'static str { "hibernation" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: hiberfil.sys var mı, disk boşluğu yeterli mi?
        Ok(Vec::new())
    }
}

/// #7 — Pagefile otomatik + RAM < 6 GB.
pub struct PagefileCheck;

#[async_trait]
impl Check for PagefileCheck {
    fn id(&self) -> &'static str { "pagefile" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: Win32_PageFileSetting; InitialSize/MaximumSize
        Ok(Vec::new())
    }
}

/// #16 — Görsel efektler tam açık + RAM < 6 GB.
pub struct VisualEffectsCheck;

#[async_trait]
impl Check for VisualEffectsCheck {
    fn id(&self) -> &'static str { "visual-effects" }
    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // TODO Faz 1: HKCU\Control Panel\Desktop\WindowMetrics ve VisualFXSetting
        Ok(Vec::new())
    }
}
