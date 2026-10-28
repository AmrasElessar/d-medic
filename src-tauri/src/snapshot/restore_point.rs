//! Windows System Restore Point oluşturma.
//!
//! `Checkpoint-Computer` iki durumda nokta OLUŞTURMADAN döner:
//!  * Son 24 saatte zaten bir nokta varsa (SystemRestorePointCreationFrequency)
//!    yalnız uyarı yazar, hata vermez.
//!  * System Protection kapalıysa hata verir.
//!
//! Bu yüzden başarı, en yüksek `SequenceNumber`'ın değişmesiyle doğrulanır.
//! System Protection kullanıcıya sorulmadan AÇILMAZ — kapalıysa `false` döner ve
//! snapshot'ın diğer katmanları (reg export + servis durumu) yine alınır.

use crate::error::DMedicResult;
use crate::ps;

/// Oluşturma sonucu; `note` yalnız oluşturulmadığında dolu (UI'da gösterilir).
pub struct Outcome {
    pub created: bool,
    pub note: Option<String>,
}

pub async fn create(description: &str) -> DMedicResult<Outcome> {
    let script = format!(
        "function Get-LastSeq {{ (Get-ComputerRestorePoint -ErrorAction SilentlyContinue | \
             Measure-Object -Property SequenceNumber -Maximum).Maximum }}; \
         $before = Get-LastSeq; \
         Checkpoint-Computer -Description '{}' -RestorePointType 'MODIFY_SETTINGS' \
             -ErrorAction Stop -WarningAction SilentlyContinue -WarningVariable cpWarn; \
         $after = Get-LastSeq; \
         if ($after -and $after -ne $before) {{ 'CREATED' }} else {{ 'SKIPPED'; $cpWarn }}",
        description.replace('\'', "''")
    );
    let out = ps::runner::run_script(&script).await?;
    let created = out.stdout.lines().any(|l| l.trim() == "CREATED");
    if created {
        return Ok(Outcome {
            created,
            note: None,
        });
    }
    let reason = if out.status != 0 {
        format!(
            "System Protection kapalı olabilir: {}",
            out.stderr
                .trim()
                .lines()
                .next()
                .unwrap_or("bilinmeyen hata")
        )
    } else {
        out.stdout
            .lines()
            .map(str::trim)
            .find(|l| !l.is_empty() && *l != "SKIPPED")
            .unwrap_or("son 24 saatte zaten bir geri yükleme noktası var")
            .to_string()
    };
    tracing::warn!(%reason, "Geri yükleme noktası oluşturulmadı");
    Ok(Outcome {
        created: false,
        note: Some(reason),
    })
}

/// Sistem sürücüsünde System Protection'ı aç. Yalnız kullanıcı UI'da açıkça
/// onayladığında çağrılır.
pub async fn enable_protection() -> DMedicResult<()> {
    let out = ps::runner::run_script(
        "Enable-ComputerRestore -Drive ($env:SystemDrive + '\\') -ErrorAction Stop; 'OK'",
    )
    .await?;
    if out.stdout.lines().any(|l| l.trim() == "OK") {
        Ok(())
    } else {
        Err(crate::error::DMedicError::PowerShell(
            out.stderr
                .trim()
                .lines()
                .next()
                .unwrap_or("bilinmeyen hata")
                .to_string(),
        ))
    }
}
