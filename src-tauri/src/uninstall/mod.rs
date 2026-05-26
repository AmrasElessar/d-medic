//! Kaldırıcı (Revo tarzı) alt sistemi.
//!
//! Akış: [`inventory`] kurulu programları listeler → [`runner`] programın kendi
//! kaldırıcısını çalıştırır → [`leftover`] kalan registry/dosya izlerini tarar →
//! [`remove`] seçilenleri [`quarantine`] güvenlik ağı üzerinden siler.
//!
//! Tüm yıkıcı işlemler geri alınabilir: dosyalar karantinaya taşınır, registry
//! anahtarları `.reg` export edilir.

pub mod inventory;
pub mod leftover;
pub mod quarantine;
pub mod remove;
pub mod runner;

pub use inventory::{find, list_all};
pub use leftover::scan as scan_leftovers;
pub use remove::remove_items;
pub use runner::run as run_uninstaller;
