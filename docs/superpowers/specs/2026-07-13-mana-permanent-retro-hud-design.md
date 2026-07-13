# mana v0.4 - Permanent Retro Glass HUD

2026-07-13 - approved by Dennis

Approved direction: keep the Party Roster permanently expanded, give every usage row an equal-length pixel-art meter, and push the glass HUD toward a restrained GBA-era visual language.

## Goal

Make mana a stable, always-readable desktop status panel. The widget must open directly into the full provider roster, keep every meter identical in length, apply the same glow behavior to every fill, and feel more deliberately retro without sacrificing the native macOS glass treatment.

## Problems being fixed

- The compact state hides information the user wants visible at all times and creates unnecessary hover-driven window resizing.
- Each row currently allocates its reset value with `max-content`, so longer reset strings steal space from that row's meter and produce visibly unequal tracks.
- Glow and motion differ by state: working bars receive a scan, low bars pulse, and ordinary bars only receive a static shadow.
- The current rounded, smooth meter treatment reads more like a modern dashboard than a pixel-game HUD.
- Multiple running instances made it easy to mistake the installed v0.2.1 widget for the v0.3 development build. Runtime consolidation is operational cleanup, while single-instance enforcement is not part of this visual change.

## Scope

### In scope

- Remove the compact roster and its hover expansion/collapse behavior.
- Launch and remain at a 420px-wide expanded roster, with height measured from rendered content.
- Preserve dragging, saved position, screen-edge containment, provider activity detection, polling, and tray behavior.
- Give every usage row a fixed 128x16 logical-pixel meter.
- Reuse the supplied Boss HUD frame pieces as the meter outline.
- Keep Claude and Codex colors distinct while applying one shared glow and stepped shimmer treatment.
- Make the container, dividers, type, signals, and spacing feel more retro while retaining native glass.
- Preserve stale, absent, low-mana, and reduced-motion behavior.
- Add the required third-party attribution.

### Out of scope

- Runtime single-instance enforcement or changing Tauri process startup semantics.
- New usage sources, polling cadence, credentials, token refresh, or activity detection.
- New settings, pin controls, notifications, sounds, clicks, or keyboard interaction.
- Replacing Clawd or Nimbus, adding more decorative sprites, or importing unrelated scenery from the asset pack.
- Framework migration or a new runtime dependency.

## Permanent Window Behavior

- The only visible frontend is the Party Roster card. `#pill`, `pillHtml`, and compact reset formatting are removed rather than hidden.
- Tauri starts the main window at 420 logical pixels wide so there is no 340px compact flash before JavaScript runs.
- Initial height is large enough for the normal roster shell, then the frontend measures and applies the exact rendered height after initial rendering and subsequent data-shape changes.
- The serialized native resize queue remains so data updates cannot race window operations.
- Startup and resize clamp the 420px window to the active monitor work area. A position saved for the old 340px widget must not leave the wider roster offscreen.
- Mouse entry and exit may continue to change familiar animation state, but never change window geometry.
- Dragging continues to use the existing deep Tauri drag region. Movement may use the hover sprite row while in progress and returns to the provider's idle or working row after settling.

## Layout

- Window width: 420 logical pixels.
- Container radius: 8 logical pixels, matched by the native vibrancy radius.
- Container padding: 14px 16px.
- Each provider band keeps the existing 44px familiar column and flexible content column.
- Every usage row uses the same columns: `52px 128px minmax(0, 1fr)` with 8px gaps.
- The final column is right-aligned and must fit the full percentage and normal reset string, including `100% · Sun 12:51 PM`.
- Every meter must report exactly 128 logical pixels from `getBoundingClientRect().width`, regardless of label, percentage, reset length, provider, or row count.
- Provider bands remain unframed sections separated by one pixel divider. No nested cards are introduced.

```text
+--------------------------------------------------+
| [Clawd]  CLAUDE  Max                          [*] |
|          5 hour  [||||||||||||....]  96% · 1h 10m |
|          Weekly  [|||||...........]  36% · Tue... |
|          Fable   [................]   0% · Tue... |
| ------------------------------------------------ |
| [Nimbus] CODEX  Pro                           [*] |
|          Weekly  [|||||||.........]  45% · Sun... |
+--------------------------------------------------+
```

The shortened reset strings above are wireframe notation only. The implemented expanded values render in full.

## Pixel Meter Assets

Use these files from `Another Metroidvania Asset Pack ver. 1.7/User Interface/Boss Hud/`:

- `health_bar_icon_left.png` - 32x16
- `health_bar_icon_mid.png` - 32x16
- `health_bar_icon_right.png` - 32x16

Copy the three source PNGs unchanged into `public/hud/`. Compose one left piece, two repeating middle pieces, and one right piece into a 128x16 outline. The colored CSS fill sits beneath the frame and is clipped inside the frame's interior.

Rendering requirements:

- Use integer logical dimensions and `image-rendering: pixelated`.
- Never stretch or rescale a piece to a fractional size.
- The frame stays fixed while fill width changes from 0% through 100%.
- Frame pixels remain neutral so Claude cyan, Codex magenta, and low-mana coral fills all remain legible.
- Do not import the complete red bar, potion, orb, environment, or character assets.

## Color, Glow, and Motion

