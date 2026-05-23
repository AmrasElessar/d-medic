//! Profil → action_id eşlemesi. Bkz. Teknik Döküman §5.3.
//!
//! İçe alma kuralı: Basic ⊂ Moderate ⊂ Aggressive. Custom kullanıcı seçimi.
//!
//! action_id'ler `action::registry()` ile birebir eşleşmelidir; uyumsuzluk
//! testlerle yakalanır (`tests::all_profile_actions_exist`).

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
    // Basic: yalnız geri alınabilir + zararsız işlemler.
    let basic_actions = vec![
        "clean-temp",
        "sfc-repair",
        "dism-restore-health",
        "reset-windows-update",
        "startup-cleanup",
        "set-high-performance-plan",
        "enable-windows-re",
    ];

    // Moderate: Basic + servis kapatma + pagefile + görsel efekt.
    let mut moderate_actions = basic_actions.clone();
    moderate_actions.extend_from_slice(&[
        "disable-sysmain",
        "disable-telemetry",
        "pagefile-optimize",
        "set-visual-effects-performance",
        "disable-wsearch",
    ]);

    // Aggressive: Moderate + bloatware kaldırma (KALICI) + VBS off + hibernation + DNS + defrag.
    let mut aggressive_actions = moderate_actions.clone();
    aggressive_actions.extend_from_slice(&[
        "uninstall-uwp-bloat",
        "disable-vbs",
        "disable-hibernation",
        "set-fast-dns",
        "defrag-system",
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action;
    use std::collections::HashSet;

    /// Profil'deki her action_id, action::registry()'de KAYITLI olmalı.
    /// Aksi halde UI "Düzelt" → NotFound hatası verir.
    #[test]
    fn all_profile_actions_exist_in_registry() {
        let registered: HashSet<&'static str> =
            action::registry().iter().map(|a| a.id()).collect();
        for prof in definitions() {
            for action_id in &prof.action_ids {
                assert!(
                    registered.contains(action_id),
                    "Profile {:?}: action_id '{}' action::registry()'de yok",
                    prof.kind,
                    action_id
                );
            }
        }
    }

    /// Basic ⊂ Moderate ⊂ Aggressive invaryantı.
    #[test]
    fn basic_subset_moderate_subset_aggressive() {
        let defs = definitions();
        let basic: HashSet<_> = defs
            .iter()
            .find(|p| p.kind == ProfileKind::Basic)
            .unwrap()
            .action_ids
            .iter()
            .collect();
        let moderate: HashSet<_> = defs
            .iter()
            .find(|p| p.kind == ProfileKind::Moderate)
            .unwrap()
            .action_ids
            .iter()
            .collect();
        let aggressive: HashSet<_> = defs
            .iter()
            .find(|p| p.kind == ProfileKind::Aggressive)
            .unwrap()
            .action_ids
            .iter()
            .collect();
        assert!(
            basic.is_subset(&moderate),
            "Basic ⊄ Moderate. Eksik: {:?}",
            basic.difference(&moderate).collect::<Vec<_>>()
        );
        assert!(
            moderate.is_subset(&aggressive),
            "Moderate ⊄ Aggressive. Eksik: {:?}",
            moderate.difference(&aggressive).collect::<Vec<_>>()
        );
    }
}
