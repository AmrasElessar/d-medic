use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStateRecord {
    pub name: String,
    pub startup_type: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub restore_point_created: bool,
    pub registry_export_paths: Vec<String>,
    pub service_states: Vec<ServiceStateRecord>,
}
