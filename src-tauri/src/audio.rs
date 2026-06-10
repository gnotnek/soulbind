use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    path::Path,
    sync::{Arc, Mutex},
};

use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Device, SampleFormat, Stream, SupportedStreamConfig,
};
use rodio::{
    ChannelCount, Decoder, DeviceSinkBuilder, MixerDeviceSink, Player,
    SampleRate as RodioSampleRate, Source,
};

use crate::models::{AudioInputDevice, AudioOutputDevice, PlaybackMode};

pub trait AudioHandle: Send + Sync {
    fn play(&self, path: &Path, volume: f32, mode: PlaybackMode) -> Result<(), String>;
    fn stop_all(&self) -> Result<(), String>;
    fn output_devices(&self) -> Result<Vec<AudioOutputDevice>, String>;
    fn input_devices(&self) -> Result<Vec<AudioInputDevice>, String>;
    fn set_output_device(&self, device_name: Option<String>) -> Result<(), String>;
    fn set_input_device(&self, device_name: Option<String>) -> Result<(), String>;
    fn set_orchestrator_enabled(&self, enabled: bool) -> Result<(), String>;
    fn selected_output_device_name(&self) -> Option<String>;
    fn selected_input_device_name(&self) -> Option<String>;
    fn orchestrator_enabled(&self) -> bool;
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

    fn input_devices(&self) -> Result<Vec<AudioInputDevice>, String> {
        list_input_devices(None)
    }

    fn set_output_device(&self, _device_name: Option<String>) -> Result<(), String> {
        Err("Audio output is unavailable.".to_string())
    }

    fn set_input_device(&self, _device_name: Option<String>) -> Result<(), String> {
        Err("Audio input is unavailable.".to_string())
    }

    fn set_orchestrator_enabled(&self, _enabled: bool) -> Result<(), String> {
        Err("Audio routing is unavailable.".to_string())
    }

    fn selected_output_device_name(&self) -> Option<String> {
        None
    }

    fn selected_input_device_name(&self) -> Option<String> {
        None
    }

    fn orchestrator_enabled(&self) -> bool {
        false
    }
}

pub struct AudioEngine {
    sink: Mutex<MixerDeviceSink>,
    mic_bridge: Mutex<Option<MicBridge>>,
    players: Mutex<HashMap<String, Vec<Player>>>,
    selected_output_device_name: Mutex<Option<String>>,
    selected_input_device_name: Mutex<Option<String>>,
    orchestrator_enabled: Mutex<bool>,
}

