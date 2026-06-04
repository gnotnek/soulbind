use crate::error::{AppError, AppResult};
use crate::models::{BindingInput, SoundBinding};
use crate::shortcuts::normalize_shortcut;

#[derive(Debug, Default, Clone)]
pub struct BindingCollection {
    bindings: Vec<SoundBinding>,
}

impl BindingCollection {
    pub fn new(bindings: Vec<SoundBinding>) -> Self {
        Self { bindings }
    }

    pub fn all(&self) -> Vec<SoundBinding> {
        self.bindings.clone()
    }

    pub fn get(&self, id: &str) -> Option<SoundBinding> {
        self.bindings
            .iter()
            .find(|binding| binding.id == id)
            .cloned()
    }

    pub fn add(&mut self, id: String, input: BindingInput) -> AppResult<()> {
        let binding = SoundBinding::from_input(id, input)?;
        self.ensure_unique_shortcut(None, &binding.shortcut)?;
        self.bindings.push(binding);
        Ok(())
    }

    pub fn update(&mut self, id: &str, input: BindingInput) -> AppResult<()> {
        let next = SoundBinding::from_input(id.to_string(), input)?;
        self.ensure_unique_shortcut(Some(id), &next.shortcut)?;

        let binding = self
            .bindings
            .iter_mut()
            .find(|binding| binding.id == id)
            .ok_or(AppError::NotFound)?;
        *binding = next;
        Ok(())
    }

    pub fn remove(&mut self, id: &str) -> AppResult<()> {
        let before = self.bindings.len();
        self.bindings.retain(|binding| binding.id != id);

        if self.bindings.len() == before {
            return Err(AppError::NotFound);
        }

        Ok(())
    }

    pub fn duplicate(&mut self, id: &str, new_id: String) -> AppResult<()> {
        let mut copy = self
            .bindings
            .iter()
            .find(|binding| binding.id == id)
            .cloned()
            .ok_or(AppError::NotFound)?;

        copy.id = new_id;
        copy.name = format!("{} Copy", copy.name);
        copy.enabled = false;
        copy.status = "disabled".to_string();
        self.bindings.push(copy);
        Ok(())
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> AppResult<()> {
        let binding = self
            .bindings
            .iter_mut()
            .find(|binding| binding.id == id)
            .ok_or(AppError::NotFound)?;
        binding.enabled = enabled;
        binding.status = if enabled {
            "pending".to_string()
        } else {
            "disabled".to_string()
        };
        Ok(())
    }

    pub fn set_all_enabled(&mut self, enabled: bool) {
        for binding in &mut self.bindings {
            binding.enabled = enabled;
            binding.status = if enabled {
                "pending".to_string()
            } else {
                "disabled".to_string()
            };
        }
    }

    pub fn update_status(&mut self, id: &str, status: impl Into<String>) {
        if let Some(binding) = self.bindings.iter_mut().find(|binding| binding.id == id) {
            binding.status = status.into();
        }
    }

    pub fn as_slice(&self) -> &[SoundBinding] {
        &self.bindings
    }

    fn ensure_unique_shortcut(&self, current_id: Option<&str>, shortcut: &str) -> AppResult<()> {
        let shortcut = normalize_shortcut(shortcut)?;
        let has_duplicate = self.bindings.iter().any(|binding| {
            current_id != Some(binding.id.as_str())
                && normalize_shortcut(&binding.shortcut).ok().as_deref() == Some(shortcut.as_str())
        });

        if has_duplicate {
            return Err(AppError::Validation(
                "Another binding already uses that shortcut.".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::models::PlaybackMode;

    fn input(name: &str, shortcut: &str) -> BindingInput {
        let path = std::env::temp_dir().join(format!(
            "soulbind-binding-test-{}-{}.wav",
            std::process::id(),
            name
        ));
        fs::write(&path, [0_u8]).unwrap();
        BindingInput {
            name: name.to_string(),
            file_path: path,
            shortcut: shortcut.to_string(),
            volume: 1.0,
            mode: PlaybackMode::Layer,
        }
    }

    #[test]
    fn rejects_duplicate_shortcuts_when_adding() {
        let mut collection = BindingCollection::default();
        collection
            .add("a".to_string(), input("one", "CmdOrControl+Shift+1"))
            .unwrap();

        let error = collection
            .add("b".to_string(), input("two", "cmdorcontrol + shift + 1"))
            .unwrap_err();

        assert!(error.to_string().contains("already uses"));
    }

    #[test]
    fn allows_updating_existing_binding_with_same_shortcut() {
        let mut collection = BindingCollection::default();
        collection
            .add("a".to_string(), input("one", "CmdOrControl+Shift+1"))
            .unwrap();

        collection
            .update("a", input("renamed", "CmdOrControl+Shift+1"))
            .unwrap();

        assert_eq!(collection.all()[0].name, "renamed");
    }

    #[test]
    fn remove_reports_missing_binding() {
        let mut collection = BindingCollection::default();
        assert!(matches!(
            collection.remove("missing"),
            Err(AppError::NotFound)
        ));
    }

    #[test]
    fn duplicate_creates_disabled_copy() {
        let mut collection = BindingCollection::default();
        collection
            .add("a".to_string(), input("one", "CmdOrControl+Shift+1"))
            .unwrap();

        collection.duplicate("a", "b".to_string()).unwrap();

        let copy = collection.get("b").unwrap();
        assert_eq!(copy.name, "one Copy");
        assert!(!copy.enabled);
        assert_eq!(copy.status, "disabled");
    }
}
