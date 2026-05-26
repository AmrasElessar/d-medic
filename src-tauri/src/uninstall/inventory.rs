//! Kurulu program envanteri.
//!
//! İki kaynak:
//!  * **Win32** — registry `...\CurrentVersion\Uninstall` dalları (HKLM 64-bit,
//!    HKLM 32-bit/WOW6432Node, HKCU). Native `winreg` ile okunur (reg.exe spawn
//!    yok — tek tarama yüzlerce alt-anahtar gezer).
//!  * **UWP/MSIX** — `Get-AppxPackage` PS batch'i (native AppX API'si windows-rs'te
//!    ergonomik değil; tek spawn ilkesini korur).

use serde::Deserialize;

use crate::error::DMedicResult;
use crate::models::{InstalledProgram, ProgramKind};
use crate::ps;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Tüm kurulu programları topla (Win32 + UWP), ada göre sıralı döner.
pub async fn list_all() -> DMedicResult<Vec<InstalledProgram>> {
    let mut win32 = tokio::task::spawn_blocking(list_win32)
        .await
        .map_err(|e| crate::error::DMedicError::Other(format!("win32 envanter join: {e}")))??;
    let uwp = list_uwp().await.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "UWP envanteri okunamadı, atlanıyor");
        Vec::new()
    });
    win32.extend(uwp);
    // Ada göre büyük/küçük harf duyarsız sırala.
    win32.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(win32)
}

/// Tek bir programı id ile bul (kaldırma/tarama için).
pub async fn find(program_id: &str) -> DMedicResult<Option<InstalledProgram>> {
    let all = list_all().await?;
    Ok(all.into_iter().find(|p| p.id == program_id))
}

// ----------------------------------------------------------------------------
// Win32 (registry)
// ----------------------------------------------------------------------------

#[cfg(windows)]
const UNINSTALL_PATH: &str = r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall";

#[cfg(windows)]
fn list_win32() -> DMedicResult<Vec<InstalledProgram>> {
    let mut out: Vec<InstalledProgram> = Vec::new();
    // (predefined kök, ek erişim bayrağı) — 64-bit, 32-bit (WOW6432Node), kullanıcı.
    let sources = [
        (HKEY_LOCAL_MACHINE, KEY_WOW64_64KEY),
        (HKEY_LOCAL_MACHINE, KEY_WOW64_32KEY),
        (HKEY_CURRENT_USER, KEY_WOW64_64KEY),
    ];
    for (root, view) in sources {
        let base = match RegKey::predef(root)
            .open_subkey_with_flags(UNINSTALL_PATH, KEY_READ | view)
        {
            Ok(k) => k,
            Err(_) => continue,
        };
        for sub_name in base.enum_keys().filter_map(Result::ok) {
            let Ok(sub) = base.open_subkey_with_flags(&sub_name, KEY_READ | view) else {
                continue;
            };
            if let Some(prog) = program_from_key(&sub, &sub_name) {
                // Aynı program 32/64 view'da çift gelebilir — id ile tekille.
                if !out.iter().any(|p| p.id == prog.id) {
                    out.push(prog);
                }
            }
        }
    }
    Ok(out)
}

#[cfg(windows)]
fn program_from_key(key: &RegKey, sub_name: &str) -> Option<InstalledProgram> {
    let opt_str = |name: &str| -> Option<String> {
        key.get_value::<String, _>(name)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    };

    // DisplayName yoksa kullanıcıya gösterilemez → atla.
    let name = opt_str("DisplayName")?;

    // Güncellemeleri / yamaları ele: KB####### desenleri, "Update for ...",
    // SystemComponent=1 işaretli sistem bileşenleri.
    let system_component = key
        .get_value::<u32, _>("SystemComponent")
        .map(|v| v == 1)
        .unwrap_or(false);
    let is_update = key.get_value::<String, _>("ParentKeyName").is_ok()
        || name.starts_with("Update for ")
        || name.starts_with("Security Update for ")
        || name.starts_with("Hotfix for ")
        || (name.starts_with("KB") && name[2..].chars().take(6).all(|c| c.is_ascii_digit()));
    if is_update {
        return None;
    }

    // EstimatedSize KB cinsinden DWORD → byte.
    let size_bytes = key
        .get_value::<u32, _>("EstimatedSize")
        .ok()
        .map(|kb| kb as u64 * 1024);

    Some(InstalledProgram {
        id: sub_name.to_string(),
        name,
        publisher: opt_str("Publisher"),
        version: opt_str("DisplayVersion"),
        kind: ProgramKind::Win32,
        install_location: opt_str("InstallLocation"),
        uninstall_string: opt_str("UninstallString"),
        quiet_uninstall_string: opt_str("QuietUninstallString"),
        size_bytes,
        install_date: opt_str("InstallDate"),
        icon_base64: None, // İkon çıkarımı sonraki tur — DisplayIcon path'ten.
        is_system_component: system_component,
    })
}

#[cfg(not(windows))]
fn list_win32() -> DMedicResult<Vec<InstalledProgram>> {
    Ok(Vec::new())
}

// ----------------------------------------------------------------------------
// UWP (Get-AppxPackage)
// ----------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct AppxRaw {
    #[serde(rename = "Name")]
    name: Option<String>,
    #[serde(rename = "PackageFullName")]
    package_full_name: Option<String>,
    #[serde(rename = "Publisher")]
    publisher: Option<String>,
    #[serde(rename = "Version")]
    version: Option<String>,
    #[serde(rename = "InstallLocation")]
    install_location: Option<String>,
}

async fn list_uwp() -> DMedicResult<Vec<InstalledProgram>> {
    // Framework / sistem-kritik (NonRemovable) paketleri ele. ConvertTo-Json
    // tek öğede dizi değil obje döndürür → çıktıyı her zaman diziye sar.
    let script = "$p = Get-AppxPackage | Where-Object { -not $_.IsFramework -and -not $_.NonRemovable } | \
         Select-Object Name, PackageFullName, Publisher, Version, InstallLocation; \
         @($p) | ConvertTo-Json -Depth 3 -Compress";
    let out = ps::runner::run_script(script).await?;
    let trimmed = out.stdout.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }
    let raws: Vec<AppxRaw> = serde_json::from_str(trimmed).unwrap_or_default();
    Ok(raws
        .into_iter()
        .filter_map(|r| {
            let full = r.package_full_name?;
            let name = r.name.unwrap_or_else(|| full.clone());
            Some(InstalledProgram {
                id: full,
                name,
                publisher: r.publisher,
                version: r.version,
                kind: ProgramKind::Uwp,
                install_location: r.install_location,
                uninstall_string: None,
                quiet_uninstall_string: None,
                size_bytes: None,
                install_date: None,
                icon_base64: None,
                is_system_component: false,
            })
        })
        .collect())
}