impl AudioEngine {
    pub fn new(
        selected_output_device_name: Option<String>,
        selected_input_device_name: Option<String>,
        orchestrator_enabled: bool,
    ) -> Result<Self, String> {
        let sink = open_sink(selected_output_device_name.as_deref())?;
        let bridge = if orchestrator_enabled {
            start_mic_bridge(selected_input_device_name.as_deref(), &sink).ok()
        } else {
            None
        };
        let orchestrator_enabled = bridge.is_some();

        Ok(Self {
            sink: Mutex::new(sink),
            mic_bridge: Mutex::new(bridge),
            players: Mutex::new(HashMap::new()),
            selected_output_device_name: Mutex::new(selected_output_device_name),
            selected_input_device_name: Mutex::new(selected_input_device_name),
            orchestrator_enabled: Mutex::new(orchestrator_enabled),
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

    fn input_devices(&self) -> Result<Vec<AudioInputDevice>, String> {
        list_input_devices(self.selected_input_device_name().as_deref())
    }

    fn set_output_device(&self, device_name: Option<String>) -> Result<(), String> {
        let sink = open_sink(device_name.as_deref())?;

        self.stop_all()?;
        *self
            .sink
            .lock()
            .map_err(|_| "Audio output state is unavailable.".to_string())? = sink;
        *self
            .selected_output_device_name
            .lock()
            .map_err(|_| "Audio output selection is unavailable.".to_string())? = device_name;
        self.restart_mic_bridge_if_enabled()?;
        Ok(())
    }

    fn set_input_device(&self, device_name: Option<String>) -> Result<(), String> {
        if let Some(name) = device_name.as_deref() {
            open_named_input_device(name)?;
        }

        *self
            .selected_input_device_name
            .lock()
            .map_err(|_| "Audio input selection is unavailable.".to_string())? = device_name;
        self.restart_mic_bridge_if_enabled()
    }

    fn set_orchestrator_enabled(&self, enabled: bool) -> Result<(), String> {
        if enabled {
            let input_name = self.selected_input_device_name();
            let sink = self
                .sink
                .lock()
                .map_err(|_| "Audio output state is unavailable.".to_string())?;
            let bridge = start_mic_bridge(input_name.as_deref(), &sink)?;
            *self
                .mic_bridge
                .lock()
                .map_err(|_| "Audio routing state is unavailable.".to_string())? = Some(bridge);
            *self
                .orchestrator_enabled
                .lock()
                .map_err(|_| "Audio routing state is unavailable.".to_string())? = true;
            Ok(())
        } else {
            *self
                .mic_bridge
                .lock()
                .map_err(|_| "Audio routing state is unavailable.".to_string())? = None;
            *self
                .orchestrator_enabled
                .lock()
                .map_err(|_| "Audio routing state is unavailable.".to_string())? = false;
            Ok(())
        }
    }

    fn selected_output_device_name(&self) -> Option<String> {
        self.selected_output_device_name
            .lock()
            .ok()
            .and_then(|name| name.clone())
    }

    fn selected_input_device_name(&self) -> Option<String> {
        self.selected_input_device_name
            .lock()
            .ok()
            .and_then(|name| name.clone())
    }

    fn orchestrator_enabled(&self) -> bool {
        self.orchestrator_enabled
            .lock()
            .is_ok_and(|enabled| *enabled)
    }
}

impl AudioEngine {
    fn restart_mic_bridge_if_enabled(&self) -> Result<(), String> {
        if !self.orchestrator_enabled() {
            return Ok(());
        }

        let input_name = self.selected_input_device_name();
        let sink = self
            .sink
            .lock()
            .map_err(|_| "Audio output state is unavailable.".to_string())?;
        let bridge = start_mic_bridge(input_name.as_deref(), &sink)?;
        *self
            .mic_bridge
            .lock()
            .map_err(|_| "Audio routing state is unavailable.".to_string())? = Some(bridge);
        Ok(())
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
    let device = open_named_output_device(device_name)?;

    DeviceSinkBuilder::from_device(device)
        .and_then(|builder| builder.open_sink_or_fallback())
        .map_err(|error| format!("Could not open audio output device {device_name}: {error}"))
}

fn open_named_output_device(device_name: &str) -> Result<Device, String> {
    let host = cpal::default_host();
    let devices = host
        .output_devices()
        .map_err(|error| format!("Could not list audio output devices: {error}"))?;
    find_device_by_name(devices, device_name)
        .ok_or_else(|| format!("Audio output device was not found: {device_name}"))
}

fn open_named_input_device(device_name: &str) -> Result<Device, String> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .map_err(|error| format!("Could not list audio input devices: {error}"))?;
    find_device_by_name(devices, device_name)
        .ok_or_else(|| format!("Audio input device was not found: {device_name}"))
}

fn find_device_by_name(devices: impl Iterator<Item = Device>, device_name: &str) -> Option<Device> {
    devices
        .filter_map(|device| {
            let name = device.description().ok()?.name().to_string();
            (name == device_name).then_some(device)
        })
        .next()
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

pub fn list_input_devices(selected_name: Option<&str>) -> Result<Vec<AudioInputDevice>, String> {
    let host = cpal::default_host();
    let default_name = host.default_input_device().and_then(|device| {
        device
            .description()
            .ok()
            .map(|desc| desc.name().to_string())
    });
    let devices = host
        .input_devices()
        .map_err(|error| format!("Could not list audio input devices: {error}"))?;

    let mut input = Vec::new();
    input.push(AudioInputDevice {
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
        if input.iter().any(|device| device.id == name) {
            continue;
        }

        input.push(AudioInputDevice {
            id: name.clone(),
            is_default: default_name.as_deref() == Some(name.as_str()),
            is_selected: selected_name == Some(name.as_str()),
            name,
        });
    }

    Ok(input)
}

struct MicBridge {
    _input_stream: Stream,
    _player: Player,
}

fn start_mic_bridge(input_name: Option<&str>, sink: &MixerDeviceSink) -> Result<MicBridge, String> {
    let host = cpal::default_host();
    let input_device = match input_name {
        Some(name) if !name.trim().is_empty() => open_named_input_device(name)?,
        _ => host
            .default_input_device()
            .ok_or_else(|| "Default audio input device was not found.".to_string())?,
    };

    let input_default_config = input_device
        .default_input_config()
        .map_err(|error| format!("Could not read input format: {error}"))?;
    let sink_sample_rate = sink.config().sample_rate().get();
    let input_config =
        matching_input_config(&input_device, sink_sample_rate).unwrap_or(input_default_config);

    let output_channels = usize::from(sink.config().channel_count().get());
    let input_channels = usize::from(input_config.channels());
    let sample_rate = input_config.sample_rate();
    let max_buffer_len = sample_rate as usize * output_channels;
    let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(max_buffer_len)));

    let input_stream = build_input_bridge_stream(
        &input_device,
        &input_config,
        input_channels,
        output_channels,
        max_buffer_len,
        Arc::clone(&buffer),
    )?;
    let player = Player::connect_new(sink.mixer());
    player.append(MicBridgeSource {
        buffer,
        channels: sink.config().channel_count(),
        sample_rate: RodioSampleRate::new(sample_rate)
            .ok_or_else(|| "Mic bridge sample rate cannot be zero.".to_string())?,
    });

    input_stream
        .play()
        .map_err(|error| format!("Could not start mic bridge input: {error}"))?;

    Ok(MicBridge {
        _input_stream: input_stream,
        _player: player,
    })
}

fn matching_input_config(
    device: &Device,
    sample_rate: u32,
) -> Result<SupportedStreamConfig, cpal::SupportedStreamConfigsError> {
    let mut configs = device.supported_input_configs()?;
    configs
        .find(|config| {
            config.min_sample_rate() <= sample_rate && sample_rate <= config.max_sample_rate()
        })
        .map(|config| config.with_sample_rate(sample_rate))
        .ok_or(cpal::SupportedStreamConfigsError::DeviceNotAvailable)
}

fn build_input_bridge_stream(
    device: &Device,
    supported_config: &SupportedStreamConfig,
    input_channels: usize,
    output_channels: usize,
    max_buffer_len: usize,
    buffer: Arc<Mutex<VecDeque<f32>>>,
) -> Result<Stream, String> {
    let config = supported_config.config();
    let error_callback = |error| eprintln!("SoulBind mic bridge input stream error: {error}");

    match supported_config.sample_format() {
        SampleFormat::F32 => device.build_input_stream(
            &config,
            move |data: &[f32], _| {
                push_input_samples(
                    data.iter().copied(),
                    input_channels,
                    output_channels,
                    max_buffer_len,
                    &buffer,
                );
            },
            error_callback,
            None,
        ),
        SampleFormat::I16 => device.build_input_stream(
            &config,
            move |data: &[i16], _| {
                push_input_samples(
                    data.iter().map(|sample| f32::from(*sample) / 32768.0),
                    input_channels,
                    output_channels,
                    max_buffer_len,
                    &buffer,
                );
            },
            error_callback,
            None,
        ),
        SampleFormat::U16 => device.build_input_stream(
            &config,
            move |data: &[u16], _| {
                push_input_samples(
                    data.iter()
                        .map(|sample| (f32::from(*sample) - 32768.0) / 32768.0),
                    input_channels,
                    output_channels,
                    max_buffer_len,
                    &buffer,
                );
            },
            error_callback,
            None,
        ),
        format => {
            return Err(format!(
                "Unsupported audio input sample format for call mix: {format:?}"
            ));
        }
    }
    .map_err(|error| format!("Could not open mic bridge input stream: {error}"))
}

struct MicBridgeSource {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    channels: ChannelCount,
    sample_rate: RodioSampleRate,
}

impl Iterator for MicBridgeSource {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        Some(pop_sample_or_silence(&self.buffer))
    }
}

impl Source for MicBridgeSource {
    fn current_span_len(&self) -> Option<usize> {
        None
    }

    fn channels(&self) -> ChannelCount {
        self.channels
    }

    fn sample_rate(&self) -> RodioSampleRate {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        None
    }
}

fn push_input_samples(
    samples: impl Iterator<Item = f32>,
    input_channels: usize,
    output_channels: usize,
    max_buffer_len: usize,
    buffer: &Arc<Mutex<VecDeque<f32>>>,
) {
    if input_channels == 0 || output_channels == 0 {
        return;
    }

    let samples = samples.collect::<Vec<_>>();
    let mut queue = match buffer.lock() {
        Ok(queue) => queue,
        Err(_) => return,
    };

    for frame in samples.chunks(input_channels) {
        push_mapped_frame(&mut queue, frame, output_channels);
    }

    while queue.len() > max_buffer_len {
        queue.pop_front();
    }
}

fn push_mapped_frame(queue: &mut VecDeque<f32>, input_frame: &[f32], output_channels: usize) {
    if input_frame.is_empty() {
        return;
    }

    for channel in 0..output_channels {
        let sample = if channel < input_frame.len() {
            input_frame[channel]
        } else {
            input_frame[0]
        };
        queue.push_back(sample.clamp(-1.0, 1.0));
    }
}

fn pop_sample_or_silence(buffer: &Arc<Mutex<VecDeque<f32>>>) -> f32 {
    let mut queue = match buffer.lock() {
        Ok(queue) => queue,
        Err(_) => return 0.0,
    };

    queue.pop_front().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_mono_input_to_all_output_channels() {
        let mut queue = VecDeque::new();

        push_mapped_frame(&mut queue, &[0.25], 2);

        assert_eq!(queue.into_iter().collect::<Vec<_>>(), vec![0.25, 0.25]);
    }

    #[test]
    fn preserves_matching_input_channels() {
        let mut queue = VecDeque::new();

        push_mapped_frame(&mut queue, &[0.25, -0.5], 2);

        assert_eq!(queue.into_iter().collect::<Vec<_>>(), vec![0.25, -0.5]);
    }

    #[test]
    fn mic_source_reads_buffer_then_silence() {
        let buffer = Arc::new(Mutex::new(VecDeque::from([0.5_f32])));
        let mut source = MicBridgeSource {
            buffer,
            channels: ChannelCount::new(2).unwrap(),
            sample_rate: RodioSampleRate::new(48_000).unwrap(),
        };

        assert_eq!(source.next(), Some(0.5));
        assert_eq!(source.next(), Some(0.0));
        assert_eq!(source.next(), Some(0.0));
    }
}
