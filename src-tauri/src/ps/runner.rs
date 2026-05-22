//! PowerShell tek seferlik komut yürütücüsü.

use std::process::Stdio;
use tokio::process::Command;

use crate::error::{DMedicError, DMedicResult};

/// CREATE_NO_WINDOW — konsol penceresi açılmaz.
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug)]
pub struct PsOutput {
    pub stdout: String,
    pub stderr: String,
    pub status: i32,
}

/// Tek satır script çalıştır, UTF-8 normalize edilmiş stdout/stderr döner.
pub async fn run_script(script: &str) -> DMedicResult<PsOutput> {
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-NonInteractive",
        "-ExecutionPolicy",
        "Bypass",
        "-OutputFormat",
        "Text",
        "-Command",
        script,
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| DMedicError::PowerShell(format!("spawn: {e}")))?;

    Ok(PsOutput {
        stdout: normalize_utf16(&output.stdout),
        stderr: normalize_utf16(&output.stderr),
        status: output.status.code().unwrap_or(-1),
    })
}

/// PowerShell çıktısı UTF-16 LE BOM ile gelebilir (özellikle pwsh 5.1).
/// Heuristik: ilk iki bayt 0xFF 0xFE ise UTF-16 LE varsay.
fn normalize_utf16(bytes: &[u8]) -> String {
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xFE {
        let utf16: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        String::from_utf16_lossy(&utf16)
    } else {
        String::from_utf8_lossy(bytes).into_owned()
    }
}
