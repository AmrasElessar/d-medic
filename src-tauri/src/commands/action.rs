use crate::action::{self, ActionOutcome};
use crate::error::{DMedicError, DMedicResult};

#[tauri::command]
pub async fn apply_action(action_id: String) -> DMedicResult<ActionOutcome> {
    let handler = action::by_id(&action_id)
        .ok_or_else(|| DMedicError::NotFound(format!("action_id: {action_id}")))?;
    tracing::info!(action_id = %action_id, "Action uygulanıyor");
    handler.apply().await
}

#[tauri::command]
pub async fn apply_plan(action_ids: Vec<String>) -> DMedicResult<Vec<ActionOutcome>> {
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
    Ok(outcomes)
}
