use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    Automatic,
    Guided,
    Reboot,
    NotPossible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Performance,
    Stability,
    Security,
    Compatibility,
    Storage,
    Network,
    Power,
    Data,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    None,
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScanKind {
    Quick,
    Deep,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizedText {
    pub tr: String,
    pub en: String,
}

impl LocalizedText {
    pub fn new(tr: impl Into<String>, en: impl Into<String>) -> Self {
        Self {
            tr: tr.into(),
            en: en.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EstimatedGain {
    RamMb { value: u32 },
    BootPct { value: u8 },
    CpuPct { value: u8 },
    DiskMb { value: u32 },
    Stability,
    DataSafety,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: String,
    pub category: Category,
    pub priority: Priority,
    pub action_type: ActionType,
    pub title: LocalizedText,
    pub description: LocalizedText,
    pub estimated_gain: EstimatedGain,
    pub risk: RiskLevel,
    pub reboot_required: bool,
    /// Otomatik düzeltme yapılacaksa hangi action handler çalıştırılacak.
    pub action_id: Option<String>,
    /// Kılavuz gerekiyorsa hangi JSON kılavuz açılacak.
    pub guide_id: Option<String>,
    /// Tespite yol açan ham veri — debug ve UI detay görünümü için.
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub scan_id: String,
    pub kind: ScanKind,
    pub started_at: DateTime<Utc>,
    pub finished_at: DateTime<Utc>,
    pub findings: Vec<Finding>,
}
