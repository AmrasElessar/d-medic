use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DMedicError {
    #[error("IO hatası: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON hatası: {0}")]
    Json(#[from] serde_json::Error),

    #[error("simd-json hatası: {0}")]
    SimdJson(#[from] simd_json::Error),

    #[error("PowerShell hatası: {0}")]
    PowerShell(String),

    #[cfg(windows)]
    #[error("WMI hatası: {0}")]
    Wmi(String),

    #[error("Yönetici yetkisi gerekiyor")]
    NotElevated,

    #[error("Bulunamadı: {0}")]
    NotFound(String),

    #[error("Doğrulama hatası: {0}")]
    Validation(String),

    #[error("Beklenmeyen hata: {0}")]
    Other(String),
}

/// Tauri komutlarından dönen Result için kısa alias.
pub type DMedicResult<T> = Result<T, DMedicError>;

/// Frontend tarafına serileştirilen hata gövdesi.
/// JS tarafı için tutarlı bir { code, message } şekli sağlar.
#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub code: &'static str,
    pub message: String,
}

impl Serialize for DMedicError {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        let code = match self {
            Self::Io(_) => "io",
            Self::Json(_) | Self::SimdJson(_) => "json",
            Self::PowerShell(_) => "powershell",
            #[cfg(windows)]
            Self::Wmi(_) => "wmi",
            Self::NotElevated => "not_elevated",
            Self::NotFound(_) => "not_found",
            Self::Validation(_) => "validation",
            Self::Other(_) => "other",
        };
        ErrorPayload {
            code,
            message: self.to_string(),
        }
        .serialize(ser)
    }
}
