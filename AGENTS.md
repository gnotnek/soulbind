# Repository Guidelines

## Project Structure & Module Organization

SoulBind is a Tauri 2 desktop app with a static frontend and Rust backend.

- `src/`: frontend files (`index.html`, `main.js`, `styles.css`).
- `src-tauri/src/`: Rust application code. Key modules include `audio.rs`, `commands.rs`, `state.rs`, `storage.rs`, `shortcuts.rs`, and `tray.rs`.
- `src-tauri/icons/`: packaged app icons for desktop/mobile targets.
- `src-tauri/capabilities/`: Tauri permission configuration.
- `docs/`: architecture, platform notes, routing setup, release checklist, and development notes.
- `.github/workflows/release.yml`: cross-platform release build and asset upload workflow.

Rust unit tests live beside the modules they cover under `#[cfg(test)]`.

## Build, Test, and Development Commands

- `npm run dev`: run the Tauri app locally in development mode.
- `npm run build`: build packaged release artifacts through `cargo tauri build`.
- `cargo check --manifest-path src-tauri/Cargo.toml`: type-check the Rust backend.
- `cargo test --manifest-path src-tauri/Cargo.toml`: run Rust unit tests.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`: verify Rust formatting.
- `node --check src/main.js`: validate frontend JavaScript syntax.
- `cargo tauri build --debug`: produce a debug bundle for local packaging checks.

## Coding Style & Naming Conventions

Use `cargo fmt` for Rust formatting. Prefer small, focused modules and keep state transitions in `state.rs`, Tauri command wrappers in `commands.rs`, and audio device/playback behavior in `audio.rs`.

Rust names should follow standard conventions: `snake_case` for functions and variables, `PascalCase` for structs/enums/traits. Frontend code uses plain JavaScript with `camelCase` functions and a centralized `els` object for DOM references. Keep HTML and CSS class names descriptive and stable, for example `orchestrator-panel` or `binding-row`.

## Testing Guidelines

Add Rust unit tests near changed logic. For audio routing changes, prefer deterministic tests for buffer/channel mapping and state persistence instead of hardware-dependent tests. Run `cargo test --manifest-path src-tauri/Cargo.toml` before committing backend changes and `node --check src/main.js` before committing frontend changes.

## Commit & Pull Request Guidelines

Follow the existing concise conventional style: `feat: ...`, `ci: ...`, `chore: ...`. Keep commits focused and include detailed body text for behavioral or release-impacting changes.

Pull requests should include a clear summary, verification commands run, linked issues when applicable, and screenshots or short recordings for UI changes. For release changes, note version bumps, tag names, and expected platform artifacts.

## Security & Configuration Tips

Do not commit secrets, tokens, signing keys, or local build outputs. Keep Tauri permissions and CSP changes minimal and documented. For audio routing, document required OS-level virtual devices such as BlackHole, VB-Cable, or PipeWire rather than assuming they exist.
