//! Kaldırıcı (Revo tarzı) veri modelleri — frontend ile paylaşılan sözleşme.
//!
//! Akış: [`InstalledProgram`] listele → kullanıcı seçer → programın kendi
//! kaldırıcısı çalışır ([`UninstallReport`]) → ardından derin kalıntı taraması
//! ([`LeftoverScanResult`]) → kullanıcı [`LeftoverItem`]'leri seçer → karantina +
//! reg-export ile silinir ([`RemovalReport`]).

use serde::{Deserialize, Serialize};

/// Programın kaynağı — kaldırma stratejisini belirler.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgramKind {
    /// Klasik masaüstü uygulaması (registry Uninstall anahtarı + UninstallString).
    Win32,
    /// UWP / MSIX paketi (Get-AppxPackage / Remove-AppxPackage).
    Uwp,
}

/// Sistemde kurulu tek bir program.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledProgram {
    /// Kararlı kimlik. Win32 için registry alt-anahtar adı (örn. GUID veya
    /// app-id), UWP için PackageFullName.
    pub id: String,
    pub name: String,
    pub publisher: Option<String>,
    pub version: Option<String>,
    pub kind: ProgramKind,
    /// Kurulum kök klasörü (kalıntı taramasında kök ipucu olarak kullanılır).
    pub install_location: Option<String>,
    /// Win32: registry'deki UninstallString (komut satırı).
    pub uninstall_string: Option<String>,
    /// Win32: sessiz kaldırma komutu (varsa) — tercih edilir.
    pub quiet_uninstall_string: Option<String>,
    /// Tahmini boyut (byte). Registry EstimatedSize KB cinsindendir, byte'a çevrilir.
    pub size_bytes: Option<u64>,
    /// YYYYMMDD ham string (registry InstallDate) — UI tarihi parse eder.
    pub install_date: Option<String>,
    /// base64 PNG/ICO yoksa None; UI varsayılan ikon gösterir.
    pub icon_base64: Option<String>,
    /// Sistem bileşeni / D-Medic'in dokunmaması gereken giriş mi (UI uyarısı).
    pub is_system_component: bool,
}

/// Programın kendi kaldırıcısını çalıştırma sonucu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallReport {
    pub program_id: String,
    /// Kaldırıcı süreç başarılı exit kodu döndü mü.
    pub launched: bool,
    /// Kaldırıcı süreci tamamlandı mı (bazı kaldırıcılar fork edip hemen döner).
    pub completed: bool,
    pub exit_code: Option<i32>,
    pub message: String,
}

/// Kalıntı türü.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeftoverKind {
    File,
    Folder,
    /// Registry anahtarı (alt-ağaç dahil silinir).
    RegKey,
    /// Registry değeri (tek value).
    RegValue,
}

/// Kalıntının "gerçekten bu programa ait" olma güveni. Revo'nun
/// güvenli/orta/agresif renklendirmesinin karşılığı.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeftoverConfidence {
    /// Kurulum klasörünün altında / tam ad eşleşmesi — silmek güvenli.
    Safe,
    /// Yayıncı klasörü veya güçlü ad eşleşmesi — büyük olasılıkla ait.
    Probable,
    /// Zayıf eşleşme (kısmi ad) — kullanıcı dikkatle incelemeli.
    Possible,
}

/// Tek bir kalıntı öğesi.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeftoverItem {
    /// Kararlı kimlik (yol/registry path'inin blake3 hash'i) — UI seçim takibi.
    pub id: String,
    pub kind: LeftoverKind,
    pub confidence: LeftoverConfidence,
    /// Dosya/klasör için tam yol; registry için "HKCU\\Software\\...".
    pub path: String,
    /// RegValue için value adı (RegKey/File/Folder için None).
    pub value_name: Option<String>,
    /// Dosya/klasör boyutu (byte). Registry için None.
    pub size_bytes: Option<u64>,
    /// Bu öğenin neden kalıntı sayıldığına dair kısa gerekçe (UI ipucu).
    pub reason: String,
    /// Varsayılan olarak işaretli gelsin mi (Safe + Probable → true).
    pub default_selected: bool,
}

/// Kaldırma sonrası derin tarama sonucu.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LeftoverScanResult {
    pub program_id: String,
    pub items: Vec<LeftoverItem>,
    /// Taranan kök sayısı / atlanan vb. — UI özet/teşhis.
    pub scanned_roots: Vec<String>,
    pub registry_hits: usize,
    pub file_hits: usize,
}

/// Tek bir kalıntının silinme sonucu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovalItemResult {
    pub item_id: String,
    pub path: String,
    pub success: bool,
    pub message: Option<String>,
}

/// Seçili kalıntıların toplu silinme raporu.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemovalReport {
    /// Bu silme işlemine ait karantina partisi (geri yükleme için).
    pub quarantine_id: String,
    pub removed: usize,
    pub failed: usize,
    pub freed_bytes: u64,
    pub results: Vec<RemovalItemResult>,
}

/// Karantinaya alınmış bir silme partisi (Geçmiş/geri-yükleme paneli için).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuarantineEntry {
    pub id: String,
    /// RFC3339 oluşturma zamanı.
    pub created_at: String,
    /// Hangi program için yapıldı (etiket).
    pub program_label: String,
    pub file_count: usize,
    pub reg_export_count: usize,
    pub total_bytes: u64,
    /// Otomatik temizlenmeden önce kalan gün (UI bilgisi).
    pub expires_in_days: i64,
}
