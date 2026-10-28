//! Kaldırıcı IPC komutları (frontend `invoke(...)` hedefleri).

use crate::error::DMedicError;
use crate::error::DMedicResult;
use crate::models::{
    InstalledProgram, LeftoverScanResult, QuarantineEntry, RemovalReport,
    UninstallReport,
};
use crate::uninstall::{self, quarantine};

/// Kurulu tüm programları (Win32 + UWP) listele.
#[tauri::command]
pub async fn list_installed_programs() -> DMedicResult<Vec<InstalledProgram>> {
    uninstall::list_all().await
}

/// Programın kendi kaldırıcısını çalıştır ve bitmesini bekle. Frontend yalnız
/// id gönderir; çalıştırılacak komut satırı (UninstallString) her zaman
/// registry'den yeniden okunur — webview'dan gelen bir dize yönetici yetkisiyle
/// `cmd /c`'ye verilmez.
#[tauri::command]
pub async fn run_uninstaller(program_id: String) -> DMedicResult<UninstallReport> {
    let program = uninstall::find(&program_id)
        .await?
        .ok_or_else(|| DMedicError::NotFound(format!("program: {program_id}")))?;
    uninstall::run_uninstaller(&program).await
}

/// Kaldırma sonrası derin kalıntı taraması (program objesiyle — kaldırma
/// sonrası envanterde bulunamayacağı için id ile re-lookup yapılmaz). Tarama
/// salt-okunurdur; silinecek her yol `remove_leftovers`'ta ayrıca doğrulanır.
#[tauri::command]
pub async fn scan_leftovers(program: InstalledProgram) -> DMedicResult<LeftoverScanResult> {
    uninstall::scan_leftovers(program).await
}

/// Seçili kalıntıları karantina + reg-export üzerinden sil. Yalnız son
/// taramadaki öğe id'leri kabul edilir.
#[tauri::command]
pub async fn remove_leftovers(
    program_label: String,
    item_ids: Vec<String>,
) -> DMedicResult<RemovalReport> {
    uninstall::remove_items(program_label, item_ids).await
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

/// Tüm karantina partilerini kalıcı sil; silinen parti sayısını döner.
#[tauri::command]
pub async fn purge_all_quarantine() -> DMedicResult<usize> {
    tokio::task::spawn_blocking(quarantine::purge_all)
        .await
        .map_err(|e| DMedicError::Other(format!("karantina purge-all join: {e}")))?
}

/// Bir karantina partisini kalıcı sil.
#[tauri::command]
pub async fn purge_quarantine(id: String) -> DMedicResult<()> {
    tokio::task::spawn_blocking(move || quarantine::purge(&id))
        .await
        .map_err(|e| DMedicError::Other(format!("karantina purge join: {e}")))?
}
