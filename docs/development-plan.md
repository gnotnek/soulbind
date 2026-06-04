# Development Plan

## Current Status

The initial implementation now covers the planned MVP loop:

- Tauri 2 app shell
- Rust app state
- Versioned config storage
- `rodio` playback
- Global shortcut registration
- Shortcut capture UI
- Search and filtering
- Enable/disable controls
- Per-binding volume and playback mode
- Native file picker
- System tray menu
- Unit tests for the core Rust logic

## Phase 1: Technical Skeleton

- Complete: Create a Tauri 2 app named SoulBind.
- Complete: Add the global-shortcut plugin.
- Complete: Add a Rust app state object shared across commands.
- Complete: Add a minimal Material-styled binding list UI.
- Complete: Add load/save for a versioned config file.

## Phase 2: Audio MVP

- Complete: Add the `rodio` audio engine.
- Complete: Implement play-file by path.
- Complete: Implement stop-all.
- Complete: Add per-binding volume.
- Complete: Surface audio device and decode errors.

## Phase 3: Shortcut MVP

- Complete: Implement shortcut capture in the UI.
- Complete: Normalize shortcut strings in Rust.
- Complete: Detect internal duplicates.
- Complete: Register enabled shortcuts on startup.
- Complete: Re-register shortcuts after edits.
- Complete: Display per-binding registration status.

## Phase 4: UX Polish

- Complete: Add search and filtering.
- Complete: Add empty state for first-run users.
- Complete: Add snackbars for failures.
- Partial: Add keyboard navigation.
- Complete: Add a tray menu with show, stop all, and quit.

## Phase 5: Packaging

- Complete: Add app icon.
- Complete: Configure Windows, macOS, and Linux bundle targets.
- Complete: Document platform prerequisites.
- Complete: Add unit tests for config migration and shortcut parsing.
- Complete: Add release checklist.

## Recommended First Milestone

The first milestone should prove the core loop:

1. Add one sound file.
2. Record one shortcut.
3. Save it.
4. Restart the app.
5. Press the shortcut while the app is unfocused.
6. Hear the sound.
