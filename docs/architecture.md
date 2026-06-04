# Architecture

## High-Level Shape

SoulBind should be a Tauri 2 desktop application with a Rust core and a Material-styled webview UI.

```text
UI webview
  - Binding list
  - Binding editor
  - Settings
  - Status and conflict messages
        |
        | Tauri commands/events
        v
Rust application core
  - Binding service
  - Shortcut registry
  - Audio engine
  - Persistence
  - Platform status
        |
        v
Operating system
  - Global shortcut APIs
  - Audio output device
  - App data directory
  - System tray
```

## Rust Modules

`bindings`

Owns the domain model for sound bindings. A binding should include an id, display name, sound file path, shortcut, volume, enabled flag, and registration status.

`shortcuts`

Normalizes shortcut text and builds a registration plan. This module has no Tauri dependency so conflict and validation behavior can be unit tested.

`shortcuts_tauri`

Applies the shortcut registration plan through the Tauri global-shortcut plugin. It updates each binding with registered, disabled, duplicate, invalid, or registration-failed status.

`audio`

Keeps the audio output stream alive and plays decoded clips through `rodio`. The `AudioHandle` trait keeps app-state tests independent from real audio hardware.

`storage`

Persists a versioned JSON config file under the OS app data directory. It can still read the original legacy raw-array format.

`commands`

Tauri command boundary used by the UI. Commands should return typed errors that the UI can display without leaking internal implementation details.

`state`

Owns shared app state: bindings, shortcut index, audio handle, and config path.

`tray`

Creates a tray icon with actions for show/hide, stop all sounds, enable/disable all bindings, and quit.

## Data Model

```rust
pub struct SoundBinding {
    pub id: String,
    pub name: String,
    pub file_path: PathBuf,
    pub shortcut: String,
    pub volume: f32,
    pub enabled: bool,
    pub mode: PlaybackMode,
}
```

The shortcut should be stored as a normalized string such as `CmdOrControl+Shift+1`. A later implementation can replace the string with a richer parsed type while keeping the persisted format stable.

Config is saved as:

```json
{
  "version": 1,
  "bindings": []
}
```

## Runtime Flow

1. App starts and loads persisted bindings.
2. Rust validates file paths and shortcuts.
3. Enabled bindings are registered with the global shortcut plugin.
4. UI renders bindings with status: active, disabled, missing file, conflict, or registration failed.
5. When a shortcut fires, Rust looks up the matching binding and asks the audio engine to play it.
6. UI receives a playback event so it can briefly show activity.
7. User edits trigger revalidation, persistence, and shortcut re-registration.

## Error Model

Errors should be categorized:

- Validation: invalid shortcut, missing file, unsupported audio format
- Registration: shortcut already taken, platform does not allow global shortcut
- Audio: output device unavailable, decode failed, playback failed
- Storage: config read/write failure, config migration failure

The UI should show short, actionable text. Detailed errors should go to logs.
