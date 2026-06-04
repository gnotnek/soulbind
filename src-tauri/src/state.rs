use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use crate::audio::{AudioEngine, AudioHandle, NullAudioEngine};
use crate::bindings::BindingCollection;
use crate::error::{AppError, AppResult};
use crate::models::{BindingInput, SoundBinding};
use crate::storage;

pub struct AppState {
    bindings: Mutex<BindingCollection>,
    shortcut_index: Mutex<HashMap<u32, String>>,
    audio: Box<dyn AudioHandle>,
    config_path: PathBuf,
}

impl AppState {
    pub fn load() -> Self {
        let (config_path, bindings) = storage::load_bindings().unwrap_or_else(|_| {
            let config_path = storage::default_config_path();
            (config_path, Vec::new())
        });

        let audio: Box<dyn AudioHandle> = match AudioEngine::new() {
            Ok(engine) => Box::new(engine),
            Err(_) => Box::new(NullAudioEngine),
        };

        Self::new(config_path, bindings, audio)
    }

    pub fn new(
        config_path: PathBuf,
        bindings: Vec<SoundBinding>,
        audio: Box<dyn AudioHandle>,
    ) -> Self {
        Self {
            bindings: Mutex::new(BindingCollection::new(bindings)),
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
        storage::save_bindings(&self.config_path, bindings.as_slice())
    }

    fn lock_bindings(&self) -> AppResult<std::sync::MutexGuard<'_, BindingCollection>> {
        self.bindings
            .lock()
            .map_err(|_| AppError::State("Binding state is unavailable.".to_string()))
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
        let state = AppState::new(dir.join("bindings.json"), Vec::new(), Box::new(fake));

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
            Box::new(fake),
        );

        state.stop_all().unwrap();

        assert_eq!(*stopped.lock().unwrap(), 1);
    }
}
