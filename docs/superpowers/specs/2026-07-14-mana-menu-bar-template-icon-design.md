# Mana Menu-Bar Template Icon Design

## Goal

Make Mana recognizable and visually native in the macOS menu bar without weakening the approved full-color fantasy potion artwork used by the application bundle.

## Root Cause

The tray builder currently clones `app.default_window_icon()`. That source is the full-color square app icon, including its blue-sky and cloud background. At menu-bar scale, macOS displays the entire square as a tiny thumbnail instead of a compact status glyph.

## Selected Direction

Use direction A: a filled potion silhouette with a clear liquid cutout. The silhouette has enough mass to remain recognizable at 18 points while the interior cutout keeps it from reading as a generic circle or flask.

## Asset Model

- Keep `src-tauri/icons/mana-potion-master.png` and every derived application icon unchanged.
- Add a separate tray source at `src-tauri/icons/tray-template.svg`.
- Add the runtime raster at `src-tauri/icons/tray-template.png`.
- The PNG is exactly 36x36 pixels, representing an 18-point menu-bar glyph at 2x scale.
- The canvas is transparent with no square background, shadow, glow, color, text, or decorative cloud scene.
- The potion uses alpha only: opaque silhouette pixels, transparent exterior, and a transparent liquid-line cutout.
- The silhouette stays centered with enough clear padding that macOS does not crop it and neighboring menu-bar items retain normal spacing.

## Runtime Integration

- Load the dedicated PNG in Rust instead of cloning `app.default_window_icon()`.
- Pass the image to `TrayIconBuilder` and set `.icon_as_template(true)`.
- macOS owns the rendered foreground color so the glyph adapts automatically to light, dark, active, and inactive menu-bar appearances.
- Mana is declared as an agent application with `LSUIElement = true` in the packaged `Info.plist` and retains `ActivationPolicy::Accessory` at runtime. LaunchServices must never register the running widget as a Dock application.
- Set the tray tooltip to `Mana`.
- Keep the existing left-click menu behavior, window toggle behavior, and non-activating panel behavior unchanged.
- Change the menu command label from `Quit mana` to `Quit Mana`.

## Release Metadata

- Release version is `0.4.5` in npm, Cargo, and Tauri metadata.
- Visible product name remains `Mana`.
- Internal npm package and Rust crate names remain `mana`.
- Bundle identifier remains `com.vantasoft.mana`.

## Verification

- Add a focused Rust test or build-time assertion for the dedicated tray image path and template configuration.
- Add regression assertions that the bundle source declares `LSUIElement = true` and the runtime retains `ActivationPolicy::Accessory`.
- Verify the PNG is 36x36 RGBA, has transparent corner pixels, contains a centered non-empty silhouette, and does not fill the complete square.
- Verify production frontend and native builds succeed and all frontend and Rust tests pass.
- Install `/Applications/Mana.app`, confirm bundle metadata reports `Mana`, `com.vantasoft.mana`, and `0.4.5`, and compare built/installed executable and icon hashes.
- Relaunch with CuaDriver and visually inspect the real menu bar: the icon must have no colored square, match neighboring item height, remain recognizable as a potion, and avoid clipping or crowding. Confirm the running Mana process has no Dock presence and that the installed `Info.plist` contains `LSUIElement = true`.

## Scope

No changes to the widget layout, full-color bundle metadata icon, provider sprites, mana bars, motion timing, polling, credentials, activity detection, panel positioning, or tray interaction behavior beyond the dedicated icon, tooltip, and corrected quit label. Mana remains exclusively a menu-bar widget and must not appear in the Dock.
