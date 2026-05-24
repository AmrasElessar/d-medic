//! Registry export / import — rollback için kritik HKCU/HKLM dalları.
//!
//! Tam hive dump GB'larca olabildiği için sadece D-Medic'in dokunabileceği
//! sınırlı dalları (HKCU\Software ve HKLM\Software\Microsoft\Windows) export
//! ediyoruz.

use crate::error::{DMedicError, DMedicResult};
use crate::paths;
use crate::ps;

/// Güvenli HKCU/HKLM dallarını .reg dosyasına yaz, snapshot id'siyle isimlendir.
pub async fn export_safe_hives(snapshot_id: &str) -> DMedicResult<Vec<String>> {
    let dir = paths::backups_dir().map_err(|e| DMedicError::Other(e.to_string()))?;
    std::fs::create_dir_all(&dir)?;
    let hkcu = dir.join(format!("{snapshot_id}_hkcu.reg"));
    let hklm = dir.join(format!("{snapshot_id}_hklm-windows.reg"));

    // reg.exe export çıktıyı UTF-16 LE BOM ile yazar — import'ta sorun değil.
    // /reg:64 → 64-bit kayıt görünümünü zorla; D-Medic 64-bit app olduğu için
    // default zaten 64-view ama WOW6432Node dalına yanlışlıkla düşmemek için
    // explicit. /y → mevcut dosyayı sorgusuz üzerine yaz.
    let script = format!(
        "reg.exe export 'HKCU\\Software' '{}' /y /reg:64; \
         reg.exe export 'HKLM\\Software\\Microsoft\\Windows' '{}' /y /reg:64",
        hkcu.display(),
        hklm.display()
    );
    ps::runner::run_script(&script).await?;

    Ok(vec![
        hkcu.to_string_lossy().to_string(),
        hklm.to_string_lossy().to_string(),
    ])
}

/// Bir snapshot'a ait .reg dosyalarını sırayla import et.
/// reg import destructive değildir — sadece var olan değerleri eski haline döndürür;
/// snapshot sonrasında eklenen anahtarlar SİLİNMEZ (reg.exe import limitasyonu).
pub async fn import_safe_hives(paths: &[String]) -> DMedicResult<Vec<(String, bool)>> {
    let mut results = Vec::new();
    for p in paths {
        // HKLM import elevation gerektirir, fail olursa kullanıcıya görünür kalsın.
        // /reg:64 → export'la aynı view'da import et.
        let script = format!(
            "reg.exe import '{}' /reg:64 2>&1; $LASTEXITCODE",
            p.replace('\'', "''"),
        );
        let out = ps::runner::run_script(&script).await?;
        let success = out.status == 0
            && out.stdout.lines().last().map(|s| s.trim() == "0").unwrap_or(false);
        results.push((p.clone(), success));
    }
    Ok(results)
}
