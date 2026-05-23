use serde::Serialize;

use super::snapshot::TRACKED_SERVICES;
use crate::action::{self, ActionOutcome};
use crate::error::{DMedicError, DMedicResult};
use crate::snapshot;

/// Tek action sonucu + öncesinde oluşturulan snapshot id (rollback için).
#[derive(Debug, Serialize)]
pub struct ActionResult {
    pub snapshot_id: Option<String>,
    pub outcome: ActionOutcome,
}

#[derive(Debug, Serialize)]
pub struct PlanResult {
    pub snapshot_id: Option<String>,
    pub outcomes: Vec<ActionOutcome>,
}

/// Snapshot oluşturma fail olursa action'ı yine de çalıştırırız (kullanıcı
/// bilinçli "Düzelt" tıkladı), ama snapshot_id None döner → UI rollback'i
/// gizleyebilir / uyarı gösterebilir.
async fn try_snapshot(label: &str) -> Option<String> {
    match snapshot::capture_full(label, TRACKED_SERVICES).await {
        Ok(snap) => Some(snap.id),
        Err(e) => {
            tracing::warn!(error = %e, label, "Pre-action snapshot başarısız, action snapshot'sız devam edecek");
            None
        }
    }
}

#[tauri::command]
pub async fn apply_action(action_id: String) -> DMedicResult<ActionResult> {
    let handler = action::by_id(&action_id)
        .ok_or_else(|| DMedicError::NotFound(format!("action_id: {action_id}")))?;
    tracing::info!(action_id = %action_id, "Action uygulanıyor");

    let snapshot_id = try_snapshot(&format!("pre-action:{action_id}")).await;
    let outcome = handler.apply().await?;
    Ok(ActionResult {
        snapshot_id,
        outcome,
    })
}

#[tauri::command]
pub async fn apply_plan(action_ids: Vec<String>) -> DMedicResult<PlanResult> {
    if action_ids.is_empty() {
        return Ok(PlanResult {
            snapshot_id: None,
            outcomes: Vec::new(),
        });
    }

    let snapshot_id = try_snapshot(&format!("pre-plan:{}-actions", action_ids.len())).await;

    let mut outcomes = Vec::with_capacity(action_ids.len());
    for id in action_ids {
        match action::by_id(&id) {
            Some(handler) => match handler.apply().await {
                Ok(o) => outcomes.push(o),
                Err(e) => outcomes.push(ActionOutcome {
                    action_id: id,
                    success: false,
                    message: e.to_string(),
                    reboot_required: false,
                    details: None,
                }),
            },
            None => outcomes.push(ActionOutcome {
                action_id: id,
                success: false,
                message: "Bilinmeyen action_id".into(),
                reboot_required: false,
                details: None,
            }),
        }
    }

    Ok(PlanResult {
        snapshot_id,
        outcomes,
    })
}
