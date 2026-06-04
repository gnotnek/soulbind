# UX And Design System

## Product Shape

SoulBind is a utility app. The first screen should be the working surface, not a landing page.

Primary layout:

- Left or top area: search, add button, global enable toggle, stop-all button
- Main area: list of sound bindings
- Side panel or modal: edit selected binding
- Bottom/status area: platform and registration warnings

## Binding Row

Each binding row should show:

- Sound name
- Shortcut
- File status
- Enabled state
- Volume
- Last triggered feedback
- Actions: play, edit, duplicate, remove

Use icons for common actions. Text labels are appropriate for primary commands such as Add Sound.

## Material Design Mapping

Use Material-style components:

- Top app bar for the app name and global actions
- Filled button for Add Sound
- Icon buttons for play, edit, delete, stop
- Switch for enabled/disabled
- Slider for volume
- Text field for search
- Dialog for editing a binding
- Snackbars for short success or failure messages

## Palette

Use the supplied palette with restraint:

- `#000000`: primary text and high-contrast surfaces
- `#FFFFFC`: app background
- `#BEB7A4`: borders, dividers, muted surfaces
- `#FF7F11`: primary action and active shortcut state
- `#FF3F00`: destructive actions and registration errors

Avoid making the whole interface orange. The app should mostly read as near-white, black, and neutral, with orange reserved for action and state.

## Typography

Use a legible sans-serif stack:

```css
font-family: Inter, Roboto, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
```

If using Google's Material Symbols font, bundle it or document it as a build asset so the app does not depend on a runtime network request.

## Accessibility

Minimum requirements:

- All icon buttons need accessible labels.
- Keyboard navigation must reach every control.
- Shortcut recording must have a cancel path.
- Color should not be the only indicator of error or active state.
- Text contrast must be checked against the final colors.

