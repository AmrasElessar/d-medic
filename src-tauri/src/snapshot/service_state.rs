//! Servislerin mevcut StartupType + Status durumunu yakala (rollback için).

use serde::Deserialize;

use crate::error::{DMedicError, DMedicResult};
use crate::models::ServiceStateRecord;
use crate::ps;

/// Get-Service çıktısının ConvertTo-Json formatı: tek servis → object, çoklu → array.
/// Bu yüzden enum ile her iki şekli de kabul ediyoruz.
#[derive(Deserialize)]
#[serde(untagged)]
enum GetServiceJson {
    Single(GetServiceRaw),
    Multi(Vec<GetServiceRaw>),
}

#[derive(Deserialize)]
struct GetServiceRaw {
    #[serde(alias = "Name")]
    name: String,
    /// PS5.1 → string ("Manual"), PS7+ ConvertTo-Json default → int (3). İkisini de kabul et.
    #[serde(alias = "StartType")]
    start_type: serde_json::Value,
    #[serde(alias = "Status")]
    status: serde_json::Value,
}

fn normalize_start_type(v: &serde_json::Value) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    // ServiceStartMode enum: 2=Automatic, 3=Manual, 4=Disabled, 5=Boot, 6=System.
    match v.as_u64() {
        Some(2) => "Automatic".into(),
        Some(3) => "Manual".into(),
        Some(4) => "Disabled".into(),
        Some(5) => "Boot".into(),
        Some(6) => "System".into(),
        _ => "Manual".into(),
    }
}

fn normalize_status(v: &serde_json::Value) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    // ServiceControllerStatus: 1=Stopped, 4=Running, ...
    match v.as_u64() {
        Some(1) => "Stopped".into(),
        Some(2) => "StartPending".into(),
        Some(3) => "StopPending".into(),
        Some(4) => "Running".into(),
        Some(5) => "ContinuePending".into(),
        Some(6) => "PausePending".into(),
        Some(7) => "Paused".into(),
        _ => "Stopped".into(),
    }
}

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
        "Get-Service -Name {names} -ErrorAction SilentlyContinue | \
         Select-Object Name, StartType, Status | ConvertTo-Json -Compress"
    );
    let out = ps::runner::run_script(&script).await?;
    let trimmed = out.stdout.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let parsed: GetServiceJson = serde_json::from_str(trimmed).map_err(|e| {
        DMedicError::Other(format!("service_state parse: {e} | raw: {trimmed}"))
    })?;
    let raws = match parsed {
        GetServiceJson::Single(s) => vec![s],
        GetServiceJson::Multi(v) => v,
    };
    Ok(raws
        .into_iter()
        .map(|r| ServiceStateRecord {
            name: r.name,
            startup_type: normalize_start_type(&r.start_type),
            status: normalize_status(&r.status),
        })
        .collect())
}

/// Yakalanan kayıtları geri yükle: StartupType + Status (Stop/Start).
pub async fn restore(records: &[ServiceStateRecord]) -> DMedicResult<()> {
    for rec in records {
        // StartupType geri yükle.
        let set_st = format!(
            "Set-Service -Name '{}' -StartupType {} -ErrorAction SilentlyContinue",
            rec.name.replace('\'', "''"),
            rec.startup_type
        );
        let _ = ps::runner::run_script(&set_st).await;

        // Status geri yükle — sadece Running/Stopped'a uygula, diğerleri PS'ten karmaşık.
        match rec.status.as_str() {
            "Running" => {
                let _ = ps::runner::run_script(&format!(
                    "Start-Service -Name '{}' -ErrorAction SilentlyContinue",
                    rec.name.replace('\'', "''")
                ))
                .await;
            }
            "Stopped" => {
                let _ = ps::runner::run_script(&format!(
                    "Stop-Service -Name '{}' -Force -ErrorAction SilentlyContinue",
                    rec.name.replace('\'', "''")
                ))
                .await;
            }
            _ => {}
        }
    }
    Ok(())
}
