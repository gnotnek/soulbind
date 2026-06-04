use std::collections::{HashMap, HashSet};

use crate::error::{AppError, AppResult};
use crate::models::SoundBinding;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortcutPlanStatus {
    Disabled,
    Duplicate,
    Invalid(String),
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortcutPlanItem {
    pub binding_id: String,
    pub shortcut: String,
    pub status: ShortcutPlanStatus,
}

pub fn normalize_shortcut(shortcut: &str) -> AppResult<String> {
    let parts = shortcut
        .split('+')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(normalize_part)
        .collect::<Vec<_>>();

    if parts.len() < 2 {
        return Err(AppError::Validation(
            "Shortcut must include at least one modifier and one key.".to_string(),
        ));
    }

    Ok(parts.join("+"))
}

pub fn build_shortcut_plan(bindings: &[SoundBinding]) -> Vec<ShortcutPlanItem> {
    let mut seen = HashSet::new();
    let mut plan = Vec::with_capacity(bindings.len());

    for binding in bindings {
        if !binding.enabled {
            plan.push(ShortcutPlanItem {
                binding_id: binding.id.clone(),
                shortcut: binding.shortcut.clone(),
                status: ShortcutPlanStatus::Disabled,
            });
            continue;
        }

        match normalize_shortcut(&binding.shortcut) {
            Ok(shortcut) if seen.insert(shortcut.clone()) => plan.push(ShortcutPlanItem {
                binding_id: binding.id.clone(),
                shortcut,
                status: ShortcutPlanStatus::Ready,
            }),
            Ok(_) => plan.push(ShortcutPlanItem {
                binding_id: binding.id.clone(),
                shortcut: binding.shortcut.clone(),
                status: ShortcutPlanStatus::Duplicate,
            }),
            Err(error) => plan.push(ShortcutPlanItem {
                binding_id: binding.id.clone(),
                shortcut: binding.shortcut.clone(),
                status: ShortcutPlanStatus::Invalid(error.to_string()),
            }),
        }
    }

    plan
}

pub fn index_insert(index: &mut HashMap<u32, String>, shortcut_id: u32, binding_id: &str) {
    index.insert(shortcut_id, binding_id.to_string());
}

fn normalize_part(part: &str) -> String {
    match part
        .to_ascii_lowercase()
        .replace([' ', '-', '_'], "")
        .as_str()
    {
        "cmdorctrl" | "cmdorcontrol" | "commandorcontrol" => "CmdOrControl".to_string(),
        "cmd" | "command" | "meta" | "super" => "Super".to_string(),
        "ctrl" | "control" => "Control".to_string(),
        "shift" => "Shift".to_string(),
        "alt" | "option" => "Alt".to_string(),
        "space" => "Space".to_string(),
        other if other.len() == 1 => other.to_ascii_uppercase(),
        other => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => first.to_ascii_uppercase().to_string() + chars.as_str(),
                None => String::new(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn binding(id: &str, shortcut: &str, enabled: bool) -> SoundBinding {
        SoundBinding {
            id: id.to_string(),
            name: id.to_string(),
            file_path: PathBuf::from("/tmp/test.wav"),
            shortcut: shortcut.to_string(),
            volume: 1.0,
            enabled,
            mode: Default::default(),
            status: "pending".to_string(),
        }
    }

    #[test]
    fn normalizes_shortcut_aliases_and_spacing() {
        assert_eq!(
            normalize_shortcut(" cmdorctrl + shift + a ").unwrap(),
            "CmdOrControl+Shift+A"
        );
    }

    #[test]
    fn rejects_shortcut_without_modifier() {
        assert!(normalize_shortcut("A").is_err());
    }

    #[test]
    fn marks_disabled_duplicate_invalid_and_ready_bindings() {
        let plan = build_shortcut_plan(&[
            binding("a", "CmdOrControl+Shift+1", true),
            binding("b", "cmdorcontrol+shift+1", true),
            binding("c", "A", true),
            binding("d", "Alt+2", false),
        ]);

        assert_eq!(plan[0].status, ShortcutPlanStatus::Ready);
        assert_eq!(plan[1].status, ShortcutPlanStatus::Duplicate);
        assert!(matches!(plan[2].status, ShortcutPlanStatus::Invalid(_)));
        assert_eq!(plan[3].status, ShortcutPlanStatus::Disabled);
    }
}
