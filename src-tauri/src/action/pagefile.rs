use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

/// Pagefile'ı RAM'e bağlı sabit boyuta sabitle (otomatik yönetimden çıkar).
pub struct OptimizePagefile;

#[async_trait]
impl Action for OptimizePagefile {
    fn id(&self) -> &'static str { "pagefile-optimize" }
    fn reboot_required(&self) -> bool { true }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2:
        //   wmic computersystem set AutomaticManagedPagefile=False
        //   wmic pagefileset create name="C:\\pagefile.sys"
        //   wmic pagefileset where name="C:\\pagefile.sys" set InitialSize=4096,MaximumSize=8192
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "Pagefile optimize stub".into(),
            reboot_required: true,
            details: None,
        })
    }
}
