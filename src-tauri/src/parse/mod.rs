//! SIMD parsing katmanı.
//!
//! - [`cbs_log`] CBS.log içinde memchr ile arama (SFC sonucunu çözümlemek için).
//! - [`event_log`] Windows Event Log XML çıktısının hızlı taraması.
//! - [`json`] PowerShell çıktısı için simd-json wrapper.

pub mod cbs_log;
pub mod event_log;
pub mod json;
