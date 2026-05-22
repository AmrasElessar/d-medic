use serde::{Deserialize, Serialize};

use crate::models::LocalizedText;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuideStepType {
    Cmd,
    Bios,
    Info,
    Manual,
    Link,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GuideRisk {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideVerification {
    pub command: String,
    pub success_pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuideStep {
    pub id: u32,
    #[serde(rename = "type")]
    pub kind: GuideStepType,
    pub title: LocalizedText,
    #[serde(default)]
    pub body: Option<LocalizedText>,
    #[serde(default)]
    pub command: Option<String>,
    #[serde(default)]
    pub guide_link: Option<String>,
    #[serde(default)]
    pub success_message: Option<String>,
    #[serde(default)]
    pub fail_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Guide {
    pub id: String,
    pub title: LocalizedText,
    pub priority: String,
    pub estimated_time: String,
    pub risk: GuideRisk,
    #[serde(default)]
    pub risk_note: Option<LocalizedText>,
    #[serde(default)]
    pub prerequisites: Vec<String>,
    pub steps: Vec<GuideStep>,
    #[serde(default)]
    pub verification: Option<GuideVerification>,
    #[serde(default)]
    pub microsoft_doc: Option<String>,
}
