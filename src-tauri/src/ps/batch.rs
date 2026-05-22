//! Quick scan'in tüm PS sorgularını tek script'te birleştir.

use crate::error::{DMedicError, DMedicResult};

use super::runner;

/// Quick scan için birleştirilmiş PS batch script'i.
/// Çıktı tek bir JSON objesi olacak şekilde derlenir.
pub const QUICK_SCAN_SCRIPT: &str = r#"
$result = @{}
try { $result.ram_gb = [math]::Round(((Get-CimInstance Win32_PhysicalMemory | Measure-Object -Sum Capacity).Sum / 1GB), 2) } catch {}
try { $result.disk_type = (Get-PhysicalDisk | Select-Object -First 1).MediaType } catch {}
try {
    $sys = Get-Volume -DriveLetter C -ErrorAction Stop
    $result.disk_free_gb = [math]::Round(($sys.SizeRemaining / 1GB), 2)
} catch {}
try { $result.vbs_enabled = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control\DeviceGuard').EnableVirtualizationBasedSecurity } catch {}
try { $result.boot_mode = (Get-ItemProperty 'HKLM:\SYSTEM\CurrentControlSet\Control').PEFirmwareType } catch {}
try {
    $services = Get-Service SysMain,WSearch,DiagTrack -ErrorAction Stop |
        Select-Object @{N='name';E={$_.Name}}, @{N='status';E={[string]$_.Status}}
    $result.services = @($services)
} catch {}
try { $result.uwp_apps = @(Get-AppxPackage | Select-Object -ExpandProperty Name) } catch {}
$result | ConvertTo-Json -Depth 5 -Compress
"#;

/// Quick scan script'ini çalıştır, raw JSON string döner.
pub async fn run_quick_scan_script() -> DMedicResult<String> {
    let out = runner::run_script(QUICK_SCAN_SCRIPT).await?;
    if out.status != 0 && out.stdout.trim().is_empty() {
        return Err(DMedicError::PowerShell(format!(
            "quick batch exit {}: {}",
            out.status,
            out.stderr.trim()
        )));
    }
    Ok(out.stdout)
}