- Claude fills remain cyan-to-indigo.
- Codex fills remain violet-to-magenta.
- Low mana may shift either provider to coral-red, but it uses the same glow construction and timing as every other bar.
- Every non-empty fill receives the same two-layer halo, hard top-edge highlight, and stepped pixel shimmer. Hue comes from provider or low state; blur radius, opacity, animation duration, and easing are identical.
- Remove the activity-only scan and the distinct low-mana pulse so no row appears to have a more elaborate meter effect than another.
- Working state remains visible through the provider familiar and a square activity signal, not a different bar effect.
- Under `prefers-reduced-motion: reduce`, the shimmer and sprite animation stop while the static fill and glow remain visible.

## Retro Glass Treatment

- Keep the native macOS `HudWindow` vibrancy as the physical glass layer.
- Reduce the CSS and native corner radius from 16px/14px to 8px.
- Replace soft rounded meter caps with the imported hard pixel frame.
- Use a 4px grid/dither texture, one-pixel dividers, and hard inset bevel highlights.
- Use the existing macOS monospace stack for headings, labels, values, and metadata so column alignment is stable.
- Keep type at readable utility sizes: 13px provider names, 11px labels/reset metadata, and 12px percentages.
- Keep the palette balanced: neutral glass and text, cyan Claude, magenta Codex, coral low state, and the mascots' existing colors.
- Do not add gradients or ornaments unrelated to the functional glass, fills, and pixel texture.

## Data and State Behavior

- `ok`: full-color familiar, header, framed meters, percentages, and reset values.
- `working`: same meters as `ok`; animated familiar and square activity signal identify activity.
- `low`: coral fill using the universal glow/shimmer treatment.
- `stale`: retain all geometry, reduce provider saturation, and show fetched age without changing column widths.
- `absent`: retain the familiar and header, then show one readable login hint with no placeholder meter.
- Unknown reset timestamps omit only the reset suffix; label, track, and percentage alignment remain stable.
- Codex Pro weekly-only remains one `Weekly` row. No `5 hour` row is synthesized.

## Asset Attribution

The selected frame pieces are licensed under Creative Commons Attribution 4.0 International.

- Creator credit: `o_lobster`
- Source: `https://o-lobster.itch.io`
- License: `https://creativecommons.org/licenses/by/4.0/`
- Modification disclosure: the PNG files are copied unchanged and composed by CSS as a 128x16 usage frame; colored fills and glow are original mana styling.

Add this notice to `THIRD_PARTY_NOTICES.md` and add a short attribution link from `README.md`. Do not imply that the creator endorses mana.

## Implementation Boundaries

- Keep Tauri 2, vanilla TypeScript, HTML, and CSS.
- Keep the provider card renderer and all dynamic text escaping.
- Remove compact-only renderer, formatter, layout intent, collapse timers, origin restoration, and their dead tests.
- Retain the serial sizing queue, content-height calculation, work-area clamp, activity event handling, and saved position plugin.
- Keep `src-tauri/tauri.conf.json`, frontend geometry constants, CSS width assumptions, and native vibrancy radius synchronized.
- Synchronize package and bundle metadata to v0.4.0 because this behavior supersedes the uninstalled v0.3.0 UI.
- Leave the user's untracked `.DS_Store` untouched.

## Verification

### Automated

- Write failing tests before production changes.
- Replace hover/collapse tests with permanent-window startup, work-area clamp, measured-height, and serialized-queue recovery coverage.
- Remove `pillHtml` and compact-countdown tests only after the permanent-roster tests demonstrate the replacement behavior.
- Keep renderer tests for weekly-only Codex, absent state, and escaped labels.
- Run `npm test` and `npm run build`.
- Run `rustfmt --edition 2021 --check` for changed Rust files, `cargo test`, and `cargo check` in `src-tauri`.
- Run `git diff --check`.

### Browser and Native QA

- Launch smoke shows the complete roster without pointer input and without a compact-state flash.
- Inspect every `.row .track` and confirm all computed widths are exactly 128 logical pixels.
- Test fills at 0%, 1%, 29%, 30%, 99%, and 100% without frame distortion or overflow.
- Verify `100% · Sun 12:51 PM`, three Claude rows, and weekly-only Codex fit without clipping, overlap, scrollbars, or excess blank space.
- Verify ordinary, working, low, stale, absent, and reduced-motion states.
- Verify the same glow geometry and shimmer timing on Claude, Codex, and low fills while their colors remain distinct.
- Verify dragging, restart position persistence, content-height changes, and right-edge containment.
- Capture the real Tauri window at Retina scale because native vibrancy and pixel alignment cannot be validated in a regular browser alone.
- Confirm only one mana process is running for the final installed smoke test.

## Acceptance Criteria

- The full provider roster is visible immediately and never collapses.
- No hover action changes window dimensions.
- Every rendered usage meter is exactly the same length and uses the same frame geometry.
- Every non-empty fill uses the same glow and shimmer behavior, with provider-specific color retained.
- Full percentages and reset values remain readable without clipping.
- The asset frame is crisp at Retina scale and its fill behaves correctly from 0% to 100%.
- The glass, type, corners, dividers, meter frame, and motion read as one restrained retro-game HUD.
- Required CC BY 4.0 attribution ships with the app source.
- Polling, credentials, activity state, tray behavior, saved position, and edge containment continue to work.
