# Platform Notes

## Windows

Windows is expected to be a primary target.

Implementation notes:

- Test shortcuts with and without modifier keys.
- Avoid reserved OS shortcuts.
- Package with NSIS or MSI after the MVP is stable.
- Confirm audio output device recovery after device changes.

## macOS

macOS is expected to be a primary target.

Implementation notes:

- Use `CmdOrControl` in user-facing shortcut strings where possible.
- Some shortcuts are reserved by macOS or other apps.
- Code signing and notarization are needed for polished distribution.
- Test close-to-tray behavior carefully because macOS app lifecycle expectations differ from Windows/Linux.

## Linux

Linux support should be explicit and honest.

Implementation notes:

- Test on X11 and Wayland separately.
- Wayland may prevent global shortcuts depending on compositor policy.
- Package targets should be chosen after testing: AppImage, `.deb`, and Flatpak are likely candidates.
- Linux audio builds may require ALSA development packages through `cpal`.

## Cross-Platform Shortcut Text

Prefer portable shortcut names in storage:

- `CmdOrControl` instead of separate `Control` and `Command` when behavior should be platform-native
- `Alt`, `Shift`, and named keys for clarity
- Avoid function keys as defaults because laptops and desktop environments often reserve them

## Startup Behavior

SoulBind should not automatically start at login in the MVP. Add autostart only after users can clearly see and control the setting.

