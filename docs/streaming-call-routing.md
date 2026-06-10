# Streaming And Call Routing

SoulBind can mix your physical microphone and soundboard audio into a selected virtual audio device. For gaming calls and streams, this matters because audio played to normal speakers is usually not sent into Discord, Zoom, game voice chat, or a stream unless that app captures desktop audio.

## What SoulBind Does

SoulBind now has a Call Mix panel.

Available controls:

- Mic: your physical microphone.
- Discord input: the virtual output device that Discord, OBS, or a game should use as its microphone/input source.
- Mix mic: starts or stops the live mic-to-Discord-input mix.

When Mix mic is on, SoulBind sends your mic and soundboard to the selected Discord input device.

## Recommended Discord Setup With BlackHole

1. Install BlackHole.
2. Open SoulBind.
3. In Call Mix, set Mic to your real microphone.
4. Set Discord input to BlackHole.
5. Turn Mix mic on.
6. In Discord, set Input Device to BlackHole.
7. Test with your voice and one short sound before joining a real call.

Do not set Discord input to your physical mic in this setup. Discord should listen to BlackHole, because SoulBind is now feeding both your mic and soundboard into BlackHole.

SoulBind cannot create a new OS microphone name by itself. Discord will still show the virtual device name, such as BlackHole. Creating a real device named "SoulBind Mix" would require installing a custom virtual audio driver.

## Examples

Common routing targets include:

- macOS: BlackHole or another virtual output/input device.
- Windows: a virtual cable or mixer output.
- Linux: a PipeWire/PulseAudio virtual sink or monitor source.

## Important Limitation

SoulBind is not a virtual microphone driver. It needs BlackHole, VB-Cable, PipeWire, or another virtual routing device that the operating system exposes to Discord or OBS as an input source.

## OBS

For streaming, either set OBS microphone input to the Discord input device or add that device as its own audio source. Keeping it as a separate OBS source gives you better control over levels.

## Voice Chat

For voice chat, select the virtual cable or mixer input as the microphone source in the call app. In SoulBind, keep Mix mic on so your real microphone is mixed into that same virtual input.
