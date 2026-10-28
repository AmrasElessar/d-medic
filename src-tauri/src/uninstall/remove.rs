//! Seçili kalıntıların silinmesi — her zaman karantina + reg-export üzerinden.
//!
//! Dosya/klasör: karantinaya taşınır (geri yüklenebilir). Registry: önce `.reg`
//! export, sonra anahtar/değer silinir. Tek bir öğenin başarısızlığı diğerlerini
//! durdurmaz; sonuç [`RemovalReport`] ile döner.

use std::path::PathBuf;

use crate::error::{DMedicError, DMedicResult};
use crate::models::{LeftoverItem, LeftoverKind, RemovalItemResult, RemovalReport};

use super::quarantine::Batch;
use crate::safety::{self, FsGuard};

#[cfg(windows)]
use winreg::enums::*;
#[cfg(windows)]
use winreg::RegKey;

/// Seçili kalıntıları sil. `program_label` karantina partisini etiketler.
/// Frontend yalnız öğe id'lerini gönderir; yollar son taramanın backend'deki
/// kaydından çözülür. Ayrıca her öğe [`safety`] kontrolünden geçer; korumalı
/// yol/anahtar o öğe için hata olarak raporlanır, hiçbir şey silinmez.
pub async fn remove_items(
    program_label: String,
    item_ids: Vec<String>,
) -> DMedicResult<RemovalReport> {
    let items = super::leftover::resolve_items(&item_ids);
    tokio::task::spawn_blocking(move || remove_blocking(&program_label, items))
        .await
        .map_err(|e| DMedicError::Other(format!("silme join: {e}")))?
}

fn remove_blocking(
    program_label: &str,
    items: Vec<Result<LeftoverItem, String>>,
) -> DMedicResult<RemovalReport> {
    let guard = FsGuard::from_env();
    let mut batch = Batch::create(program_label)?;
    let mut results = Vec::with_capacity(items.len());
    let mut removed = 0usize;
    let mut failed = 0usize;
    let mut freed_bytes = 0u64;

    for item in &items {
        let item = match item {
            Ok(item) => item,
            Err(unknown_id) => {
                failed += 1;
                results.push(RemovalItemResult {
                    item_id: unknown_id.clone(),
                    path: String::new(),
                    success: false,
                    message: Some("Öğe son kalıntı taramasında yok — yeniden tarayın".into()),
                });
                continue;
            }
        };
        let res = remove_one(&mut batch, &guard, item);
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
fn remove_one(batch: &mut Batch, guard: &FsGuard, item: &LeftoverItem) -> DMedicResult<u64> {
    match item.kind {
        LeftoverKind::File | LeftoverKind::Folder => {
            guard.check(&item.path).map_err(DMedicError::Validation)?;
            let path = PathBuf::from(&item.path);
            if !path.exists() {
                return Ok(0); // Zaten yok — başarı say.
            }
            batch.quarantine_path(&path)
        }
        LeftoverKind::RegKey => {
            safety::check_reg_key(&item.path).map_err(DMedicError::Validation)?;
            remove_reg_key(batch, &item.path)?;
            Ok(0)
        }
        LeftoverKind::RegValue => {
            let value_name = item.value_name.as_deref().unwrap_or("");
            safety::check_reg_value(&item.path, value_name).map_err(DMedicError::Validation)?;
            remove_reg_value(batch, &item.path, value_name)?;
            Ok(0)
        }
    }
}

// ----------------------------------------------------------------------------
// Registry silme
// ----------------------------------------------------------------------------

/// "HKLM\\SOFTWARE\\X" → (predef kök, "SOFTWARE\\X", HKLM mi).
#[cfg(windows)]
fn split_reg_path(full: &str) -> DMedicResult<(RegKey, String, bool)> {
    let (label, sub) = full
        .split_once('\\')
        .ok_or_else(|| DMedicError::Validation(format!("geçersiz registry yolu: {full}")))?;
    match label.to_ascii_uppercase().as_str() {
        "HKCU" => Ok((RegKey::predef(HKEY_CURRENT_USER), sub.to_string(), false)),
        "HKLM" => Ok((RegKey::predef(HKEY_LOCAL_MACHINE), sub.to_string(), true)),
        other => Err(DMedicError::Validation(format!(
            "bilinmeyen registry kökü: {other}"
        ))),
    }
}

/// Denenecek registry görünümleri: (erişim bayrağı, reg.exe `/reg:` değeri).
/// HKLM\\SOFTWARE 32/64-bit ayrıdır; HKCU\\Software paylaşımlıdır.
#[cfg(windows)]
fn views(is_hklm: bool) -> &'static [(u32, &'static str)] {
    if is_hklm {
        &[(KEY_WOW64_64KEY, "64"), (KEY_WOW64_32KEY, "32")]
    } else {
        &[(KEY_WOW64_64KEY, "64")]
    }
}

/// Anahtarın bulunduğu HER görünümde: önce o görünümün `.reg` yedeği, sonra
/// alt-ağaç silme. Yedeklenemeyen görünümde silme yapılmaz.
#[cfg(windows)]
fn remove_reg_key(batch: &mut Batch, full: &str) -> DMedicResult<()> {
    let (root, sub, is_hklm) = split_reg_path(full)?;
    let (parent, leaf) = sub
        .rsplit_once('\\')
        .ok_or_else(|| DMedicError::Validation(format!("geçersiz registry yolu: {full}")))?;
    for &(flag, view) in views(is_hklm) {
        // Paylaşımlı anahtar önceki görünümde silinmiş olabilir — yeniden kontrol.
        if root.open_subkey_with_flags(&sub, KEY_READ | flag).is_err() {
            continue;
        }
        batch.export_reg(full, Some(view))?;
        let parent_key = root
            .open_subkey_with_flags(parent, KEY_READ | KEY_WRITE | flag)
            .map_err(|e| {
                DMedicError::Other(format!("registry anahtarı açılamadı ({full}): {e}"))
            })?;
        parent_key.delete_subkey_all(leaf).map_err(|e| {
            DMedicError::Other(format!("registry anahtarı silinemedi ({full}): {e}"))
        })?;
    }
    Ok(())
}

/// Değerin bulunduğu her görünümde: anahtarı yedekle, değeri sil.
#[cfg(windows)]
fn remove_reg_value(batch: &mut Batch, full: &str, value_name: &str) -> DMedicResult<()> {
    let (root, sub, is_hklm) = split_reg_path(full)?;
    for &(flag, view) in views(is_hklm) {
        let Ok(key) = root.open_subkey_with_flags(&sub, KEY_READ | KEY_SET_VALUE | flag) else {
            continue;
        };
        if key.get_raw_value(value_name).is_err() {
            continue;
        }
        batch.export_reg(full, Some(view))?;
        key.delete_value(value_name).map_err(|e| {
            DMedicError::Other(format!(
                "registry değeri silinemedi ({full}\\{value_name}): {e}"
            ))
        })?;
    }
    Ok(())
}

#[cfg(not(windows))]
fn remove_reg_key(_batch: &mut Batch, _full: &str) -> DMedicResult<()> {
    Ok(())
}

#[cfg(not(windows))]
fn remove_reg_value(_batch: &mut Batch, _full: &str, _value_name: &str) -> DMedicResult<()> {
    Ok(())
}
