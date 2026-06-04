use std::{collections::HashMap, fs::File, path::Path, sync::Mutex};

use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

use crate::models::PlaybackMode;

pub trait AudioHandle: Send + Sync {
    fn play(&self, path: &Path, volume: f32, mode: PlaybackMode) -> Result<(), String>;
    fn stop_all(&self) -> Result<(), String>;
}

pub struct NullAudioEngine;

impl AudioHandle for NullAudioEngine {
    fn play(&self, _path: &Path, _volume: f32, _mode: PlaybackMode) -> Result<(), String> {
        Err("Audio output is unavailable.".to_string())
    }

    fn stop_all(&self) -> Result<(), String> {
        Ok(())
    }
}

pub struct AudioEngine {
    sink: MixerDeviceSink,
    players: Mutex<HashMap<String, Vec<Player>>>,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let sink = DeviceSinkBuilder::open_default_sink()
            .map_err(|error| format!("Audio output is unavailable: {error}"))?;

        Ok(Self {
            sink,
            players: Mutex::new(HashMap::new()),
        })
    }

    fn play_file(&self, path: &Path, volume: f32, mode: PlaybackMode) -> Result<(), String> {
        let file = File::open(path)
            .map_err(|error| format!("Could not open audio file {}: {error}", path.display()))?;
        let source = Decoder::try_from(file)
            .map_err(|error| format!("Could not decode audio file {}: {error}", path.display()))?;

        let key = path.to_string_lossy().to_string();
        let mut players = self
            .players
            .lock()
            .map_err(|_| "Audio player state is unavailable.".to_string())?;
        let active = players.entry(key).or_default();
        active.retain(|player| !player.empty());

        match mode {
            PlaybackMode::Layer => {}
            PlaybackMode::Restart => {
                for player in active.drain(..) {
                    player.stop();
                }
            }
            PlaybackMode::Single if !active.is_empty() => return Ok(()),
            PlaybackMode::Single => {}
        }

        let player = Player::connect_new(&self.sink.mixer());
        player.set_volume(volume.clamp(0.0, 2.0));
        player.append(source);

        active.push(player);
        Ok(())
    }
}

impl AudioHandle for AudioEngine {
    fn play(&self, path: &Path, volume: f32, mode: PlaybackMode) -> Result<(), String> {
        self.play_file(path, volume, mode)
    }

    fn stop_all(&self) -> Result<(), String> {
        let mut players = self
            .players
            .lock()
            .map_err(|_| "Audio player state is unavailable.".to_string())?;

        for (_, active) in players.drain() {
            for player in active {
                player.stop();
            }
        }

        Ok(())
    }
}
