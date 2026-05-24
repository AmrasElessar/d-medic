use serde::Serialize;

use crate::diagnostic::wmi::{read_snapshot, SystemSnapshot};
use crate::error::{DMedicError, DMedicResult};
use crate::paths;

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub os: &'static str,
    pub elevated: bool,
    pub git_rev: &'static str,
    pub build_date: &'static str,
}

#[tauri::command]
pub fn ping() -> DMedicResult<String> {
    tracing::debug!("ping çağrıldı");
    Ok(format!("pong (D-Medic v{})", env!("CARGO_PKG_VERSION")))
}

#[tauri::command]
pub fn app_info() -> DMedicResult<AppInfo> {
    Ok(AppInfo {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
        os: std::env::consts::OS,
        elevated: check_elevated(),
        git_rev: env!("D_MEDIC_GIT_REV"),
        build_date: env!("D_MEDIC_BUILD_DATE"),
    })
}

#[tauri::command]
pub fn is_elevated() -> DMedicResult<bool> {
    Ok(check_elevated())
}

/// Sistem snapshot'ı — RAM/CPU/disk/VBS. Dashboard StatPill'ler ve ileride
/// "sistem durumu" görünümleri buradan beslenir.
#[tauri::command]
pub async fn system_stats() -> DMedicResult<SystemSnapshot> {
    read_snapshot().await
}

/// %APPDATA%\D-Medic\logs klasörünü Windows Explorer'da açar. Klasör yoksa
/// oluştur, sonra explorer.exe spawn et. tauri-plugin-shell'in `open()` API'si
/// de kullanılabilirdi ama spawning daha açık ve test edilebilir.
#[tauri::command]
pub fn open_logs_folder() -> DMedicResult<()> {
    let dir = paths::log_dir().map_err(|e| DMedicError::Other(e.to_string()))?;
    std::fs::create_dir_all(&dir)?;
    std::process::Command::new("explorer.exe")
        .arg(&dir)
        .spawn()
        .map_err(|e| DMedicError::Other(format!("explorer.exe spawn: {e}")))?;
    Ok(())
}

/// Sistemi yeniden başlatır. shutdown.exe /r /t <seconds> spawn eder; bu sayede
/// kullanıcı UI'ı kapatıp açık çalışmalarını kaydedebilir. /a ile iptal edilebilir
/// (gelecekte "Yeniden başlatmayı iptal et" butonu için).
#[tauri::command]
pub fn reboot_system(delay_seconds: Option<u32>) -> DMedicResult<()> {
    let delay = delay_seconds.unwrap_or(30).to_string();
    let output = std::process::Command::new("shutdown.exe")
        .args(["/r", "/t", &delay, "/c", "D-Medic değişiklikleri için yeniden başlatılıyor"])
        .output()
        .map_err(|e| DMedicError::Other(format!("shutdown.exe spawn: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(DMedicError::Other(format!(
            "shutdown.exe başarısız (exit {}): {stderr}",
            output.status
        )));
    }
    Ok(())
}

#[cfg(windows)]
fn check_elevated() -> bool {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY};
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    unsafe {
        let mut token: HANDLE = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return false;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut size: u32 = 0;
        let ok = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut _ as *mut _),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut size,
        )
        .is_ok();
        let _ = CloseHandle(token);
        ok && elevation.TokenIsElevated != 0
    }
}

#[cfg(not(windows))]
fn check_elevated() -> bool {
    // D-Medic Windows özel — diğer platformlarda her zaman false.
    false
}

#[allow(dead_code)]
fn _ensure_elevated() -> DMedicResult<()> {
    if check_elevated() {
        Ok(())
    } else {
        Err(DMedicError::NotElevated)
    }
}
