use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::audio::{AudioEngine, AudioHandle, NullAudioEngine};
use crate::bindings::BindingCollection;
use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, AudioInputDevice, AudioOutputDevice, BindingInput, SoundBinding};
use crate::storage;

pub struct AppState {
    bindings: Mutex<BindingCollection>,
    settings: Mutex<AppSettings>,
    shortcut_index: Mutex<HashMap<u32, String>>,
    audio: Box<dyn AudioHandle>,
    config_path: PathBuf,
}

impl AppState {
    pub fn load() -> Self {
        let (config_path, mut config) = storage::load_config().unwrap_or_else(|_| {
            let config_path = storage::default_config_path();
            (config_path, storage::AppConfig::default())
        });

        let audio: Box<dyn AudioHandle> = match AudioEngine::new(
            config.settings.audio.output_device_name.clone(),
            config.settings.audio.input_device_name.clone(),
            config.settings.audio.orchestrator_enabled,
        ) {
            Ok(engine) => {
                config.settings.audio.orchestrator_enabled = engine.orchestrator_enabled();
                Box::new(engine)
            }
            Err(_) => Box::new(NullAudioEngine),
        };

        Self::new(config_path, config.bindings, config.settings, audio)
    }

    pub fn new(
        config_path: PathBuf,
        bindings: Vec<SoundBinding>,
        settings: AppSettings,
        audio: Box<dyn AudioHandle>,
    ) -> Self {
        Self {
            bindings: Mutex::new(BindingCollection::new(bindings)),
            settings: Mutex::new(settings),
            shortcut_index: Mutex::new(HashMap::new()),
            audio,
            config_path,
        }
    }

    pub fn bindings(&self) -> AppResult<Vec<SoundBinding>> {
        Ok(self.lock_bindings()?.all())
    }

    pub fn add_binding(&self, id: String, input: BindingInput) -> AppResult<Vec<SoundBinding>> {
        self.lock_bindings()?.add(id, input)?;
        self.save()?;
        self.bindings()
    }

    pub fn update_binding(&self, id: &str, input: BindingInput) -> AppResult<Vec<SoundBinding>> {
        self.lock_bindings()?.update(id, input)?;
        self.save()?;
        self.bindings()
    }

    pub fn delete_binding(&self, id: &str) -> AppResult<Vec<SoundBinding>> {
        self.lock_bindings()?.remove(id)?;
        self.save()?;
        self.bindings()
    }

    pub fn duplicate_binding(&self, id: &str, new_id: String) -> AppResult<Vec<SoundBinding>> {
        self.lock_bindings()?.duplicate(id, new_id)?;
        self.save()?;
        self.bindings()
    }

    pub fn set_binding_enabled(&self, id: &str, enabled: bool) -> AppResult<Vec<SoundBinding>> {
        self.lock_bindings()?.set_enabled(id, enabled)?;
        self.save()?;
        self.bindings()
    }

    pub fn set_all_enabled(&self, enabled: bool) -> AppResult<Vec<SoundBinding>> {
        self.lock_bindings()?.set_all_enabled(enabled);
        self.save()?;
        self.bindings()
    }

    pub fn play_binding(&self, id: &str) -> AppResult<()> {
        let binding = self.lock_bindings()?.get(id).ok_or(AppError::NotFound)?;
        if !binding.enabled {
            return Err(AppError::Validation("Binding is disabled.".to_string()));
        }

        self.audio
            .play(&binding.file_path, binding.volume, binding.mode)
            .map_err(AppError::Audio)
    }

    pub fn stop_all(&self) -> AppResult<()> {
        self.audio.stop_all().map_err(AppError::Audio)
    }

    pub fn output_devices(&self) -> AppResult<Vec<AudioOutputDevice>> {
        self.audio.output_devices().map_err(AppError::Audio)
    }

    pub fn input_devices(&self) -> AppResult<Vec<AudioInputDevice>> {
        self.audio.input_devices().map_err(AppError::Audio)
    }

