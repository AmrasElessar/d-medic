//! Windows System Restore Point oluşturma.
//!
//! Not: System Protection devre dışıysa Checkpoint-Computer no-op olur. Bunu
//! engellemek için `Enable-ComputerRestore` ön adımı eklenmiş.

use crate::error::DMedicResult;
use crate::ps;

pub async fn create(description: &str) -> DMedicResult<bool> {
    // TODO Faz 1: önce System Protection durumunu kontrol et, kapalıysa aç
    let script = format!(
        "Enable-ComputerRestore -Drive 'C:\\' -ErrorAction SilentlyContinue; \
         Checkpoint-Computer -Description '{}' -RestorePointType 'MODIFY_SETTINGS' -ErrorAction Stop; \
         'OK'",
        description.replace('\'', "''")
    );
    let out = ps::runner::run_script(&script).await?;
    Ok(out.stdout.contains("OK"))
}
