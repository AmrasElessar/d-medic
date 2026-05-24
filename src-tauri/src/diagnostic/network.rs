use async_trait::async_trait;
use serde_json::json;

use super::Check;
use crate::error::DMedicResult;
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel, ScanKind,
};
use crate::ps;

/// #23 — Mevcut DNS sunucusu yavaş (> 100 ms ortalama).
pub struct DnsSpeedCheck;

#[async_trait]
impl Check for DnsSpeedCheck {
    fn id(&self) -> &'static str {
        "dns-speed"
    }
    fn applicable_in(&self, kind: ScanKind) -> bool {
        matches!(kind, ScanKind::Deep)
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        // 3 popüler hedefi ölçüp ortalamayı al — DnsOnly bayrağı cache'i bypass eder.
        let script = "$hosts = 'www.google.com','www.microsoft.com','www.cloudflare.com'\n\
            $sum = 0; $ok = 0\n\
            foreach ($h in $hosts) {\n\
              try {\n\
                $t = Measure-Command { Resolve-DnsName $h -DnsOnly -ErrorAction Stop | Out-Null }\n\
                $sum += $t.TotalMilliseconds; $ok++\n\
              } catch {}\n\
            }\n\
            if ($ok -gt 0) { [int]($sum / $ok) } else { '' }";
        let out = ps::runner::run_script(script).await.ok();
        let Some(out) = out else {
            return Ok(Vec::new());
        };
        let Ok(avg) = out.stdout.trim().parse::<u32>() else {
            return Ok(Vec::new());
        };

        let (priority, label_tr) = if avg >= 300 {
            (Priority::High, "çok yavaş")
        } else if avg >= 150 {
            (Priority::Medium, "yavaş")
        } else if avg >= 80 {
            (Priority::Low, "iyileştirilebilir")
        } else {
            return Ok(Vec::new());
        };

        Ok(vec![Finding {
            id: "dns-speed".to_string(),
            category: Category::Network,
            priority,
            action_type: ActionType::Guided,
            title: LocalizedText::new(
                format!("DNS yanıt süresi {label_tr}: {avg} ms"),
                format!("DNS response time: {avg} ms"),
            ),
            description: LocalizedText::new(
                "Mevcut DNS sunucusu sayfa açılış süresini doğrudan etkiliyor. \
                 Cloudflare (1.1.1.1) veya Google (8.8.8.8) DNS'ye geçmek genellikle \
                 belirgin hızlanma sağlar."
                    .to_string(),
                "Current DNS is slowing page loads. Switching to Cloudflare (1.1.1.1) or \
                 Google (8.8.8.8) usually yields visible speed-up."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::None,
            risk: RiskLevel::Low,
            reboot_required: false,
            action_id: Some("set-fast-dns".to_string()),
            guide_id: None,
            evidence: json!({
                "avg_dns_ms": avg,
                "targets": ["www.google.com", "www.microsoft.com", "www.cloudflare.com"],
            }),
        }])
    }
}
