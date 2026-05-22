use async_trait::async_trait;
use super::{Action, ActionOutcome};
use crate::error::DMedicResult;

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
            Self::Google     => ("8.8.8.8", "8.8.4.4"),
            Self::AdGuard    => ("94.140.14.14", "94.140.15.15"),
            Self::Quad9      => ("9.9.9.9", "149.112.112.112"),
        }
    }
}

/// DNS sunucusunu değiştir.
/// İlk implementasyonda yalnız Cloudflare uygulanıyor; UI seçimi gelince
/// `DnsProvider` parametre olarak alınacak.
pub struct SwitchDns;

#[async_trait]
impl Action for SwitchDns {
    fn id(&self) -> &'static str { "switch-dns" }
    async fn apply(&self) -> DMedicResult<ActionOutcome> {
        // TODO Faz 2:
        //   Get-NetAdapter | ? Status=Up | Foreach { Set-DnsClientServerAddress -InterfaceIndex $_.ifIndex -ServerAddresses ... }
        //   Clear-DnsClientCache; ipconfig /flushdns
        Ok(ActionOutcome {
            action_id: self.id().to_string(),
            success: true,
            message: "DNS switch stub".into(),
            reboot_required: false,
            details: None,
        })
    }
}
