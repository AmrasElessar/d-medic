use serde::Serialize;

use crate::error::{DMedicError, DMedicResult};

#[derive(Debug, Serialize)]
pub struct AppInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub os: &'static str,
    pub elevated: bool,
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
    })
}

#[tauri::command]
pub fn is_elevated() -> DMedicResult<bool> {
    Ok(check_elevated())
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
