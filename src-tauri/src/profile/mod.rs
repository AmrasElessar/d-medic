//! Profil → action_id eşlemesi. Bkz. Teknik Döküman §5.3.
//!
//! Sıralama: Basit ⊂ Orta ⊂ Agresif. Özel kullanıcı tarafından seçilen.

use serde::Serialize;

use crate::models::ProfileKind;

#[derive(Debug, Clone, Serialize)]
pub struct ProfileDefinition {
    pub kind: ProfileKind,
    pub label_tr: &'static str,
    pub label_en: &'static str,
    pub description_tr: &'static str,
    pub description_en: &'static str,
    pub action_ids: Vec<&'static str>,
}

pub fn definitions() -> Vec<ProfileDefinition> {
    let basic_actions = vec![
        "clean-temp",
        "sfc-repair",
        "dism-restore-health",
        "wu-reset",
        "startup-cleanup",
        "ultimate-performance",
        "enable-windows-re",
    ];

    let mut moderate_actions = basic_actions.clone();
    moderate_actions.extend_from_slice(&[
        "disable-sysmain",
        "disable-telemetry",
        "pagefile-optimize",
        "minimal-visual-effects",
        "limit-search-index",
    ]);

    let mut aggressive_actions = moderate_actions.clone();
    aggressive_actions.extend_from_slice(&[
        "remove-bloatware",
        "disable-vbs",
        "disable-hibernation",
        "switch-dns",
        "defrag-hdd",
    ]);

    vec![
        ProfileDefinition {
            kind: ProfileKind::Basic,
            label_tr: "Basit",
            label_en: "Basic",
            description_tr: "Yalnızca güvenli ve geri alınabilir temizlikler. Yeni nesil donanım için.",
            description_en: "Only safe, reversible cleanups. For modern hardware.",
            action_ids: basic_actions,
        },
        ProfileDefinition {
            kind: ProfileKind::Moderate,
            label_tr: "Orta",
            label_en: "Moderate",
            description_tr: "Çoğu sistem için dengeli. Telemetry ve arka plan servisleri kapatılır.",
            description_en: "Balanced for most systems. Telemetry and bg services disabled.",
            action_ids: moderate_actions,
        },
        ProfileDefinition {
            kind: ProfileKind::Aggressive,
            label_tr: "Agresif",
            label_en: "Aggressive",
            description_tr: "Eski / düşük RAM'li sistemler için. Bloatware ve VBS kalkar — geri alınamaz.",
            description_en: "For old / low-RAM systems. Bloatware + VBS removed — irreversible.",
            action_ids: aggressive_actions,
        },
        ProfileDefinition {
            kind: ProfileKind::Custom,
            label_tr: "Özel",
            label_en: "Custom",
            description_tr: "Kullanıcı seçimine bırakılır.",
            description_en: "User-defined selection.",
            action_ids: vec![],
        },
    ]
}

pub fn for_kind(kind: ProfileKind) -> Option<ProfileDefinition> {
    definitions().into_iter().find(|p| p.kind == kind)
}
