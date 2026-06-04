# Audio And Hotkeys

## Audio Engine

Use `rodio` for the first implementation.

Reasons:

- It is Rust-native and uses `cpal` for OS audio output.
- It handles common playback needs without building directly against platform audio APIs.
- Its mixer model allows multiple sounds to play at the same time, which is required for soundboard behavior.

SoulBind supports selecting the audio output device. This is required for streaming and voice chat workflows where soundboard audio must be routed to a virtual cable, loopback, or mixer device instead of ordinary speakers.

## Playback Behavior

Implemented behavior:

- Pressing a bound shortcut plays the sound immediately.
- Multiple bindings can overlap.
- Pressing the same binding repeatedly can layer, restart, or be ignored based on a per-binding mode.
- A global stop button should stop all currently playing clips.

- `Layer`: each trigger starts another copy.
- `Restart`: stop the previous instance of that binding before replaying.
- `Single`: ignore trigger while the sound is already playing.

## Supported Audio Formats

Start with the formats supported by the selected `rodio` feature set. The product UI should list supported formats in the file picker helper text after implementation confirms the exact enabled decoder features.

Practical MVP targets:

- WAV
- MP3
- OGG/Vorbis
- FLAC

## Shortcut Handling

Shortcut registration should be owned by Rust, not by the UI.

Responsibilities:

- Parse and normalize shortcut input.
- Reject duplicates inside SoulBind.
- Register enabled shortcuts on startup.
- Unregister old shortcuts when bindings change.
- Surface OS registration failures per binding.
- Keep an in-memory map from shortcut to binding id.

The UI also records keyboard combinations into the shortcut field for faster entry, but Rust remains the source of truth.

## Shortcut Conflicts

There are two conflict categories:

- Internal conflict: two SoulBind bindings use the same normalized shortcut.
- External conflict: the OS refuses registration, often because another app owns the shortcut.

Internal conflicts can be detected before registration. External conflicts can only be known after asking the OS.

## Linux Limitation

Linux global shortcut behavior depends heavily on the display server. X11 is generally workable. Wayland compositors often restrict global key capture by design, so registration may fail or require portal-specific support in the future.

SoulBind should not hide this. It should show the binding as unavailable and keep the rest of the app usable.
