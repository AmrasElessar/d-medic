//! simd-json wrapper. PowerShell çıktısı UTF-8'e normalize edildikten sonra
//! buradaki helper'lar AVX2/SSE ile parse eder.

use serde::de::DeserializeOwned;

use crate::error::DMedicResult;

pub fn from_str<T: DeserializeOwned>(s: &str) -> DMedicResult<T> {
    let mut bytes = s.as_bytes().to_vec();
    let value: T = simd_json::serde::from_slice(&mut bytes)?;
    Ok(value)
}

pub fn from_slice<T: DeserializeOwned>(buffer: &mut [u8]) -> DMedicResult<T> {
    let value: T = simd_json::serde::from_slice(buffer)?;
    Ok(value)
}
