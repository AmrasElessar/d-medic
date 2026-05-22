//! Kılavuz (guide) yükleyici. JSON dosyaları `resources/guides/` içinden okunur.

pub mod loader;
pub mod schema;

pub use loader::{list_all, load_one};
pub use schema::{Guide, GuideRisk, GuideStep, GuideStepType};
