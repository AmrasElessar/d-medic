use serde::{Deserialize, Serialize};

/// Bir kaynağın ne tip otoritede olduğu.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    /// learn.microsoft.com / support.microsoft.com
    Microsoft,
    /// CIS Benchmarks (Center for Internet Security)
    Cis,
    /// NIST National Institute of Standards and Technology
    Nist,
    /// NSA hardening guides
    Nsa,
    /// DISA STIGs
    Disa,
    /// Popüler GitHub repository (winutil, Privatezilla, Win11Debloat vb.)
    Github,
    /// Topluluk forumu (r/Windows11, ElevenForum, AskWoody, BleepingComputer)
    Forum,
    /// MVP veya tanınmış kişisel teknik blog
    Mvp,
    /// Diğer (akademik makale, vendor blog vb.)
    Other,
}

/// Bu kaynağın aksiyona yaklaşımı.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceStance {
    /// Aktif olarak önerir
    Recommends,
    /// Belgeler / yapılandırma yolu sunar ama "yap" demez (Microsoft için tipik)
    Documents,
    /// Belirli durumlarda destekler
    Supports,
    /// Sessiz / yorum yok
    Silent,
    /// Önermez ama engellemez
    Discourages,
    /// Aktif olarak engeller / uyarır
    Blocks,
}

/// Tek bir kaynağın kaydı.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSource {
    /// Kaynak türü — UI'da ikon eşlemesi için
    #[serde(rename = "type")]
    pub kind: SourceType,
    /// Doğrudan URL (Microsoft Learn, GitHub repo vb.). None ise reference doldurulur.
    #[serde(default)]
    pub url: Option<String>,
    /// URL olmayan referanslar için (örn. "CIS Win11 Benchmark v3.0.0 §18.10.4.1")
    #[serde(default)]
    pub reference: Option<String>,
    /// GitHub repo için yıldız sayısı (popülerlik metriği)
    #[serde(default)]
    pub stars: Option<u32>,
    /// Bu kaynağın yaklaşımı
    pub stance: SourceStance,
    /// İnsan-okunur not — UI'da gösterilen kısa açıklama
    #[serde(default)]
    pub note: Option<String>,
    /// Belge / kaynak güncelleme tarihi (YYYY-MM-DD) — eskimişlik kontrolü için
    #[serde(default)]
    pub last_verified: Option<String>,
}

/// Genel doğrulama seviyesi — kaynak çoğunluğundan derive edilir.
/// UI rozetinin rengi ve "uygula" butonunun tonu bu değere bağlıdır.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VerificationLevel {
    /// 🟢 3+ bağımsız kaynak (Microsoft + CIS/NIST + topluluk) onaylar; zarar kaydı yok.
    Safe,
    /// 🟡 Microsoft sessiz / belgeli alternatif sunar; 2+ ikincil kaynak onaylar; sınırlı zarar.
    DocumentedAlternative,
    /// 🟠 Sadece topluluk önerir; Microsoft uyarı veya sessiz; geri alınabilir.
    TriedNotOfficial,
    /// 🔴 Tek kaynak veya zarar kaydı kayıtlı; D-Medic önermez.
    NotRecommended,
}

/// Bir aksiyon (action_id) veya check (check_id) için doğrulama kaydı.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationRecord {
    /// Genel seviye — sources özetinden manuel set edilir (otomatik derive zor; her
    /// kaynağın etkisi senaryoya göre farklı olabilir).
    pub verification_level: VerificationLevel,
    /// Bilinen zarar / yan etkiler — Microsoft'tan veya topluluktan derlenmiş.
    /// Boş ise ""→ JSON null. UI'da ⚠ ile gösterilir.
    #[serde(default)]
    pub harm_record: Option<String>,
    /// Kaynak listesi — bağımsız doğrulama için tıklanabilir.
    pub sources: Vec<VerificationSource>,
    /// Kayıt son güncelleme tarihi (D-Medic geliştirici tarafından doğrulanma).
    #[serde(default)]
    pub last_audit_date: Option<String>,
}

/// Tüm verification.json dosyasının kökü.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationDb {
    /// Schema versiyonu (gelecek migrate'ler için).
    pub version: u32,
    /// action_id / check_id → record eşlemesi.
    pub records: std::collections::HashMap<String, VerificationRecord>,
}
