use async_trait::async_trait;
use serde_json::json;

use super::{Action, ActionOutcome};
use crate::error::DMedicResult;
use crate::ps;

#[derive(Debug, Clone, Copy)]
pub enum DnsProvider {
    Cloudflare,
    Google,
    AdGuard,
    Quad9,
}

impl DnsProvider {
    pub fn servers(&self) -> (&'static str, &'static str) {
        match self {
            Self::Cloudflare => ("1.1.1.1", "1.0.0.1"),
            Self::Google => ("8.8.8.8", "8.8.4.4"),
            Self::AdGuard => ("94.140.14.14", "94.140.15.15"),
            Self::Quad9 => ("9.9.9.9", "149.112.112.112"),
        }
    }
}

/// Aktif ağ adaptörlerinin DNS sunucusunu Cloudflare'e geçir.
/// İleride UI'dan provider seçimi gelince DnsProvider parametre olacak.
pub struct SwitchDns;

#[async_trait]
impl Action for SwitchDns {
    fn id(&self) -> &'static str {
        // Check tarafı bu id ile arıyor.
        "set-fast-dns"
    }

    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        let (primary, secondary) = DnsProvider::Cloudflare.servers();
        let script = format!(
            "$adapters = Get-NetAdapter | Where-Object {{ $_.Status -eq 'Up' }}\n\
             $results = @()\n\
             foreach ($a in $adapters) {{\n\
               try {{\n\
                 Set-DnsClientServerAddress -InterfaceIndex $a.ifIndex -ServerAddresses ('{primary}','{secondary}') -ErrorAction Stop\n\
                 $results += \"$($a.Name):ok\"\n\
               }} catch {{ $results += \"$($a.Name):$($_.Exception.Message)\" }}\n\
             }}\n\
             try {{ Clear-DnsClientCache -ErrorAction SilentlyContinue }} catch {{}}\n\
             $results -join ' | '"
        );
        let out = ps::runner::run_script(&script).await?;
        let success = out.status == 0 && !out.stdout.contains("Access is denied");
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success,
            message: if success {
                format!(
                    "DNS aktif adaptörlerde {primary}/{secondary} olarak ayarlandı; cache temizlendi."
                )
            } else {
                "DNS değiştirilemedi (yönetici yetkisi gerek olabilir).".to_string()
            },
            reboot_required: false,
            details: Some(json!({
                "primary": primary,
                "secondary": secondary,
                "stdout": out.stdout.trim(),
                "stderr": out.stderr.trim(),
            })),
        })
    }
}
