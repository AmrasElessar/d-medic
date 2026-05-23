use async_trait::async_trait;
use serde_json::json;

use super::Check;
use crate::error::{DMedicError, DMedicResult};
use crate::models::{
    ActionType, Category, EstimatedGain, Finding, LocalizedText, Priority, RiskLevel,
};
use crate::ps;

/// MS default bloat (prefix match — Microsoft.X*.Y* versiyon farklarını yakalar).
pub const KNOWN_BLOATWARE: &[&str] = &[
    "Microsoft.BingNews",
    "Microsoft.BingWeather",
    "Microsoft.GetHelp",
    "Microsoft.Getstarted",
    "Microsoft.MicrosoftOfficeHub",
    "Microsoft.MicrosoftSolitaireCollection",
    "Microsoft.MixedReality.Portal",
    "Microsoft.People",
    "Microsoft.PowerAutomateDesktop",
    "Microsoft.SkypeApp",
    "Microsoft.Todos",
    "Microsoft.WindowsAlarms",
    "Microsoft.WindowsCommunicationsApps",
    "Microsoft.WindowsFeedbackHub",
    "Microsoft.WindowsMaps",
    "Microsoft.WindowsSoundRecorder",
    "Microsoft.Xbox",
    "Microsoft.YourPhone",
    "Microsoft.ZuneMusic",
    "Microsoft.ZuneVideo",
    "Microsoft.GamingApp",
    "Clipchamp.Clipchamp",
    "MicrosoftTeams",
    "MicrosoftCorporationII.QuickAssist",
];

/// 3. taraf yaygın bloat (OEM image'larında sık görülür) — contains match.
pub const THIRD_PARTY_BLOAT_CONTAINS: &[&str] = &[
    "CandyCrush",
    "DisneyMagic",
    "SpotifyAB",
    "HiddenCity",
    "AdobeExpress",
    "Facebook",
    "Twitter",
    "Netflix",
    "TikTok",
];

/// #5 — Bilinen MS/OEM UWP bloatware sayısı.
pub struct UwpBloatwareCheck;

#[async_trait]
impl Check for UwpBloatwareCheck {
    fn id(&self) -> &'static str {
        "uwp-bloatware"
    }

    async fn run(&self) -> DMedicResult<Vec<Finding>> {
        let out = ps::runner::run_script(
            "Get-AppxPackage | Select-Object -ExpandProperty Name",
        )
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "Get-AppxPackage spawn failed");
            DMedicError::PowerShell(format!("uwp list: {e}"))
        })?;

        if out.status != 0 && out.stdout.trim().is_empty() {
            return Ok(Vec::new());
        }

        let packages: Vec<&str> = out
            .stdout
            .lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty())
            .collect();

        let mut hits: Vec<String> = Vec::new();
        for pkg in &packages {
            if KNOWN_BLOATWARE.iter().any(|p| pkg.starts_with(p))
                || THIRD_PARTY_BLOAT_CONTAINS.iter().any(|c| pkg.contains(c))
            {
                hits.push((*pkg).to_string());
            }
        }

        if hits.is_empty() {
            return Ok(Vec::new());
        }

        let count = hits.len();
        let priority = if count >= 10 {
            Priority::High
        } else if count >= 5 {
            Priority::Medium
        } else {
            Priority::Low
        };

        Ok(vec![Finding {
            id: "uwp-bloatware".to_string(),
            category: Category::Performance,
            priority,
            action_type: ActionType::Automatic,
            title: LocalizedText::new(
                format!("{count} bloatware UWP uygulaması tespit edildi"),
                format!("{count} bloatware UWP apps detected"),
            ),
            description: LocalizedText::new(
                "Microsoft varsayılan kurulu uygulamalardan birçoğu nadiren kullanılır \
                 (Solitaire, Xbox, Maps, vb.). Kaldırmak disk alanı ve arka plan ağı \
                 yükünü düşürür. Microsoft Store'dan geri yüklenebilirler."
                    .to_string(),
                "Many default Microsoft apps are rarely used (Solitaire, Xbox, Maps...). \
                 Removing them frees disk and reduces background traffic. Reinstallable \
                 from Microsoft Store."
                    .to_string(),
            ),
            estimated_gain: EstimatedGain::DiskMb {
                value: (count as u32) * 80,
            },
            risk: RiskLevel::Low,
            reboot_required: false,
            action_id: Some("uninstall-uwp-bloat".to_string()),
            guide_id: None,
            evidence: json!({
                "matched_packages": hits,
                "total_installed": packages.len(),
            }),
        }])
    }
}
