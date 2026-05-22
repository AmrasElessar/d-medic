//! Registry export — HKCU\Software + HKLM\Software\Microsoft\Windows.

use crate::error::DMedicResult;
use crate::paths;
use crate::ps;

/// Güvenli (rollback'i mümkün) HKCU / HKLM dallarını export eder.
/// Tüm hive değil; çünkü tam dump GB'larca olabilir ve gereksiz.
pub async fn export_safe_hives(snapshot_id: &str) -> DMedicResult<Vec<String>> {
    let dir = paths::backups_dir().map_err(|e| crate::error::DMedicError::Other(e.to_string()))?;
    std::fs::create_dir_all(&dir)?;
    let hkcu = dir.join(format!("{snapshot_id}_hkcu.reg"));
    let hklm = dir.join(format!("{snapshot_id}_hklm-windows.reg"));

    let script = format!(
        "reg.exe export 'HKCU\\Software' '{}' /y; \
         reg.exe export 'HKLM\\Software\\Microsoft\\Windows' '{}' /y",
        hkcu.display(),
        hklm.display()
    );
    ps::runner::run_script(&script).await?;

    Ok(vec![
        hkcu.to_string_lossy().to_string(),
        hklm.to_string_lossy().to_string(),
    ])
}
