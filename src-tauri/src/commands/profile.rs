use crate::error::DMedicResult;
use crate::profile::{self, ProfileDefinition};

#[tauri::command]
pub fn list_profiles() -> DMedicResult<Vec<ProfileDefinition>> {
    Ok(profile::definitions())
}
