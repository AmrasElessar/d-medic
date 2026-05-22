//! PowerShell yardımcıları.
//!
//! - [`runner`] tek seferlik komut çalıştırır, çıktıyı (UTF-16 LE BOM → UTF-8)
//!   normalize eder.
//! - [`batch`] çoklu sorguları tek script'te birleştirir.
//! - [`wait`] `Stop-Service` sonrası servisin gerçekten durması için bekleme
//!   döngüsü (file lock race condition koruması).

pub mod batch;
pub mod runner;
pub mod wait;
