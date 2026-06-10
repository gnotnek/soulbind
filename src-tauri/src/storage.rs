use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, SoundBinding};

const CONFIG_FILE: &str = "bindings.json";
const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    version: u32,
    #[serde(default)]
    settings: AppSettings,
    bindings: Vec<SoundBinding>,
}

#[derive(Debug, Clone, Default)]
pub struct AppConfig {
    pub settings: AppSettings,
    pub bindings: Vec<SoundBinding>,
}

pub fn default_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("SoulBind")
        .join(CONFIG_FILE)
}

#[cfg(test)]
pub fn load_bindings_from_path(path: PathBuf) -> AppResult<(PathBuf, Vec<SoundBinding>)> {
    load_config_from_path(path).map(|(path, config)| (path, config.bindings))
}

pub fn load_config() -> AppResult<(PathBuf, AppConfig)> {
    let path = default_config_path();
    load_config_from_path(path)
}

pub fn load_config_from_path(path: PathBuf) -> AppResult<(PathBuf, AppConfig)> {
    if !path.exists() {
        return Ok((path, AppConfig::default()));
    }

    let contents = fs::read_to_string(&path).map_err(|error| {
        AppError::Storage(format!("Could not read config {}: {error}", path.display()))
    })?;
    let config = parse_config(&contents).map_err(|error| {
        AppError::Storage(format!(
            "Could not parse config {}: {error}",
            path.display()
        ))
    })?;
    Ok((path, config))
}

#[cfg(test)]
pub fn save_bindings(path: &Path, bindings: &[SoundBinding]) -> AppResult<()> {
    save_config(
        path,
        &AppConfig {
            settings: AppSettings::default(),
            bindings: bindings.to_vec(),
        },
    )
}

pub fn save_config(path: &Path, config: &AppConfig) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::Storage(format!("Could not create config directory: {error}"))
        })?;
    }

    let config = ConfigFile {
        version: CONFIG_VERSION,
        settings: config.settings.clone(),
        bindings: config.bindings.clone(),
    };
    let contents = serde_json::to_string_pretty(&config)
        .map_err(|error| AppError::Storage(format!("Could not serialize config: {error}")))?;
    fs::write(path, contents).map_err(|error| {
        AppError::Storage(format!(
            "Could not write config {}: {error}",
            path.display()
        ))
    })
}

fn parse_config(contents: &str) -> serde_json::Result<AppConfig> {
    let value = serde_json::from_str::<Value>(contents)?;
    if value.is_array() {
        return serde_json::from_value(value).map(|bindings| AppConfig {
            settings: AppSettings::default(),
            bindings,
        });
    }

    serde_json::from_value::<ConfigFile>(value).map(|config| AppConfig {
        settings: config.settings,
        bindings: config.bindings,
    })
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::models::PlaybackMode;

    fn binding(id: &str) -> SoundBinding {
        SoundBinding {
            id: id.to_string(),
            name: "Test".to_string(),
            file_path: PathBuf::from("/tmp/test.wav"),
            shortcut: "CmdOrControl+Shift+1".to_string(),
            volume: 1.0,
            enabled: true,
            mode: PlaybackMode::Layer,
            status: "pending".to_string(),
        }
    }

    #[test]
    fn saves_and_loads_versioned_config() {
        let dir =
            std::env::temp_dir().join(format!("soulbind-storage-test-{}", std::process::id()));
        let path = dir.join("bindings.json");
        save_bindings(&path, &[binding("a")]).unwrap();

        let (_, bindings) = load_bindings_from_path(path).unwrap();

        assert_eq!(bindings.len(), 1);
        assert_eq!(bindings[0].id, "a");
    }

    #[test]
    fn saves_and_loads_audio_settings() {
        let dir = std::env::temp_dir().join(format!(
            "soulbind-storage-settings-test-{}",
            std::process::id()
        ));
        let path = dir.join("bindings.json");
        save_config(
            &path,
            &AppConfig {
                settings: AppSettings {
                    audio: crate::models::AudioSettings {
                        output_device_name: Some("Virtual Cable".to_string()),
                        input_device_name: Some("Mic".to_string()),
                        orchestrator_enabled: true,
                    },
                },
                bindings: vec![binding("a")],
            },
        )
        .unwrap();

        let (_, config) = load_config_from_path(path).unwrap();

        assert_eq!(
            config.settings.audio.output_device_name.as_deref(),
            Some("Virtual Cable")
        );
        assert_eq!(
            config.settings.audio.input_device_name.as_deref(),
            Some("Mic")
        );
        assert!(config.settings.audio.orchestrator_enabled);
        assert_eq!(config.bindings.len(), 1);
    }

    #[test]
    fn loads_legacy_array_config() {
        let dir = std::env::temp_dir().join(format!(
            "soulbind-storage-legacy-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bindings.json");
        fs::write(
            &path,
            serde_json::to_string(&vec![binding("legacy")]).unwrap(),
        )
        .unwrap();

        let (_, bindings) = load_bindings_from_path(path).unwrap();

        assert_eq!(bindings[0].id, "legacy");
    }
}
