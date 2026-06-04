# Streaming And Call Routing

SoulBind can play soundboard audio to a selected output device. For gaming calls and streams, this matters because audio played to normal speakers is usually not sent into Discord, Zoom, game voice chat, or a stream unless that app captures desktop audio.

## What SoulBind Does

SoulBind now has an Audio Route setting in the app sidebar.

Available routes:

- System default: plays wherever the OS currently sends app audio.
- Named output device: plays directly to the selected output device.

Use a named virtual cable, loopback, or mixer device when you want other apps to hear SoulBind.

## Recommended Setup

1. Install or configure a virtual audio output device for your OS.
2. Open SoulBind.
3. In Audio Route, select that virtual output device.
4. In your call or streaming app, add/select the matching virtual device as an audio source or microphone input.
5. Test with one short sound before going live.

## Examples

Common routing targets include:

- macOS: a virtual output or aggregate device.
- Windows: a virtual cable or mixer output.
- Linux: a PipeWire/PulseAudio virtual sink or monitor source.

## Important Limitation

SoulBind is not a virtual microphone driver. It cannot make Discord or a game voice chat hear audio by itself. The operating system or a routing tool must expose SoulBind's output as an input source for the call or streaming app.

## OBS

For streaming, the cleanest setup is usually to add SoulBind's routed output as its own OBS audio source. This lets you adjust soundboard volume separately from your microphone and game audio.

## Voice Chat

For voice chat, select the virtual cable or mixer input as the microphone source in the call app. If you also need your real microphone, route both your microphone and SoulBind into the same virtual/mixer input.

