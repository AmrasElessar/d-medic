//! Servis durdurma + bekleme döngüsü.
//! Stop-Service async — "Stopping" statüsü saniyeler sürebilir. Dosya kilidi
//! oluşturmamak için tam "Stopped" olana kadar bekle.

use std::time::Duration;

use crate::error::{DMedicError, DMedicResult};

use super::runner;

/// Servisi durdurmaya başla ve gerçekten durana kadar bekle.
pub async fn stop_and_wait(service: &str, timeout: Duration) -> DMedicResult<()> {
    let stop_script = format!(
        "Stop-Service -Name '{}' -Force -ErrorAction SilentlyContinue",
        service
    );
    runner::run_script(&stop_script).await?;

    let check_script = format!("(Get-Service '{}').Status", service);
    let start = std::time::Instant::now();
    loop {
        let out = runner::run_script(&check_script).await?;
        if out.stdout.trim().eq_ignore_ascii_case("Stopped") {
            return Ok(());
        }
        if start.elapsed() > timeout {
            return Err(DMedicError::PowerShell(format!(
                "{} servisi {:?} içinde durmadı",
                service, timeout
            )));
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

/// Birden fazla servis için ardışık bekleme.
pub async fn stop_many(services: &[&str], timeout: Duration) -> DMedicResult<()> {
    for svc in services {
        stop_and_wait(svc, timeout).await?;
    }
    Ok(())
}
