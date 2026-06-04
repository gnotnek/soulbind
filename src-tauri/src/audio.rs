use std::{collections::HashMap, fs::File, path::Path, sync::Mutex};

use cpal::traits::{DeviceTrait, HostTrait};
use rodio::{Decoder, DeviceSinkBuilder, MixerDeviceSink, Player};

use crate::models::{AudioOutputDevice, PlaybackMode};

pub trait AudioHandle: Send + Sync {
    fn play(&self, path: &Path, volume: f32, mode: PlaybackMode) -> Result<(), String>;
    fn stop_all(&self) -> Result<(), String>;
    fn output_devices(&self) -> Result<Vec<AudioOutputDevice>, String>;
    fn set_output_device(&self, device_name: Option<String>) -> Result<(), String>;
    fn selected_output_device_name(&self) -> Option<String>;
}

pub struct NullAudioEngine;

impl AudioHandle for NullAudioEngine {
    fn play(&self, _path: &Path, _volume: f32, _mode: PlaybackMode) -> Result<(), String> {
        Err("Audio output is unavailable.".to_string())
    }

    fn stop_all(&self) -> Result<(), String> {
        Ok(())
    }

    fn output_devices(&self) -> Result<Vec<AudioOutputDevice>, String> {
        list_output_devices(None)
    }

    fn set_output_device(&self, _device_name: Option<String>) -> Result<(), String> {
        Err("Audio output is unavailable.".to_string())
    }

    fn selected_output_device_name(&self) -> Option<String> {
        None
    }
}

pub struct AudioEngine {
    sink: Mutex<MixerDeviceSink>,
    players: Mutex<HashMap<String, Vec<Player>>>,
    selected_device_name: Mutex<Option<String>>,
}

impl AudioEngine {
    pub fn new(selected_device_name: Option<String>) -> Result<Self, String> {
        let sink = open_sink(selected_device_name.as_deref())?;

        Ok(Self {
            sink: Mutex::new(sink),
            players: Mutex::new(HashMap::new()),
            selected_device_name: Mutex::new(selected_device_name),
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

        let sink = self
            .sink
            .lock()
            .map_err(|_| "Audio output state is unavailable.".to_string())?;
        let player = Player::connect_new(sink.mixer());
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

    fn output_devices(&self) -> Result<Vec<AudioOutputDevice>, String> {
        list_output_devices(self.selected_output_device_name().as_deref())
    }

    fn set_output_device(&self, device_name: Option<String>) -> Result<(), String> {
        let sink = open_sink(device_name.as_deref())?;

        self.stop_all()?;
        *self
            .sink
            .lock()
            .map_err(|_| "Audio output state is unavailable.".to_string())? = sink;
        *self
            .selected_device_name
            .lock()
            .map_err(|_| "Audio output selection is unavailable.".to_string())? = device_name;
        Ok(())
    }

    fn selected_output_device_name(&self) -> Option<String> {
        self.selected_device_name
            .lock()
            .ok()
            .and_then(|name| name.clone())
    }
}

fn open_sink(device_name: Option<&str>) -> Result<MixerDeviceSink, String> {
    match device_name {
        Some(name) if !name.trim().is_empty() => open_named_sink(name),
        _ => DeviceSinkBuilder::open_default_sink()
            .map_err(|error| format!("Audio output is unavailable: {error}")),
    }
}

fn open_named_sink(device_name: &str) -> Result<MixerDeviceSink, String> {
    let host = cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|error| format!("Could not list audio output devices: {error}"))?;
    let device = devices
        .filter_map(|device| {
            let name = device.description().ok()?.name().to_string();
            (name == device_name).then_some(device)
        })
        .next()
        .ok_or_else(|| format!("Audio output device was not found: {device_name}"))?;

    DeviceSinkBuilder::from_device(device)
        .and_then(|builder| builder.open_sink_or_fallback())
        .map_err(|error| format!("Could not open audio output device {device_name}: {error}"))
}

pub fn list_output_devices(selected_name: Option<&str>) -> Result<Vec<AudioOutputDevice>, String> {
    let host = cpal::default_host();
    let default_name = host.default_output_device().and_then(|device| {
        device
            .description()
            .ok()
            .map(|desc| desc.name().to_string())
    });
    let devices = host
        .output_devices()
        .map_err(|error| format!("Could not list audio output devices: {error}"))?;

    let mut output = Vec::new();
    output.push(AudioOutputDevice {
        id: String::new(),
        name: "System default".to_string(),
        is_default: selected_name.is_none(),
        is_selected: selected_name.is_none(),
    });

    for name in devices.filter_map(|device| {
        device
            .description()
            .ok()
            .map(|desc| desc.name().to_string())
    }) {
        if output.iter().any(|device| device.id == name) {
            continue;
        }

        output.push(AudioOutputDevice {
            id: name.clone(),
            is_default: default_name.as_deref() == Some(name.as_str()),
            is_selected: selected_name == Some(name.as_str()),
            name,
        });
    }

    Ok(output)
}
