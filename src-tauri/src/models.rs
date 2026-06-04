use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlaybackMode {
    #[default]
    Layer,
    Restart,
    Single,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoundBinding {
    pub id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub shortcut: String,
    pub volume: f32,
    pub enabled: bool,
    #[serde(default)]
    pub mode: PlaybackMode,
    #[serde(default = "default_status")]
    pub status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BindingInput {
    pub name: String,
    pub file_path: PathBuf,
    pub shortcut: String,
    pub volume: f32,
    #[serde(default)]
    pub mode: PlaybackMode,
}

impl SoundBinding {
    pub fn from_input(id: String, input: BindingInput) -> AppResult<Self> {
        let name = input.name.trim();
        if name.is_empty() {
            return Err(AppError::Validation("Name is required.".to_string()));
        }

        let shortcut = input.shortcut.trim();
        if shortcut.is_empty() {
            return Err(AppError::Validation("Shortcut is required.".to_string()));
        }

        if !input.file_path.exists() {
            return Err(AppError::Validation(format!(
                "Audio file does not exist: {}",
                input.file_path.display()
            )));
        }

        Ok(Self {
            id,
            name: name.to_string(),
            file_path: input.file_path,
            shortcut: shortcut.to_string(),
            volume: input.volume.clamp(0.0, 2.0),
            enabled: true,
            mode: input.mode,
            status: "pending".to_string(),
        })
    }
}

fn default_status() -> String {
    "pending".to_string()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn input(path: PathBuf) -> BindingInput {
        BindingInput {
            name: " Test ".to_string(),
            file_path: path,
            shortcut: " CmdOrControl+Shift+1 ".to_string(),
            volume: 5.0,
            mode: PlaybackMode::Single,
        }
    }

    #[test]
    fn trims_fields_and_clamps_volume() {
        let path =
            std::env::temp_dir().join(format!("soulbind-model-test-{}.wav", std::process::id()));
        fs::write(&path, [0_u8]).unwrap();

        let binding = SoundBinding::from_input("id".to_string(), input(path)).unwrap();

        assert_eq!(binding.name, "Test");
        assert_eq!(binding.shortcut, "CmdOrControl+Shift+1");
        assert_eq!(binding.volume, 2.0);
        assert_eq!(binding.mode, PlaybackMode::Single);
    }

    #[test]
    fn rejects_missing_audio_file() {
        let path = std::env::temp_dir().join("soulbind-missing-model-test.wav");

        let error = SoundBinding::from_input("id".to_string(), input(path)).unwrap_err();

        assert!(error.to_string().contains("does not exist"));
    }
}
