# SoulBind

SoulBind is a planned cross-platform desktop soundboard for Linux, Windows, and macOS. Its core job is simple: bind local sound files to global keyboard shortcuts so clips can be played without bringing the app into focus.

## Framework Decision

SoulBind should be built with **Tauri 2**.

The Rust side owns the parts that must be efficient and reliable: shortcut registration, audio playback, persistence, validation, and platform integration. The UI runs in the system webview, which makes Material Design and Material Icons practical without shipping a full Chromium runtime.

See [docs/framework-decision.md](docs/framework-decision.md) for the full comparison.

## Planned Stack

- Desktop shell: Tauri 2
- Core language: Rust
- Global shortcuts: Tauri global-shortcut plugin, backed by Rust-side registration
- Audio playback: rodio over cpal
- UI: Material Design styled HTML/CSS with Material Icons
- Persistence: versioned app config in the OS app data directory

## MVP Scope

- Add, edit, duplicate, remove, enable, and disable sound bindings
- Bind one sound file to one global shortcut
- Play sounds while SoulBind is in the background
- Stop all currently playing sounds
- Per-binding volume
- Per-binding playback mode: layer, restart, or single
- Search and filter bindings
- Conflict detection for duplicate shortcuts inside SoulBind
- Platform warnings for unavailable global shortcuts
- Import existing audio files by path or with the native file picker
- System tray actions for show, stop all, and quit

## Documentation

- [Architecture](docs/architecture.md)
- [Framework Decision](docs/framework-decision.md)
- [Audio And Hotkeys](docs/audio-and-hotkeys.md)
- [UX And Design System](docs/ux-design-system.md)
- [Platform Notes](docs/platform-notes.md)
- [Development Plan](docs/development-plan.md)
- [Release Checklist](docs/release-checklist.md)
- [Streaming And Call Routing](docs/streaming-call-routing.md)

## Development Setup

Install Rust and the Tauri prerequisites for your platform first. Tauri 2 requires Rust 1.77.2 or newer for its plugins.

Recommended first implementation steps:

```sh
cargo install tauri-cli
cargo tauri dev
```

Run checks:

```sh
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## Design Direction

SoulBind should feel like a focused utility rather than a marketing page. The first screen should be the binding workspace: a searchable list of sound bindings, a compact editor panel, and transport controls.

Palette:

- `#000000` black
- `#FFFFFC` near-white
- `#BEB7A4` muted neutral
- `#FF7F11` orange
- `#FF3F00` red-orange

The UI should use Material Design interaction patterns and Material Icons for actions.

## Important Platform Caveat

Global shortcuts are not equally available everywhere. Windows and macOS are the best-supported targets. Linux support depends on the display server and desktop environment; X11 is generally more permissive than Wayland. SoulBind should detect failures, explain them in-app, and keep in-window shortcuts available as a fallback.

## Current Test Coverage

The Rust core has unit tests for binding validation, duplicate shortcut detection, disabled duplicate copies, shortcut normalization and planning, versioned config save/load including legacy migration, and app-state audio delegation.
