//! Çok-kaynaklı doğrulama sistemi.
//!
//! D-Medic'in her aksiyonu/check'i için Microsoft / CIS / topluluk konsensüsü
//! kaynaklarını tutar. Schema: `resources/verification.json`.
//!
//! Felsefe: bilgilendirilmiş onay. Microsoft "kapat" demese de Group Policy
//! ile belgeli alternatif varsa, kullanıcıya iki perspektifi de göster.
//! 3+ bağımsız kaynak onayı + zarar kaydı yok = güvenli; tek topluluk = risk.

pub mod loader;
pub mod schema;

pub use loader::{get_record, get_all_records};
pub use schema::{VerificationLevel, VerificationRecord, VerificationSource, SourceType, SourceStance};
