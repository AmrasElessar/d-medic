use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::diagnostic::Finding;
use super::profile::ProfileKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanItemStatus {
    Pending,
    Running,
    Success,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub finding_id: String,
    pub action_id: String,
    pub status: PlanItemStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub plan_id: String,
    pub profile: ProfileKind,
    pub selected_findings: Vec<Finding>,
    pub items: Vec<PlanItem>,
    pub snapshot_id: Option<String>,
    pub reboot_required: bool,
}
