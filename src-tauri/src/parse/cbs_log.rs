//! CBS.log SIMD tarama. SFC çıktısının ne sonuçlandığını anlamak için
//! `%SystemRoot%\Logs\CBS\CBS.log` dosyasında belirli desenleri arar.

use memchr::memmem;

#[derive(Debug, Clone, Copy, Default)]
pub struct SfcResult {
    /// Bozuk dosya bulundu mu?
    pub corrupt: bool,
    /// Onarım gerçekleşti mi?
    pub repaired: bool,
    /// Onarılamayan dosya var mı?
    pub failed: bool,
}

/// CBS.log buffer'ı içinde son SFC çalışmasının imzalarını ara.
pub fn parse(log: &[u8]) -> SfcResult {
    SfcResult {
        corrupt: memmem::find(log, b"CORRUPT").is_some()
            || memmem::find(log, b"cannot repair").is_some(),
        repaired: memmem::find(log, b"successfully repaired").is_some()
            || memmem::find(log, b"repaired").is_some(),
        failed: memmem::find(log, b"could not").is_some()
            || memmem::find(log, b"unable to repair").is_some(),
    }
}
