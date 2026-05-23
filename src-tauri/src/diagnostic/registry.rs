//! Windows registry'den DWORD/string okuma için ince wrapper.
//!
//! windows-rs `RegGetValueW` üzerine yazılmış — `winreg` crate eklemeden
//! yalnızca okuma ihtiyacımız için yeterli. Yazma desteği yok (D-Medic
//! registry değişikliklerini PS aracılığıyla yapıyor, undo kolaylaşsın diye).

use windows::core::PCWSTR;
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::Registry::{
    RegCloseKey, RegOpenKeyExW, RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE,
    KEY_READ, RRF_RT_REG_DWORD, RRF_RT_REG_SZ,
};

pub const HKLM: HKEY = HKEY_LOCAL_MACHINE;
pub const HKCU: HKEY = HKEY_CURRENT_USER;

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// HKLM/HKCU\subkey\value DWORD okuma. Yoksa None.
pub fn read_dword(hive: HKEY, subkey: &str, value: &str) -> Option<u32> {
    let sk = to_wide(subkey);
    let vn = to_wide(value);
    let mut data: u32 = 0;
    let mut size: u32 = std::mem::size_of::<u32>() as u32;
    let res = unsafe {
        RegGetValueW(
            hive,
            PCWSTR(sk.as_ptr()),
            PCWSTR(vn.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut _ as *mut _),
            Some(&mut size),
        )
    };
    if res == ERROR_SUCCESS {
        Some(data)
    } else {
        None
    }
}

/// HKLM/HKCU\subkey altında bir anahtarın var olup olmadığını kontrol et.
/// Pending reboot gibi "değer önemli değil, anahtar varlığı yeter" senaryoları için.
pub fn key_exists(hive: HKEY, subkey: &str) -> bool {
    let sk = to_wide(subkey);
    let mut h: HKEY = HKEY::default();
    let res = unsafe { RegOpenKeyExW(hive, PCWSTR(sk.as_ptr()), 0, KEY_READ, &mut h) };
    if res == ERROR_SUCCESS {
        unsafe {
            let _ = RegCloseKey(h);
        }
        true
    } else {
        false
    }
}

/// HKLM/HKCU\subkey\value REG_SZ okuma. Yoksa None.
pub fn read_string(hive: HKEY, subkey: &str, value: &str) -> Option<String> {
    let sk = to_wide(subkey);
    let vn = to_wide(value);

    // İlk pas: boyutu öğren (bytes cinsinden). Dönüş kodu burada önemli değil —
    // ikinci pas zaten gerçek hatayı raporluyor.
    let mut size: u32 = 0;
    let _ = unsafe {
        RegGetValueW(
            hive,
            PCWSTR(sk.as_ptr()),
            PCWSTR(vn.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut size),
        )
    };
    if size == 0 {
        return None;
    }

    let mut buf = vec![0u16; (size as usize).div_ceil(2)];
    let res = unsafe {
        RegGetValueW(
            hive,
            PCWSTR(sk.as_ptr()),
            PCWSTR(vn.as_ptr()),
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr() as *mut _),
            Some(&mut size),
        )
    };
    if res != ERROR_SUCCESS {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..len]))
}
