//! Servislerin mevcut StartupType + Status durumunu yakala (rollback için).

use crate::error::DMedicResult;
use crate::models::ServiceStateRecord;
use crate::ps;

pub async fn capture(services: &[&str]) -> DMedicResult<Vec<ServiceStateRecord>> {
    if services.is_empty() {
        return Ok(Vec::new());
    }
    let names = services
        .iter()
        .map(|s| format!("'{}'", s.replace('\'', "''")))
        .collect::<Vec<_>>()
        .join(",");
    let script = format!(
        "Get-Service {} | Select-Object Name, StartType, Status | ConvertTo-Json -Compress",
        names
    );
    let out = ps::runner::run_script(&script).await?;

    // TODO Faz 1: out.stdout'u parse et — şimdilik boş Vec dön.
    let _ = out;
    Ok(Vec::<ServiceStateRecord>::new())
}

pub async fn restore(records: &[ServiceStateRecord]) -> DMedicResult<()> {
    for rec in records {
        let script = format!(
            "Set-Service -Name '{}' -StartupType {} -ErrorAction SilentlyContinue",
            rec.name.replace('\'', "''"),
            rec.startup_type
        );
        ps::runner::run_script(&script).await?;
    }
    Ok(())
}
