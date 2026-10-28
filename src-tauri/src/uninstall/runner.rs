//! Programın KENDİ kaldırıcısını çalıştırma.
//!
//! Revo modeli: önce uygulamanın resmi kaldırıcısı çalışır (kullanıcı kendi
//! sihirbazını görür), tamamlanınca biz kalıntı taramasına geçeriz. Bizim
//! kaldırıcımız değil — sadece tetikleyici + bekleyici.

use crate::error::{DMedicError, DMedicResult};
use crate::models::{InstalledProgram, ProgramKind, UninstallReport};
use crate::ps;

use super::inventory;

/// Verilen programın kaldırıcısını çalıştır ve bitmesini bekle. Ardından
/// programın envanterden gerçekten düşüp düşmediği kontrol edilir.
pub async fn run(program: &InstalledProgram) -> DMedicResult<UninstallReport> {
    // SystemComponent=1 girişler Windows'un kendi "Programlar ve Özellikler"
    // listesinde bile gizlidir (sürücü / runtime parçaları); bunları başka bir
    // ürünün kaldırıcısı yönetir.
    if program.is_system_component {
        return Err(DMedicError::Validation(format!(
            "'{}' bir sistem bileşeni olarak işaretli; D-Medic bunu kaldırmaz",
            program.name
        )));
    }

    let mut report = match program.kind {
        ProgramKind::Win32 => run_win32(program).await?,
        ProgramKind::Uwp => run_uwp(program).await?,
    };

    report.still_registered = inventory::is_still_installed(program).await;
    if report.still_registered {
        report.completed = false;
        report.message = format!(
            "{} hâlâ kurulu görünüyor — kaldırma iptal edilmiş ya da arka planda sürüyor \
             olabilir. Kaldırıcı bitince kalıntı taramasını yeniden çalıştırın.",
            program.name
        );
    }
    Ok(report)
}

/// `MsiExec.exe /I{GUID}` kurulum/onarım sihirbazını açar; kaldırma için `/X`
/// gerekir. Birçok MSI paketi UninstallString'e `/I` yazar.
pub fn normalize_msi_uninstall(cmdline: &str) -> String {
    let trimmed = cmdline.trim();
    let lower = trimmed.to_ascii_lowercase();
    let exe_end = ["msiexec.exe", "msiexec"]
        .iter()
        .find_map(|exe| lower.strip_prefix(exe).map(|_| exe.len()));
    let Some(exe_end) = exe_end else {
        return trimmed.to_string();
    };
    let (exe, args) = trimmed.split_at(exe_end);
    let fixed: Vec<String> = args
        .split(' ')
        .map(|a| {
            let al = a.to_ascii_lowercase();
            if (al.starts_with("/i") || al.starts_with("-i")) && a[2..].starts_with('{') {
                format!("/X{}", &a[2..])
            } else if al == "/i" || al == "-i" {
                "/X".to_string()
            } else {
                a.to_string()
            }
        })
        .collect();
    format!("{exe}{}", fixed.join(" "))
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
    let cmdline = normalize_msi_uninstall(&cmdline);

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
        still_registered: false,
    })
}

async fn run_uwp(program: &InstalledProgram) -> DMedicResult<UninstallReport> {
    tracing::info!(package = %program.id, "UWP paketi kaldırılıyor");
    // PackageFullName tek tırnak içinde — defansif escape.
    let safe = program.id.replace('\'', "''");
    let script = format!("Remove-AppxPackage -Package '{safe}' -ErrorAction Stop; $LASTEXITCODE");
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
                out.stderr
                    .trim()
                    .lines()
                    .next()
                    .unwrap_or("bilinmeyen hata")
            )
        },
        still_registered: false,
    })
}

#[cfg(test)]
mod tests {
    use super::normalize_msi_uninstall as norm;

    #[test]
    fn msi_install_switch_becomes_uninstall() {
        assert_eq!(
            norm("MsiExec.exe /I{1234-ABCD}"),
            "MsiExec.exe /X{1234-ABCD}"
        );
        assert_eq!(norm("msiexec /i{1234}"), "msiexec /X{1234}");
        assert_eq!(
            norm("MsiExec.exe /I {1234} /qn"),
            "MsiExec.exe /X {1234} /qn"
        );
    }

    #[test]
    fn already_uninstall_or_non_msi_is_untouched() {
        assert_eq!(norm("MsiExec.exe /X{1234}"), "MsiExec.exe /X{1234}");
        assert_eq!(
            norm(r#""C:\Program Files\Acme\unins000.exe" /SILENT"#),
            r#""C:\Program Files\Acme\unins000.exe" /SILENT"#
        );
        // /i ile başlayan başka bir argüman (örn. /iAgree) GUID değilse dokunulmaz.
        assert_eq!(
            norm("MsiExec.exe /X{1} /iAgree"),
            "MsiExec.exe /X{1} /iAgree"
        );
    }
}
