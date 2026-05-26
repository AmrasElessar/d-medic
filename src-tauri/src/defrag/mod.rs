//! Özel defrag motoru (NTFS, IOCTL tabanlı).
//!
//! Windows `Optimize-Volume`'un veremediği iki şeyi sağlar: **canlı cluster
//! haritası görselleştirmesi** ve **küme-seviyesi kontrol**. Tüm taşımalar
//! `FSCTL_MOVE_FILE` ile yapılır — NTFS bu işlemi journaller, yarıda kalsa bile
//! veri kaybı olmaz.
//!
//! Güvenlik invaryantları:
//!  * SSD'de **asla** full defrag yapılmaz (yalnız analiz). [`volume`] bunu
//!    seek-penalty IOCTL'i ile tespit eder.
//!  * Taşınamaz bölgeler (MFT zone, pagefile, `$BadClus`) atlanır.
//!  * İşlem iptal edilebilir ([`cancel`]).
//!
//! Modüller: [`volume`] (geometri+SSD), [`bitmap`] (cluster bitmap), [`retrieval`]
//! (dosya extent/parça analizi), [`map`] (UI haritası), [`engine`] (taşıma planı+
//! uygulama).

pub mod bitmap;
pub mod engine;
pub mod map;
pub mod retrieval;
pub mod volume;

use std::sync::atomic::{AtomicBool, Ordering};

/// Tek-iş modeli: aynı anda en fazla bir defrag çalışır. İptal bayrağı.
static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

/// Çalışan defrag'i iptal et (bir sonraki güvenli noktada durur).
pub fn request_cancel() {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
}

/// Yeni işe başlarken bayrağı temizle.
pub(crate) fn reset_cancel() {
    CANCEL_FLAG.store(false, Ordering::SeqCst);
}

/// İptal istendi mi (engine döngüsü kontrol eder).
pub(crate) fn is_cancelled() -> bool {
    CANCEL_FLAG.load(Ordering::SeqCst)
}
