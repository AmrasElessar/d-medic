use tauri::{AppHandle, Manager};

use crate::error::{DMedicError, DMedicResult};
use crate::guide::{self, schema::Guide};

#[tauri::command]
pub async fn list_guides(app: AppHandle) -> DMedicResult<Vec<Guide>> {
    let dir = app
        .path()
        .resource_dir()
        .map_err(|e| DMedicError::Other(e.to_string()))?;
    guide::list_all(dir).await
}

#[tauri::command]
pub async fn get_guide(app: AppHandle, id: String) -> DMedicResult<Guide> {
    let dir = app
        .path()
        .resource_dir()
        .map_err(|e| DMedicError::Other(e.to_string()))?;
    guide::load_one(dir, &id).await
}
