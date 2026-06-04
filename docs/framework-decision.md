# Framework Decision

## Decision

Use **Tauri 2** as the desktop application framework for SoulBind.

The application should be Rust-first: audio, global shortcuts, persistence, validation, and platform integration live in Rust. The visual interface can use web technologies inside Tauri's system webview because the prompt asks for Material Design and Material Icons, which are significantly easier to implement well in HTML/CSS than in today's pure-Rust GUI stacks.

## Why Tauri 2 Fits SoulBind

SoulBind needs:

- Linux, Windows, and macOS desktop support
- Global keyboard shortcuts while the app is in the background
- Low-overhead audio playback
- A polished Material-style UI
- System tray behavior for a background utility
- Practical packaging and updates later

Tauri 2 directly covers the desktop shell, packaging model, plugin model, system tray, and global shortcut integration. Its official docs describe cross-platform app support across Linux, macOS, and Windows, and the global-shortcut plugin supports registering shortcuts from both JavaScript and Rust. The system tray API is also available from Rust.

## Candidate Comparison

| Framework | Strengths | Weaknesses For SoulBind | Decision |
| --- | --- | --- | --- |
| Tauri 2 | Cross-platform shell, official global-shortcut plugin, Rust backend, system webview, system tray support, small compared with Electron-style apps | UI is not pure Rust unless paired with a Rust web UI framework; Linux global shortcuts still need platform handling | Chosen |
| Iced | Pure Rust, cross-platform GUI, good for native-feeling tools | No first-party app shell equivalent to Tauri; Material Design and tray/global-shortcut integration need more custom glue | Not first choice |
| Slint | Efficient declarative UI, strong Rust integration, desktop support | Licensing and commercial constraints need review for closed-source distribution; Material ecosystem less straightforward than web Material | Not first choice |
| egui/eframe | Very fast iteration, pure Rust, efficient immediate-mode UI | Material Design polish and conventional desktop utility controls require extensive custom work | Not first choice |
| Dioxus Desktop | Rust UI model, desktop/web/mobile story | Soundboard-specific OS integration still needs extra Rust glue; smaller desktop ecosystem than Tauri | Not first choice |

## Core Library Choices

Use `rodio` for audio playback first. It is a Rust playback library built on `cpal`, supports common decoded formats through its decoder backends, and mixes sounds before sending them to the OS audio device. That behavior matches a soundboard because several clips may overlap.

Use the Tauri global-shortcut plugin for shortcut registration. Keep shortcut ownership in Rust so the app can validate, register, unregister, and persist bindings without depending on frontend state.

## Risks

Linux global shortcuts are the biggest technical risk. The lower-level `global-hotkey` crate documents Linux support as X11-only, and many Wayland compositors intentionally restrict global key capture. SoulBind should treat global shortcut registration as fallible and show a clear per-binding status.

System webview differences can affect fine UI details. Keep the UI simple, test the main window on all three platforms, and avoid browser APIs that are unavailable in older system webviews.

## Sources

- Tauri 2 overview: https://v2.tauri.app/
- Tauri global-shortcut plugin: https://v2.tauri.app/plugin/global-shortcut/
- Tauri system tray docs: https://v2.tauri.app/learn/system-tray/
- Tauri file system plugin docs: https://v2.tauri.app/plugin/file-system/
- rodio docs: https://docs.rs/rodio/latest/rodio/
- rodio repository: https://github.com/RustAudio/rodio
- global-hotkey docs: https://docs.rs/global-hotkey
- Iced docs: https://docs.iced.rs/
- Slint docs: https://docs.slint.dev/

