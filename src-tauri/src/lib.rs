pub mod action;
pub mod commands;
pub mod defrag;
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
pub mod uninstall;
pub mod verification;

use commands::{
    action as action_cmd, defrag as defrag_cmd, guide as guide_cmd, profile as profile_cmd, scan,
    snapshot as snap_cmd, system, uninstall as uninstall_cmd, verification as verification_cmd,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    if let Err(e) = logging::init() {
        eprintln!("D-Medic logger başlatılamadı: {e}");
    }

    tracing::info!(version = env!("CARGO_PKG_VERSION"), "D-Medic başlatılıyor");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        // tauri-plugin-log `log` crate facade'ını yakalar (wmi gibi dep'ler).
        // Varsayılanı Info'ya çekip gürültülü dep'leri Warn'a kısıyoruz —
        // dev konsolu WMI TRACE/DEBUG seli yerine işe yarar logla dolsun.
        // (Kendi kodumuz `tracing` kullanıyor; o ayrı console layer'dan akar.)
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .level_for("wmi", log::LevelFilter::Warn)
                .level_for("tao", log::LevelFilter::Warn)
                .level_for("wry", log::LevelFilter::Warn)
                .build(),
        )
        .invoke_handler(tauri::generate_handler![
            // system
            system::ping,
            system::app_info,
            system::is_elevated,
            system::system_stats,
            system::list_disks,
            system::reboot_system,
            system::relaunch_as_admin,
            system::open_logs_folder,
            system::dev_log,
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
            // verification
            verification_cmd::get_verification,
            verification_cmd::list_verifications,
            // uninstall (Revo tarzı kaldırıcı)
            uninstall_cmd::list_installed_programs,
            uninstall_cmd::run_uninstaller,
            uninstall_cmd::scan_leftovers,
            uninstall_cmd::remove_leftovers,
            uninstall_cmd::list_quarantine,
            uninstall_cmd::restore_quarantine,
            uninstall_cmd::purge_quarantine,
            // defrag (özel IOCTL motoru)
            defrag_cmd::list_volumes,
            defrag_cmd::analyze_volume,
            defrag_cmd::get_cluster_map,
            defrag_cmd::start_defrag,
            defrag_cmd::cancel_defrag,
        ])
        .run(tauri::generate_context!())
        .expect("Tauri uygulama başlatılamadı");
}
