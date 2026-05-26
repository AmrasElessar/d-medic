//! Kaldırıcı IPC komutları (frontend `invoke(...)` hedefleri).

use crate::error::DMedicError;
use crate::error::DMedicResult;
use crate::models::{
    InstalledProgram, LeftoverItem, LeftoverScanResult, QuarantineEntry, RemovalReport,
    UninstallReport,
};
use crate::uninstall::{self, quarantine};

/// Kurulu tüm programları (Win32 + UWP) listele.
#[tauri::command]
pub async fn list_installed_programs() -> DMedicResult<Vec<InstalledProgram>> {
    uninstall::list_all().await
}

/// Programın kendi kaldırıcısını çalıştır ve bitmesini bekle. Frontend program
/// objesini gönderir (kaldırma sonrası envanterde bulunamayacağı için id ile
/// re-lookup yapmayız).
#[tauri::command]
pub async fn run_uninstaller(program: InstalledProgram) -> DMedicResult<UninstallReport> {
    uninstall::run_uninstaller(&program).await
}

/// Kaldırma sonrası derin kalıntı taraması (program objesiyle).
#[tauri::command]
pub async fn scan_leftovers(program: InstalledProgram) -> DMedicResult<LeftoverScanResult> {
    uninstall::scan_leftovers(program).await
}

/// Seçili kalıntıları karantina + reg-export üzerinden sil.
#[tauri::command]
pub async fn remove_leftovers(
    program_label: String,
    items: Vec<LeftoverItem>,
) -> DMedicResult<RemovalReport> {
    uninstall::remove_items(program_label, items).await
}

/// Tüm karantina partilerini listele.
#[tauri::command]
pub async fn list_quarantine() -> DMedicResult<Vec<QuarantineEntry>> {
    tokio::task::spawn_blocking(quarantine::list)
        .await
        .map_err(|e| DMedicError::Other(format!("karantina liste join: {e}")))?
}

/// Bir karantina partisini geri yükle (dosyaları + registry'yi eski haline al).
#[tauri::command]
pub async fn restore_quarantine(id: String) -> DMedicResult<()> {
    tokio::task::spawn_blocking(move || quarantine::restore(&id))
        .await
        .map_err(|e| DMedicError::Other(format!("karantina restore join: {e}")))?
}

/// Bir karantina partisini kalıcı sil.
#[tauri::command]
pub async fn purge_quarantine(id: String) -> DMedicResult<()> {
    tokio::task::spawn_blocking(move || quarantine::purge(&id))
        .await
        .map_err(|e| DMedicError::Other(format!("karantina purge join: {e}")))?
}
