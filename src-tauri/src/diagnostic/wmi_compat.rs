//! `wmi` crate'i yalnız Windows'ta derlenir. Diğer platformlarda (Linux CI'da
//! birim testleri koşabilsin diye) aynı API yüzeyini taklit eden, her sorguda
//! hata dönen bir stub sağlanır — check'ler "veri yok" yoluna düşer.

#[cfg(windows)]
pub use ::wmi::{COMLibrary, WMIConnection};

#[cfg(not(windows))]
pub use stub::{COMLibrary, WMIConnection};

#[cfg(not(windows))]
mod stub {
    use std::fmt;

    #[derive(Debug)]
    pub struct Unsupported;

    impl fmt::Display for Unsupported {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("WMI bu platformda desteklenmiyor")
        }
    }

    impl std::error::Error for Unsupported {}

    #[derive(Clone, Copy)]
    pub struct COMLibrary;

    impl COMLibrary {
        pub fn new() -> Result<Self, Unsupported> {
            Ok(Self)
        }

        /// # Safety
        /// Stub — gerçek COM durumu yok.
        pub unsafe fn assume_initialized() -> Self {
            Self
        }
    }

    pub struct WMIConnection;

    impl WMIConnection {
        pub fn new(_com: COMLibrary) -> Result<Self, Unsupported> {
            Err(Unsupported)
        }

        pub fn with_namespace_path(_ns: &str, _com: COMLibrary) -> Result<Self, Unsupported> {
            Err(Unsupported)
        }

        pub fn query<T>(&self) -> Result<Vec<T>, Unsupported> {
            Err(Unsupported)
        }

        pub fn raw_query<T>(&self, _query: impl AsRef<str>) -> Result<Vec<T>, Unsupported> {
            Err(Unsupported)
        }
    }
}
