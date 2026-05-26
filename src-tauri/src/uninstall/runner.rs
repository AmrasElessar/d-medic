//! Programın KENDİ kaldırıcısını çalıştırma.
//!
//! Revo modeli: önce uygulamanın resmi kaldırıcısı çalışır (kullanıcı kendi
//! sihirbazını görür), tamamlanınca biz kalıntı taramasına geçeriz. Bizim
//! kaldırıcımız değil — sadece tetikleyici + bekleyici.

use crate::error::{DMedicError, DMedicResult};
use crate::models::{InstalledProgram, ProgramKind, UninstallReport};
use crate::ps;

/// Verilen programın kaldırıcısını çalıştır ve bitmesini bekle.
pub async fn run(program: &InstalledProgram) -> DMedicResult<UninstallReport> {
    match program.kind {
        ProgramKind::Win32 => run_win32(program).await,
        ProgramKind::Uwp => run_uwp(program).await,
    }
}

async fn run_win32(program: &InstalledProgram) -> DMedicResult<UninstallReport> {
    // Sessiz komut varsa onu, yoksa standart (etkileşimli) kaldırıcıyı kullan.
    let cmdline = program
        .quiet_uninstall_string
        .clone()
        .or_else(|| program.uninstall_string.clone())
        .ok_or_else(|| {
            DMedicError::Validation(format!(
                "'{}' için kaldırma komutu (UninstallString) bulunamadı",
                program.name
            ))
        })?;

    tracing::info!(program = %program.name, "Win32 kaldırıcı başlatılıyor");

    // cmd /c komut satırını olduğu gibi yorumlar (tırnak/argüman ayrıştırması
    // Windows kabuğuna bırakılır). Kaldırıcı kendi penceresini açabilir →
    // CREATE_NO_WINDOW KULLANMIYORUZ, kullanıcı süreci görsün.
    let status = tokio::process::Command::new("cmd")
        .args(["/c", &cmdline])
        .status()
        .await
        .map_err(|e| DMedicError::Other(format!("kaldırıcı spawn: {e}")))?;

    let exit_code = status.code();
    let completed = status.success();
    Ok(UninstallReport {
        program_id: program.id.clone(),
        launched: true,
        completed,
        exit_code,
        message: if completed {
            format!("{} kaldırıcısı tamamlandı.", program.name)
        } else {
            format!(
                "{} kaldırıcısı sıfırdan farklı kod döndürdü (exit={:?}). \
                 Kaldırıcı arka planda devam ediyor olabilir; yine de kalıntı taraması yapılabilir.",
                program.name, exit_code
            )
        },
    })
}

async fn run_uwp(program: &InstalledProgram) -> DMedicResult<UninstallReport> {
    tracing::info!(package = %program.id, "UWP paketi kaldırılıyor");
    // PackageFullName tek tırnak içinde — defansif escape.
    let safe = program.id.replace('\'', "''");
    let script = format!(
        "Remove-AppxPackage -Package '{safe}' -ErrorAction Stop; $LASTEXITCODE"
    );
    let out = ps::runner::run_script(&script).await?;
    let completed = out.status == 0 && out.stderr.trim().is_empty();
    Ok(UninstallReport {
        program_id: program.id.clone(),
        launched: true,
        completed,
        exit_code: Some(out.status),
        message: if completed {
            format!("{} (UWP) kaldırıldı.", program.name)
        } else {
            format!(
                "UWP kaldırma başarısız: {}",
                out.stderr.trim().lines().next().unwrap_or("bilinmeyen hata")
            )
        },
    })
}
