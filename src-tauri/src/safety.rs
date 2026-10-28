//! Yıkıcı IPC girdileri için ortak güvenlik kontrolleri.
//!
//! D-Medic yönetici yetkisiyle çalışır; frontend'den gelen her yol, registry
//! anahtarı ve depolama id'si burada doğrulanmadan silme/taşıma yapılmaz.
//! Modül yalnızca `std`'ye bağlıdır ve yol karşılaştırmaları dize tabanlıdır —
//! böylece Windows dışında da birim testleri koşar.

/// Karantina partisi / snapshot id'si: yalnız ASCII alfanümerik, `-`, `_`.
/// `..`, ayraç ve sürücü harfi içeren her şey reddedilir (path traversal).
pub fn is_valid_storage_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Bir dizeyi karşılaştırma anahtarına indirger: küçük harf + yalnız alfanümerik.
pub fn norm_key(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// Asla "kalıntı" sayılmaması gereken klasör / registry anahtarı adları
/// ([`norm_key`] biçiminde). Bunlar birden çok ürünün ya da Windows'un ortak
/// alanlarıdır; ad eşleşmesi ne kadar güçlü olursa olsun silinmezler.
const PROTECTED_NAMES: &[&str] = &[
    // Windows / Microsoft ortak alanları
    "microsoft",
    "windows",
    "windowsapps",
    "modifiablewindowsapps",
    "windowsdefender",
    "windowsdefenderadvancedthreatprotection",
    "windowsnt",
    "windowsmail",
    "windowsmediaplayer",
    "windowsphotoviewer",
    "windowsportabledevices",
    "windowssecurity",
    "windowssidebar",
    "windowspowershell",
    "internetexplorer",
    "microsoftnet",
    "referenceassemblies",
    "commonfiles",
    // Common Files altı ortak klasörler
    "microsoftshared",
    "system",
    "services",
    "speechengines",
    "packages",
    "packagecache",
    "programs",
    "temp",
    // Registry SOFTWARE altı ortak dallar
    "classes",
    "policies",
    "wow6432node",
    "clients",
    "registeredapplications",
    // Başlat menüsü sistem klasörleri
    "startup",
    "accessibility",
    "accessories",
    "administrativetools",
    "maintenance",
    "systemtools",
    "windowsaccessories",
    "windowsadministrativetools",
    "windowseaseofaccess",
    "windowssystem",
    "windowstools",
    // Kullanıcı profili klasörleri
    "appdata",
    "desktop",
    "documents",
    "downloads",
    "music",
    "pictures",
    "videos",
    "onedrive",
    "favorites",
    // Kendi veri klasörümüz
    "dmedic",
];

/// Ad korumalı mı (büyük/küçük harf ve noktalama duyarsız).
pub fn is_protected_name(name: &str) -> bool {
    let k = norm_key(name);
    !k.is_empty() && PROTECTED_NAMES.contains(&k.as_str())
}

// ----------------------------------------------------------------------------
// Dosya sistemi
// ----------------------------------------------------------------------------

/// Yolu karşılaştırma biçimine getir: küçük harf, `\` ayraç, `\\?\` öneki ve
/// sondaki ayraç atılır.
fn norm_path(p: &str) -> String {
    let mut s = p.trim().replace('/', "\\").to_lowercase();
    if let Some(rest) = s.strip_prefix(r"\\?\") {
        s = rest.to_string();
    }
    while s.len() > 3 && s.ends_with('\\') {
        s.pop();
    }
    s
}

/// `child`, `parent` ile aynı ya da onun altında mı (normalize edilmiş yollar).
fn is_within(child: &str, parent: &str) -> bool {
    child == parent || (child.starts_with(parent) && child[parent.len()..].starts_with('\\'))
}

/// Silme/taşıma hedefi olabilecek dosya sistemi yollarını denetler.
///
/// Kurallar:
///  * Yol sürücü-mutlak olmalı (`C:\...`); UNC, göreli yol ve `.`/`..` reddedilir.
///  * Sürücü kökü, `%SystemRoot%` ve altı, D-Medic veri klasörü korunur.
///  * Bilinen kökler (Program Files, AppData, ProgramData, Başlat menüsü…)
///    kendileri ve ataları korunur.
///  * Yol bir kökün altındaysa, o köke göre ilk bileşen [`is_protected_name`]
///    ise korunur (örn. `%LOCALAPPDATA%\Microsoft\...`). En derin kök esas
///    alınır; böylece `%APPDATA%\Microsoft\Windows\Start Menu\Programs\Acme.lnk`
///    izinli, `%APPDATA%\Microsoft\Credentials` korumalı kalır.
#[derive(Debug, Clone, Default)]
pub struct FsGuard {
    system_root: Option<String>,
    own_data: Option<String>,
    /// Altındaki öğelerin silinebildiği, kendisinin silinemediği kökler.
    containers: Vec<String>,
}

impl FsGuard {
    /// Ortam değişkenlerinden (çalışan makinenin gerçek yolları) kur.
    pub fn from_env() -> Self {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        let mut containers: Vec<String> = [
            "ProgramFiles",
            "ProgramFiles(x86)",
            "ProgramW6432",
            "CommonProgramFiles",
            "CommonProgramFiles(x86)",
            "CommonProgramW6432",
            "ProgramData",
            "APPDATA",
            "LOCALAPPDATA",
            "PUBLIC",
            "USERPROFILE",
        ]
        .iter()
        .filter_map(|k| var(k))
        .collect();
        if let Some(local) = var("LOCALAPPDATA") {
            containers.push(format!(r"{local}\Programs"));
        }
        for base in [var("APPDATA"), var("ProgramData")].into_iter().flatten() {
            containers.push(format!(r"{base}\Microsoft\Windows\Start Menu\Programs"));
        }
        let own_data = var("APPDATA").map(|a| format!(r"{a}\D-Medic"));
        Self::new(var("SystemRoot"), own_data, containers)
    }

    pub fn new(
        system_root: Option<String>,
        own_data: Option<String>,
        containers: Vec<String>,
    ) -> Self {
        let mut containers: Vec<String> = containers.iter().map(|c| norm_path(c)).collect();
        // Kullanıcı profillerinin ortak atası (örn. C:\Users) da korunsun.
        let parents: Vec<String> = containers
            .iter()
            .filter_map(|c| c.rsplit_once('\\').map(|(p, _)| p.to_string()))
            .filter(|p| p.len() > 3)
            .collect();
        containers.extend(parents);
        containers.sort();
        containers.dedup();
        Self {
            system_root: system_root.map(|s| norm_path(&s)),
            own_data: own_data.map(|s| norm_path(&s)),
            containers,
        }
    }

    /// `Ok(())` → silinebilir; `Err(neden)` → korumalı ya da geçersiz yol.
    pub fn check(&self, path: &str) -> Result<(), String> {
        let p = norm_path(path);
        let bytes = p.as_bytes();
        let drive_abs = bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && bytes[2] == b'\\';
        if !drive_abs {
            return Err(format!("mutlak yerel yol değil: {path}"));
        }
        let comps: Vec<&str> = p[3..].split('\\').collect();
        if comps
            .iter()
            .any(|c| c.is_empty() || *c == "." || *c == "..")
        {
            return Err(format!("geçersiz yol bileşeni: {path}"));
        }
        if p.len() <= 3 {
            return Err(format!("sürücü kökü silinemez: {path}"));
        }
        if let Some(sr) = &self.system_root {
            if is_within(&p, sr) || is_within(sr, &p) {
                return Err(format!("Windows sistem klasörü korumalı: {path}"));
            }
        }
        if let Some(own) = &self.own_data {
            if is_within(&p, own) || is_within(own, &p) {
                return Err(format!("D-Medic veri klasörü korumalı: {path}"));
            }
        }
        // Bir kökün kendisi ya da atası mı?
        if self.containers.iter().any(|c| is_within(c, &p)) {
            return Err(format!("sistem kök klasörü korumalı: {path}"));
        }
        // En derin kapsayan köke göre ilk bileşen korumalı ad mı?
        if let Some(root) = self
            .containers
            .iter()
            .filter(|c| is_within(&p, c))
            .max_by_key(|c| c.len())
        {
            let first = p[root.len() + 1..].split('\\').next().unwrap_or("");
            if is_protected_name(first) {
                return Err(format!("ortak sistem klasörü korumalı: {path}"));
            }
        }
        Ok(())
    }
}

// ----------------------------------------------------------------------------
// Registry
// ----------------------------------------------------------------------------

/// Run / RunOnce: tek tek değer silinmesine izin verilen otomatik başlatma anahtarları.
const AUTORUN_KEYS: &[&str] = &[
    r"software\microsoft\windows\currentversion\run",
    r"software\microsoft\windows\currentversion\runonce",
];

/// Registry yolunu (kök, bileşenler) olarak ayrıştır. Yalnız HKCU / HKLM.
fn split_reg(full: &str) -> Result<(&str, Vec<&str>), String> {
    let mut parts = full.split('\\');
    let root = parts.next().unwrap_or("");
    let rest: Vec<&str> = parts.collect();
    if !root.eq_ignore_ascii_case("HKCU") && !root.eq_ignore_ascii_case("HKLM") {
        return Err(format!(
            "yalnız HKCU/HKLM altında silme yapılabilir: {full}"
        ));
    }
    if rest.iter().any(|c| c.trim().is_empty()) {
        return Err(format!("geçersiz registry yolu: {full}"));
    }
    Ok((root, rest))
}

/// Bir registry anahtarının (alt ağacıyla) silinebilir olup olmadığını denetle.
/// Yalnız `SOFTWARE\<Ürün>` veya `SOFTWARE\<Yayıncı>\<Ürün>` biçimi kabul edilir
/// ve `SOFTWARE` altındaki ilk bileşen korumalı ad olamaz.
pub fn check_reg_key(full: &str) -> Result<(), String> {
    let (_, rest) = split_reg(full)?;
    if rest.len() < 2 || !rest[0].eq_ignore_ascii_case("software") {
        return Err(format!(
            "yalnız SOFTWARE altındaki ürün anahtarları silinebilir: {full}"
        ));
    }
    if is_protected_name(rest[1]) {
        return Err(format!("ortak registry dalı korumalı: {full}"));
    }
    Ok(())
}

/// Tek bir registry değerinin silinebilir olup olmadığını denetle: Run/RunOnce
/// anahtarlarındaki adlandırılmış değerler ya da silinebilir bir ürün anahtarı.
pub fn check_reg_value(full: &str, value_name: &str) -> Result<(), String> {
    if value_name.trim().is_empty() {
        return Err(format!("varsayılan (adsız) değer silinemez: {full}"));
    }
    let (_, rest) = split_reg(full)?;
    let sub = rest.join("\\").to_lowercase();
    if AUTORUN_KEYS.contains(&sub.as_str()) {
        return Ok(());
    }
    check_reg_key(full)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn guard() -> FsGuard {
        FsGuard::new(
            Some(r"C:\Windows".into()),
            Some(r"C:\Users\u\AppData\Roaming\D-Medic".into()),
            vec![
                r"C:\Program Files".into(),
                r"C:\Program Files (x86)".into(),
                r"C:\Program Files\Common Files".into(),
                r"C:\ProgramData".into(),
                r"C:\Users\u\AppData\Roaming".into(),
                r"C:\Users\u\AppData\Local".into(),
                r"C:\Users\u\AppData\Local\Programs".into(),
                r"C:\Users\u".into(),
                r"C:\Users\Public".into(),
                r"C:\Users\u\AppData\Roaming\Microsoft\Windows\Start Menu\Programs".into(),
            ],
        )
    }

    #[test]
    fn storage_id_rejects_traversal() {
        assert!(is_valid_storage_id("20260101-120000-123"));
        assert!(is_valid_storage_id("5f0c6f9e-1b2a-4c3d-8e9f-001122334455"));
        assert!(!is_valid_storage_id(""));
        assert!(!is_valid_storage_id(".."));
        assert!(!is_valid_storage_id(r"..\..\Windows"));
        assert!(!is_valid_storage_id("a/b"));
        assert!(!is_valid_storage_id("C:"));
        assert!(!is_valid_storage_id(&"a".repeat(65)));
    }

    #[test]
    fn fs_allows_product_leftovers() {
        let g = guard();
        assert!(g.check(r"C:\Program Files\AcmeApp").is_ok());
        assert!(g.check(r"C:\Program Files (x86)\Acme Corp\AcmeApp").is_ok());
        assert!(g.check(r"C:\Users\u\AppData\Local\AcmeApp").is_ok());
        assert!(g
            .check(r"C:\Users\u\AppData\Local\Programs\Microsoft VS Code")
            .is_ok());
        assert!(g
            .check(r"C:\Users\u\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Acme.lnk")
            .is_ok());
        assert!(g.check(r"D:\Games\Acme").is_ok());
        assert!(g.check(r"\\?\C:\Program Files\AcmeApp\").is_ok());
    }

    #[test]
    fn fs_blocks_roots_and_shared_dirs() {
        let g = guard();
        for bad in [
            r"C:\",
            r"C:",
            r"D:\",
            r"C:\Windows",
            r"C:\Windows\System32\drivers",
            r"c:/windows/system32",
            r"C:\Program Files",
            r"C:\Program Files\",
            r"C:\Program Files\Microsoft",
            r"C:\Program Files\Windows Defender",
            r"C:\Program Files\Common Files",
            r"C:\Program Files\Common Files\Microsoft Shared",
            r"C:\Users",
            r"C:\Users\u",
            r"C:\Users\u\Documents",
            r"C:\Users\u\AppData",
            r"C:\Users\u\AppData\Local\Microsoft",
            r"C:\Users\u\AppData\Local\Microsoft\Edge",
            r"C:\Users\u\AppData\Local\Packages",
            r"C:\Users\u\AppData\Local\Temp",
            r"C:\Users\u\AppData\Local\Programs",
            r"C:\Users\u\AppData\Roaming\Microsoft\Credentials",
            r"C:\Users\u\AppData\Roaming\Microsoft\Windows\Start Menu\Programs",
            r"C:\Users\u\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup",
            r"C:\Users\u\AppData\Roaming\D-Medic\quarantine",
            r"C:\ProgramData\Microsoft",
            r"C:\Program Files\AcmeApp\..\..\Windows",
            r"\\server\share\x",
            r"relative\path",
            "",
        ] {
            assert!(g.check(bad).is_err(), "korunmalıydı: {bad}");
        }
    }

    #[test]
    fn reg_key_rules() {
        assert!(check_reg_key(r"HKLM\SOFTWARE\AcmeApp").is_ok());
        assert!(check_reg_key(r"HKCU\Software\Acme Corp\AcmeApp").is_ok());
        for bad in [
            r"HKLM\SOFTWARE\Microsoft",
            r"HKLM\SOFTWARE\Microsoft\Windows",
            r"HKLM\SOFTWARE\Classes",
            r"HKLM\SOFTWARE\WOW6432Node",
            r"HKLM\SOFTWARE\Policies\Acme",
            r"HKLM\SOFTWARE",
            r"HKLM\SYSTEM\CurrentControlSet",
            r"HKCR\AcmeFile",
            r"HKU\S-1-5-18\Software\Acme",
            r"HKLM\SOFTWARE\\Acme",
            r"HKLM",
        ] {
            assert!(check_reg_key(bad).is_err(), "korunmalıydı: {bad}");
        }
    }

    #[test]
    fn reg_value_rules() {
        let run = r"HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Run";
        assert!(check_reg_value(run, "AcmeUpdater").is_ok());
        assert!(check_reg_value(run, "").is_err());
        assert!(check_reg_value(r"HKLM\SOFTWARE\AcmeApp", "Path").is_ok());
        assert!(check_reg_value(
            r"HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion",
            "ProgramFilesDir"
        )
        .is_err());
    }
}
