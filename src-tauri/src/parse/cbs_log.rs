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

/// Desenlerden herhangi biri buffer'da geçiyor mu.
fn any(log: &[u8], patterns: &[&[u8]]) -> bool {
    patterns.iter().any(|p| memmem::find(log, p).is_some())
}

/// CBS.log buffer'ı içinde son SFC çalışmasının imzalarını ara.
///
/// SFC satırları büyük harfle başlar ("[SR] Cannot repair member file …",
/// "[SR] Repairing corrupted file …") — memmem büyük/küçük harf duyarlı olduğu
/// için iki yazım da aranır. "Cannot repair" hem bozulma hem onarılamama demektir.
pub fn parse(log: &[u8]) -> SfcResult {
    let cannot_repair = any(log, &[b"Cannot repair", b"cannot repair"]);
    SfcResult {
        corrupt: cannot_repair || any(log, &[b"CORRUPT", b"Corrupt", b"corrupted"]),
        repaired: any(log, &[b"successfully repaired", b"Repaired file", b"Repairing corrupted"]),
        failed: cannot_repair || any(log, &[b"could not", b"unable to repair", b"Unable to repair"]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clean_log() {
        let r = parse(b"[SR] Verify complete\n[SR] Beginning Verify and Repair transaction");
        assert!(!r.corrupt && !r.repaired && !r.failed);
    }

    #[test]
    fn repaired_log() {
        let r = parse(b"[SR] Repairing corrupted file \\??\\C:\\Windows\\a.dll\n[SR] Repaired file \\SystemRoot\\a.dll");
        assert!(r.corrupt && r.repaired && !r.failed);
    }

    #[test]
    fn cannot_repair_is_corrupt_and_failed() {
        let r = parse(b"[SR] Cannot repair member file [l:24]\"a.dll\" of Microsoft-Windows-X");
        assert!(r.corrupt && r.failed);
    }
}