    pub fn settings(&self) -> AppResult<AppSettings> {
        Ok(self.lock_settings()?.clone())
    }

    pub fn set_output_device(&self, device_name: Option<String>) -> AppResult<AppSettings> {
        let device_name = device_name.filter(|name| !name.trim().is_empty());
        self.audio
            .set_output_device(device_name.clone())
            .map_err(AppError::Audio)?;

        {
            let mut settings = self.lock_settings()?;
            settings.audio.output_device_name = device_name;
        }

        self.save()?;
        self.settings()
    }

    pub fn set_input_device(&self, device_name: Option<String>) -> AppResult<AppSettings> {
        let device_name = device_name.filter(|name| !name.trim().is_empty());
        self.audio
            .set_input_device(device_name.clone())
            .map_err(AppError::Audio)?;

        {
            let mut settings = self.lock_settings()?;
            settings.audio.input_device_name = device_name;
        }

        self.save()?;
        self.settings()
    }

    pub fn set_orchestrator_enabled(&self, enabled: bool) -> AppResult<AppSettings> {
        self.audio
            .set_orchestrator_enabled(enabled)
            .map_err(AppError::Audio)?;

        {
            let mut settings = self.lock_settings()?;
            settings.audio.orchestrator_enabled = enabled;
        }

        self.save()?;
        self.settings()
    }

    pub fn shortcut_binding_id(&self, shortcut_id: u32) -> Option<String> {
        self.shortcut_index
            .lock()
            .ok()
            .and_then(|index| index.get(&shortcut_id).cloned())
    }

    pub fn replace_shortcut_index(&self, index: HashMap<u32, String>) -> AppResult<()> {
        *self
            .shortcut_index
            .lock()
            .map_err(|_| AppError::State("Shortcut state is unavailable.".to_string()))? = index;
        Ok(())
    }

    pub fn with_bindings_mut<T>(
        &self,
        f: impl FnOnce(&mut BindingCollection) -> T,
    ) -> AppResult<T> {
        let mut bindings = self.lock_bindings()?;
        Ok(f(&mut bindings))
    }

    pub fn save(&self) -> AppResult<()> {
        let bindings = self.lock_bindings()?;
        let settings = self.lock_settings()?;
        storage::save_config(
            &self.config_path,
            &storage::AppConfig {
                settings: settings.clone(),
                bindings: bindings.as_slice().to_vec(),
            },
        )
    }

