pub mod action;
pub mod commands;
pub mod diagnostic;
pub mod error;
pub mod guide;
pub mod logging;
pub mod models;
pub mod parse;
pub mod paths;
pub mod profile;
pub mod ps;
pub mod snapshot;

use commands::{action as action_cmd, guide as guide_cmd, profile as profile_cmd, scan, snapshot as snap_cmd, system};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = logging::init() {
        eprintln!("D-Medic logger başlatılamadı: {e}");
    }

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "D-Medic başlatılıyor");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            // system
            system::ping,
            system::app_info,
            system::is_elevated,
            system::system_stats,
            system::reboot_system,
            system::open_logs_folder,
            // scan
            scan::quick_scan,
            scan::deep_scan,
            // action
            action_cmd::apply_action,
            action_cmd::apply_plan,
            // snapshot
            snap_cmd::list_snapshots,
            snap_cmd::create_snapshot,
            snap_cmd::rollback_snapshot,
            snap_cmd::delete_snapshot,
            // guide
            guide_cmd::list_guides,
            guide_cmd::get_guide,
            // profile
            profile_cmd::list_profiles,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri uygulama başlatılamadı");
}
