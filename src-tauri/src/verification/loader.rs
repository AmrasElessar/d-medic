use std::sync::OnceLock;

use super::schema::{VerificationDb, VerificationRecord};

/// Verification DB tek seferlik yüklenir, sonra cache'lenir. JSON dosyası
/// resources/verification.json'da; tauri-bundle resource olarak dahil eder,
/// dev'de doğrudan disk'ten okunur.
static DB: OnceLock<VerificationDb> = OnceLock::new();

/// JSON içeriği derleme zamanında binary'ye gömülür — runtime okuma yok,
/// path sorunu yok. Trade-off: dosyayı değiştirince yeniden derleme gerek
/// (D-Medic kullanıcı tarafından yapılmaz; geliştirici turunda OK).
const VERIFICATION_JSON: &str = include_str!("../../resources/verification.json");

fn db() -> &'static VerificationDb {
    DB.get_or_init(|| {
        serde_json::from_str(VERIFICATION_JSON).unwrap_or_else(|e| {
            tracing::error!(error = %e, "verification.json parse edilemedi, boş DB kullanılıyor");
            VerificationDb {
                version: 1,
                records: Default::default(),
            }
        })
    })
}

/// Belirli bir action_id veya check_id için kayıt. Yoksa None.
pub fn get_record(id: &str) -> Option<&'static VerificationRecord> {
    db().records.get(id)
}

/// Tüm kayıtlar — UI'da "tüm aksiyonların doğrulama durumu" görünümü için.
pub fn get_all_records() -> &'static std::collections::HashMap<String, VerificationRecord> {
    &db().records
}