    fn lock_bindings(&self) -> AppResult<std::sync::MutexGuard<'_, BindingCollection>> {
        self.bindings
            .lock()
            .map_err(|_| AppError::State("Binding state is unavailable.".to_string()))
    }

    fn lock_settings(&self) -> AppResult<std::sync::MutexGuard<'_, AppSettings>> {
        self.settings
            .lock()
            .map_err(|_| AppError::State("Settings state is unavailable.".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::models::PlaybackMode;

    #[derive(Default)]
    struct FakeAudio {
        played: Arc<Mutex<Vec<(PathBuf, f32, PlaybackMode)>>>,
        stopped: Arc<Mutex<usize>>,
        selected_output: Arc<Mutex<Option<String>>>,
    }

    impl AudioHandle for FakeAudio {
        fn play(
            &self,
            path: &std::path::Path,
            volume: f32,
            mode: PlaybackMode,
        ) -> Result<(), String> {
            self.played
                .lock()
                .unwrap()
                .push((path.to_path_buf(), volume, mode));
            Ok(())
        }

        fn stop_all(&self) -> Result<(), String> {
            *self.stopped.lock().unwrap() += 1;
            Ok(())
        }

        fn output_devices(&self) -> Result<Vec<AudioOutputDevice>, String> {
            let selected = self.selected_output.lock().unwrap().clone();
            Ok(vec![AudioOutputDevice {
                id: "Virtual Cable".to_string(),
                name: "Virtual Cable".to_string(),
                is_default: false,
                is_selected: selected.as_deref() == Some("Virtual Cable"),
            }])
        }

        fn input_devices(&self) -> Result<Vec<AudioInputDevice>, String> {
            Ok(vec![AudioInputDevice {
                id: "Mic".to_string(),
                name: "Mic".to_string(),
                is_default: false,
                is_selected: false,
            }])
        }

        fn set_output_device(&self, device_name: Option<String>) -> Result<(), String> {
            *self.selected_output.lock().unwrap() = device_name;
            Ok(())
        }

        fn set_input_device(&self, _device_name: Option<String>) -> Result<(), String> {
            Ok(())
        }

        fn set_orchestrator_enabled(&self, _enabled: bool) -> Result<(), String> {
            Ok(())
        }

        fn selected_output_device_name(&self) -> Option<String> {
            self.selected_output.lock().unwrap().clone()
        }

        fn selected_input_device_name(&self) -> Option<String> {
            None
        }

        fn orchestrator_enabled(&self) -> bool {
            false
        }
    }

    fn input(path: PathBuf) -> BindingInput {
        BindingInput {
            name: "Test".to_string(),
            file_path: path,
            shortcut: "CmdOrControl+Shift+1".to_string(),
            volume: 1.5,
            mode: PlaybackMode::Restart,
        }
    }

    #[test]
    fn plays_existing_binding_through_audio_handle() {
        let dir = std::env::temp_dir().join(format!("soulbind-state-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        let audio_path = dir.join("sound.wav");
        fs::write(&audio_path, [0_u8]).unwrap();

        let fake = FakeAudio::default();
        let played = fake.played.clone();
        let state = AppState::new(
            dir.join("bindings.json"),
            Vec::new(),
            AppSettings::default(),
            Box::new(fake),
        );

        state
            .add_binding("a".to_string(), input(audio_path.clone()))
            .unwrap();
        state.play_binding("a").unwrap();

        let calls = played.lock().unwrap();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].0, audio_path);
        assert_eq!(calls[0].1, 1.5);
        assert_eq!(calls[0].2, PlaybackMode::Restart);
    }

    #[test]
    fn stop_all_delegates_to_audio_handle() {
        let fake = FakeAudio::default();
        let stopped = fake.stopped.clone();
        let state = AppState::new(
            PathBuf::from("/tmp/no-write.json"),
            Vec::new(),
            AppSettings::default(),
            Box::new(fake),
        );

        state.stop_all().unwrap();

        assert_eq!(*stopped.lock().unwrap(), 1);
    }

    #[test]
    fn set_output_device_updates_and_persists_settings() {
        let dir = std::env::temp_dir().join(format!(
            "soulbind-output-settings-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();

        let fake = FakeAudio::default();
        let state = AppState::new(
            dir.join("bindings.json"),
            Vec::new(),
            AppSettings::default(),
            Box::new(fake),
        );

        let settings = state
            .set_output_device(Some("Virtual Cable".to_string()))
            .unwrap();

        assert_eq!(
            settings.audio.output_device_name.as_deref(),
            Some("Virtual Cable")
        );
    }

    #[test]
    fn set_input_device_updates_and_persists_settings() {
        let dir = std::env::temp_dir().join(format!(
            "soulbind-input-settings-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();

        let state = AppState::new(
            dir.join("bindings.json"),
            Vec::new(),
            AppSettings::default(),
            Box::new(FakeAudio::default()),
        );

        let settings = state.set_input_device(Some("Mic".to_string())).unwrap();

        assert_eq!(settings.audio.input_device_name.as_deref(), Some("Mic"));
    }

    #[test]
    fn set_orchestrator_enabled_updates_and_persists_settings() {
        let dir = std::env::temp_dir().join(format!(
            "soulbind-orchestrator-settings-test-{}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).unwrap();

        let state = AppState::new(
            dir.join("bindings.json"),
            Vec::new(),
            AppSettings::default(),
            Box::new(FakeAudio::default()),
        );

        let settings = state.set_orchestrator_enabled(true).unwrap();

        assert!(settings.audio.orchestrator_enabled);
    }
}
