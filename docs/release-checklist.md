# Release Checklist

Use this checklist before publishing a SoulBind release.

## Code Quality

- Run `cargo fmt --manifest-path src-tauri/Cargo.toml --check`.
- Run `cargo check --manifest-path src-tauri/Cargo.toml`.
- Run `cargo test --manifest-path src-tauri/Cargo.toml`.
- Run `cargo audit --file src-tauri/Cargo.lock`.
- Run `node --check src/main.js`.
- Confirm there are no Rust warnings.

## Manual App Test

- Launch with `cargo tauri dev`.
- Add a WAV file through the native file picker.
- Add a sound by typing a full path manually.
- Record a shortcut in the shortcut field.
- Save, restart, and confirm the binding persists.
- Press the shortcut while another app is focused.
- Test layer, restart, and single playback modes.
- Test stop-all from the window and tray menu.
- Disable one binding and confirm its shortcut no longer plays.
- Disable all bindings and confirm no global shortcuts fire.
- Duplicate a binding and confirm the copy is disabled.

## Platform Pass

- Windows: verify global shortcuts, audio playback, tray menu, and installer output.
- macOS: verify global shortcuts, audio playback, tray menu, `.app`, and `.dmg`.
- Linux X11: verify global shortcuts, audio playback, tray menu, and package output.
- Linux Wayland: verify graceful failure messaging when global shortcuts are unavailable.

## Distribution

- Replace development signing with release signing.
- Notarize macOS builds.
- Confirm bundle identifier and version.
- Confirm final app icon renders at small and large sizes.
- Attach platform-specific caveats to release notes.
