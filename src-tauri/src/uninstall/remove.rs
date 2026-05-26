//! Seçili kalıntıların silinmesi — her zaman karantina + reg-export üzerinden.
//!
//! Dosya/klasör: karantinaya taşınır (geri yüklenebilir). Registry: önce `.reg`
//! export, sonra anahtar/değer silinir. Tek bir öğenin başarısızlığı diğerlerini
//! durdurmaz; sonuç [`RemovalReport`] ile döner.

use std::path::PathBuf;

use crate::error::{DMedicError, DMedicResult};
use crate::models::{LeftoverItem, LeftoverKind, RemovalItemResult, RemovalReport};

use super::quarantine::Batch;

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Seçili kalıntıları sil. `program_label` karantina partisini etiketler.
pub async fn remove_items(
    program_label: String,
    items: Vec<LeftoverItem>,
) -> DMedicResult<RemovalReport> {
    tokio::task::spawn_blocking(move || remove_blocking(&program_label, items))
        .await
        .map_err(|e| DMedicError::Other(format!("silme join: {e}")))?
}

fn remove_blocking(program_label: &str, items: Vec<LeftoverItem>) -> DMedicResult<RemovalReport> {
    let mut batch = Batch::create(program_label)?;
    let mut results = Vec::with_capacity(items.len());
    let mut removed = 0usize;
    let mut failed = 0usize;
    let mut freed_bytes = 0u64;

    for item in &items {
        let res = remove_one(&mut batch, item);
        match &res {
            Ok(bytes) => {
                removed += 1;
                freed_bytes += bytes;
                results.push(RemovalItemResult {
                    item_id: item.id.clone(),
                    path: item.path.clone(),
                    success: true,
                    message: None,
                });
            }
            Err(e) => {
                failed += 1;
                results.push(RemovalItemResult {
                    item_id: item.id.clone(),
                    path: item.path.clone(),
                    success: false,
                    message: Some(e.to_string()),
                });
            }
        }
    }

    let manifest = batch.finalize()?;
    tracing::info!(
        program = program_label,
        quarantine_id = %manifest.id,
        removed,
        failed,
        "Kalıntı silme tamamlandı"
    );

    Ok(RemovalReport {
        quarantine_id: manifest.id,
        removed,
        failed,
        freed_bytes,
        results,
    })
}

/// Tek bir öğeyi sil; karantinaya alınan byte miktarını döner.
fn remove_one(batch: &mut Batch, item: &LeftoverItem) -> DMedicResult<u64> {
    match item.kind {
        LeftoverKind::File | LeftoverKind::Folder => {
            let path = PathBuf::from(&item.path);
            if !path.exists() {
                return Ok(0); // Zaten yok — başarı say.
            }
            batch.quarantine_path(&path)
        }
        LeftoverKind::RegKey => {
            batch.export_reg(&item.path)?;
            delete_reg_key(&item.path)?;
            Ok(0)
        }
        LeftoverKind::RegValue => {
            batch.export_reg(&item.path)?;
            let value_name = item.value_name.as_deref().unwrap_or("");
            delete_reg_value(&item.path, value_name)?;
            Ok(0)
        }
    }
}

// ----------------------------------------------------------------------------
// Registry silme
// ----------------------------------------------------------------------------

/// "HKLM\\SOFTWARE\\X" → (predef kök, "SOFTWARE\\X").
#[cfg(windows)]
fn split_reg_path(full: &str) -> DMedicResult<(RegKey, String)> {
    let (label, sub) = full
        .split_once('\\')
        .ok_or_else(|| DMedicError::Validation(format!("geçersiz registry yolu: {full}")))?;
    let root = match label {
        "HKCU" => RegKey::predef(HKEY_CURRENT_USER),
        "HKLM" => RegKey::predef(HKEY_LOCAL_MACHINE),
        "HKCR" => RegKey::predef(HKEY_CLASSES_ROOT),
        "HKU" => RegKey::predef(HKEY_USERS),
        other => {
            return Err(DMedicError::Validation(format!(
                "bilinmeyen registry kökü: {other}"
            )))
        }
    };
    Ok((root, sub.to_string()))
}

/// Anahtar alt-ağacını sil. 64/32-bit view'ların ikisini de dener (kalıntı
/// taraması redirection-şeffaf yol kaydeder).
#[cfg(windows)]
fn delete_reg_key(full: &str) -> DMedicResult<()> {
    let (root, sub) = split_reg_path(full)?;
    let (parent, leaf) = match sub.rsplit_once('\\') {
        Some((p, l)) => (p.to_string(), l.to_string()),
        None => (String::new(), sub.clone()),
    };
    let mut last_err: Option<std::io::Error> = None;
    for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
        let parent_key = if parent.is_empty() {
            // Doğrudan kökün altındaki anahtar.
            match root.open_subkey_with_flags("", KEY_READ | KEY_WRITE | view) {
                Ok(k) => k,
                Err(e) => {
                    last_err = Some(e);
                    continue;
                }
            }
        } else {
            match root.open_subkey_with_flags(&parent, KEY_READ | KEY_WRITE | view) {
                Ok(k) => k,
                Err(e) => {
                    last_err = Some(e);
                    continue;
                }
            }
        };
        match parent_key.delete_subkey_all(&leaf) {
            Ok(()) => return Ok(()),
            Err(e) => last_err = Some(e),
        }
    }
    Err(DMedicError::Other(format!(
        "registry anahtarı silinemedi ({full}): {}",
        last_err
            .map(|e| e.to_string())
            .unwrap_or_else(|| "bilinmeyen".into())
    )))
}

/// Tek registry değerini sil (her iki view).
#[cfg(windows)]
fn delete_reg_value(full: &str, value_name: &str) -> DMedicResult<()> {
    let (root, sub) = split_reg_path(full)?;
    let mut last_err: Option<std::io::Error> = None;
    for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
        match root.open_subkey_with_flags(&sub, KEY_SET_VALUE | view) {
            Ok(key) => match key.delete_value(value_name) {
                Ok(()) => return Ok(()),
                Err(e) => last_err = Some(e),
            },
            Err(e) => last_err = Some(e),
        }
    }
    Err(DMedicError::Other(format!(
        "registry değeri silinemedi ({full}\\{value_name}): {}",
        last_err
            .map(|e| e.to_string())
            .unwrap_or_else(|| "bilinmeyen".into())
    )))
}

#[cfg(not(windows))]
fn delete_reg_key(_full: &str) -> DMedicResult<()> {
    Ok(())
}

#[cfg(not(windows))]
fn delete_reg_value(_full: &str, _value_name: &str) -> DMedicResult<()> {
    Ok(())
}
