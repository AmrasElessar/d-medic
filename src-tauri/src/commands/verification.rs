use std::collections::HashMap;

use crate::error::DMedicResult;
use crate::verification::{self, VerificationRecord};

/// Tek aksiyon/check için doğrulama kaydı. None → henüz araştırılmamış.
#[tauri::command]
pub fn get_verification(id: String) -> DMedicResult<Option<VerificationRecord>> {
    Ok(verification::get_record(&id).cloned())
}

/// Tüm kayıtları döndür — ön bellekleme için frontend tek seferde çeker.
#[tauri::command]
pub fn list_verifications() -> DMedicResult<HashMap<String, VerificationRecord>> {
    Ok(verification::get_all_records().clone())
}
