use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::{AppError, AppResult};
use crate::models::SoundBinding;

const CONFIG_FILE: &str = "bindings.json";
const CONFIG_VERSION: u32 = 1;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    version: u32,
    bindings: Vec<SoundBinding>,
}

pub fn default_config_path() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("SoulBind")
        .join(CONFIG_FILE)
}

pub fn load_bindings() -> AppResult<(PathBuf, Vec<SoundBinding>)> {
    let path = default_config_path();
    load_bindings_from_path(path)
}

pub fn load_bindings_from_path(path: PathBuf) -> AppResult<(PathBuf, Vec<SoundBinding>)> {
    if !path.exists() {
        return Ok((path, Vec::new()));
    }

    let contents = fs::read_to_string(&path).map_err(|error| {
        AppError::Storage(format!("Could not read config {}: {error}", path.display()))
    })?;
    let bindings = parse_config(&contents).map_err(|error| {
        AppError::Storage(format!(
            "Could not parse config {}: {error}",
            path.display()
        ))
    })?;
    Ok((path, bindings))
}

pub fn save_bindings(path: &Path, bindings: &[SoundBinding]) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| {
            AppError::Storage(format!("Could not create config directory: {error}"))
        })?;
    }

    let config = ConfigFile {
        version: CONFIG_VERSION,
        bindings: bindings.to_vec(),
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

fn parse_config(contents: &str) -> serde_json::Result<Vec<SoundBinding>> {
    let value = serde_json::from_str::<Value>(contents)?;
    if value.is_array() {
        return serde_json::from_value(value);
    }

    serde_json::from_value::<ConfigFile>(value).map(|config| config.bindings)
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
