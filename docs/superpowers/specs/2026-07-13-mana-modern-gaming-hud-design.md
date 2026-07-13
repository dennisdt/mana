# mana v0.4 - Modern Gaming HUD

2026-07-13 - approved by Dennis

Approved direction: replace the rejected retro HUD with a premium sci-fi glass interface built around an original image-generated mana-bar bezel. Preserve the permanent expanded roster, equal-length meters, and existing pixel mascots.

This document supersedes the visual direction in `2026-07-13-mana-permanent-retro-hud-design.md`. That earlier document remains as project history, but its imported pixel frames, retro grid, monospaced-everywhere treatment, and asset attribution requirements do not ship in v0.4.

## Goal

Make mana feel like a polished game status HUD while remaining a readable macOS utility. The widget must expose all provider usage at a glance, render every meter at the same length, show complete percentages and reset times, and use original project-owned visual assets rather than the supplied metroidvania pack.

## Scope

### In scope

- Keep the roster permanently expanded and draggable.
- Increase the logical window width from 420px to 440px for reliable value-column readability.
- Retain content-measured height and monitor-edge containment.
- Replace the imported boss-bar pieces with one original image-generated sci-fi bezel.
- Render every meter at exactly 144x18 logical pixels.
- Keep meter fill, percentage, color, state, and motion driven by application code.
- Restyle the glass shell, provider bands, portrait slots, dividers, signals, typography, empty state, and stale state as one modern gaming system.
- Keep Claude and Codex energy colors distinct while giving every non-empty bar the same glow construction and energy sweep.
- Preserve the Clawd and Nimbus sprites as deliberate character avatars.
- Correct Codex Pro to one `Weekly` row with no synthesized `5 hour` row.
- Remove unused third-party HUD assets and any attribution that exists only for those removed assets.

### Out of scope

- Replacing or regenerating Clawd and Nimbus.
- Changing provider polling, authentication, credentials, activity detection, tray behavior, or saved-position behavior.
- Adding compact mode, settings, pin controls, sounds, notifications, clicks, or keyboard controls.
- Runtime single-instance enforcement.
- New providers, new usage windows, or inferred quota data.
- Framework migration or a new runtime dependency.

## Visual Direction

The interface is a premium sci-fi energy HUD rather than a retro game panel.

- The shell is smoked obsidian glass with graphite structure, a narrow cool border, one restrained top highlight, and subtle internal depth.
- The palette stays neutral around the data: charcoal, smoke, silver-white type, Claude cyan-to-blue energy, Codex magenta-to-pink energy, coral-red warning energy, and the mascots' existing warm/cool accents.
- Decorative treatment is functional. Fine bracket corners, bezel ticks, an activity light, and portrait illumination communicate game state; there are no decorative orbs, scenery, fake controls, or text explaining the UI.
- Provider names use the system sans-serif stack with compact uppercase styling. Labels and live numeric values use a tabular system-monospace stack.
- Corners stay at 8px or less. The design uses precise edges and clipped highlights rather than oversized rounding.
- Provider bands are unframed full-width sections separated by a hairline. There are no cards inside the glass shell.

## Layout

- Window width: 440 logical pixels.
- Window height: measured from rendered roster content after every data-shape change.
- Shell radius: 8 logical pixels, synchronized with the native vibrancy radius.
- Shell padding: 14px 16px.
- Provider layout: `44px minmax(0, 1fr)` with an 11px gap.
- Each familiar occupies a 44px portrait rail beside its provider, centered within that provider band.
- Usage-row layout: `52px 144px minmax(0, 1fr)` with 8px gaps.
- Meter dimensions: exactly 144x18 logical pixels for every provider, label, percentage, and reset value.
- The value column is right-aligned and wide enough for `100% · Sun 12:51 PM` without clipping, ellipsis, overlap, or horizontal scrolling.
- Header metadata may collapse only when absent; it must not steal space from usage values.

```text
+--------------------------------------------------------+
| [Clawd]  CLAUDE  Max                              [*]  |
|          5 hour  [ ENERGY CORE          ]  96% · 1h 10m |
|          Weekly  [ ENERGY CORE          ]  36% · Tue 1:59 PM |
|          Fable   [                      ]   0% · Tue 1:59 PM |
| ------------------------------------------------------ |
| [Nimbus] CODEX  Pro                               [*]  |
|          Weekly  [ ENERGY CORE          ]  45% · Sun 12:51 PM |
+--------------------------------------------------------+
```

The wireframe is schematic; the implemented grid reserves enough measured width for every value shown above.

## Generated Mana-Bar Bezel

Use the built-in image generation workflow to create one original neutral bezel. The selected production asset is stored at `public/hud/mana-bar-frame.png`.

### Artwork requirements

- One isolated horizontal hollow frame, shown straight-on with no perspective.
- Premium sci-fi equipment aesthetic: precision graphite alloy, restrained dark chrome, compact angular end caps, fine machined seams, and sparse etched calibration ticks.
- Symmetrical silhouette so one asset works for every row and provider.
- Transparent outer background and transparent center after chroma-key removal.
- No energy fill, text, numbers, logos, icons, mascot, scenery, cast shadow, reflection, particles, or watermark.
- No pixel-art treatment, chunky fantasy ornament, oversized bolts, or bright green material.
- High-resolution source with crisp continuous edges that remains legible when rendered at 144x18 logical pixels on Retina displays.
- The scaled frame must leave a continuous inner channel for a code-rendered fill. Bezel details may not visually divide the channel into inaccurate percentage segments.

### Generation prompt

Use case: `stylized-concept`

```text
Asset type: transparent UI game HUD element for a desktop usage widget
Primary request: Create one original premium sci-fi mana-bar bezel, an isolated horizontal hollow frame viewed perfectly straight-on.
Subject: Precision graphite-alloy frame with restrained dark-chrome highlights, compact angular end caps, fine machined seams, and sparse etched calibration ticks. The center must be completely open for a live energy fill rendered underneath.
Style/medium: polished modern AAA game interface asset, physically plausible hard-surface rendering, crisp and restrained rather than ornate
Composition/framing: one centered ultra-wide bezel, perfectly horizontal and symmetrical, orthographic front view, generous separation from the canvas edge
Lighting/mood: controlled cool rim light that defines the metal without adding a cast shadow
Color palette: neutral charcoal, graphite, gunmetal, and small silver highlights only
Scene/backdrop: perfectly flat solid #00ff00 chroma-key background for removal; the open center of the bezel must also show exactly the same flat #00ff00
Constraints: one frame only; continuous hollow center; crisp silhouette; no perspective; no background variation; no green anywhere in the frame
Avoid: energy fill, text, letters, numbers, logos, icons, gradients in the background, texture in the background, floor plane, cast shadow, contact shadow, reflection, particles, watermark, pixel art, fantasy ornament, oversized bolts
```

### Asset pipeline

1. Generate the source with the built-in imagegen tool.
2. Inspect the result for a single straight-on frame, a clean hollow center, forbidden content, and usable downscaled detail.
3. Copy the selected source into a temporary project workspace.
4. Run the installed imagegen `remove_chroma_key.py` helper with border sampling, soft matte, and despill.
5. Validate an alpha channel, transparent corners and center, subject coverage, and the absence of green fringe.
6. Store the final optimized PNG at `public/hud/mana-bar-frame.png` and remove temporary keyed output before completion.
7. Record the final prompt and asset path in the implementation closeout.

If the generated frame cannot produce a clean transparent center after one targeted retry, stop and request approval before switching to a true-transparency CLI fallback.

## Meter Construction

The generated PNG is a visual overlay, not a data representation.

- The `.track` owns a fixed 144x18 layout box and a dark recessed channel.
- The energy fill sits beneath the bezel in a 130x8 inner channel positioned 7px from the left edge and 5px from the top edge.
- Shared TypeScript meter constants define the 144px frame width, 18px frame height, 7px horizontal inset, 5px vertical inset, 130px usable fill width, and 8px channel height. Rendering and live updates must call the same percentage-to-pixel function.
- A value of 0% renders no energy. A value of 100% fills the complete inner channel without extending beneath the outer edge.
- Claude uses cyan-to-electric-blue energy. Codex uses magenta-to-hot-pink energy. Low mana uses coral-to-red energy.
- Every non-empty fill uses the same two-layer halo, one narrow specular highlight, and one slow energy sweep. Only hue changes by provider or warning state.
- Working state does not alter meter geometry or effect intensity.
- The bezel remains neutral and identical across all rows.

## Provider Bands And Familiars

- Clawd remains beside the Claude section; Nimbus remains beside the Codex section.
- Each sprite sits in a subtle portrait socket formed by a small edge bracket and provider-tinted backlight. The socket is part of the band, not a nested card.
- Idle, working, and hover/move sprite rows remain unchanged.
- Working state adds a restrained pulse to the small provider activity light and portrait backlight.
- The activity light is a compact familiar status symbol, not a labeled button.
- Sprite pixel edges remain crisp. The surrounding shell and bezel remain smooth and modern, making the sprites deliberate character accents rather than setting a retro art direction.

## Data And State Behavior

- `ok`: full-color familiar, header, meters, percentages, and reset values.
- `working`: same meters as `ok`; familiar animation, portrait backlight, and activity signal indicate work.
- `low`: warning-colored energy with the same glow geometry and timing as other fills.
- `stale`: preserve text contrast and layout; mute the energy and portrait accents while showing fetched age. Do not apply a filter that makes the entire section hard to read.
- `absent`: keep the familiar and provider header, then show one readable login hint without a placeholder meter.
- Unknown reset timestamps omit only the reset suffix.
- Codex Pro renders exactly one row labeled `Weekly`. No `5 hour` label is synthesized for Codex.
- Claude renders only the rows supplied by its provider data.

## Motion And Accessibility

- The energy sweep is slow and low-amplitude. It must not resemble a busy scanline or change perceived fill width.
- Active-state pulsing is limited to the activity light and portrait backlight.
- Existing sprite animation remains the most expressive motion.
- Under `prefers-reduced-motion: reduce`, energy sweep, active pulse, and sprite animation stop. Static fill, glow, status color, and readable values remain.
- Text and critical status information must meet usable contrast against both native vibrancy and the CSS fallback background.
- No meaning depends on glow or animation alone; percentage text and labels remain authoritative.

## Window And Interaction Behavior

- The roster is visible at launch and never collapses.
- Hover and mouse exit update familiar state only; neither changes window geometry.
- Deep drag-region behavior remains available across non-interactive shell space.
- Content-height measurement and the serialized native resize queue remain.
- Startup and subsequent resizes clamp the 440px window to the active monitor work area.
- Saved positions created by narrower builds must not leave the wider window beyond the right screen edge.
- The implementation does not add click targets or controls.

## Implementation Boundaries

- Keep Tauri 2, vanilla TypeScript, HTML, and CSS.
- Keep provider polling, event payloads, renderer escaping, countdown updates, activity events, and saved-position behavior.
- Keep the permanent expanded layout introduced by the prior v0.4 work.
- Update geometry constants, Tauri initial dimensions, tests, and native radius together.
- Remove `public/hud/boss-bar-left.png`, `boss-bar-mid.png`, and `boss-bar-right.png` after the generated frame is integrated.
- Remove styles that depend on pixelated HUD pieces, the 4px retro grid, stepped fill transition, square retro signal, and all-monospace card typography.
- Do not add an attribution notice for generated original artwork. If a notice was added solely for the removed asset-pack files, remove it.
- Synchronize package and bundle metadata to v0.4.0 when the complete redesign is ready to install.
- Leave the user's untracked `.DS_Store` untouched.

## Verification

### Automated

- Write failing tests before changing meter geometry or provider-label behavior.
- Verify percentage-to-pixel output at 0%, 1%, 29%, 30%, 50%, 99%, and 100% using the new channel width.
- Verify every rendered track uses the fixed meter class and Codex renders one `Weekly` row.
- Keep renderer escaping, absent-state, content-height, work-area clamp, and serialized-queue recovery coverage.
- Run `npm test` and `npm run build`.
- Run `rustfmt --edition 2021 --check` for changed Rust files, `cargo test`, and `cargo check` in `src-tauri`.
- Run `git diff --check`.

### Asset QA

- Confirm the final PNG has RGBA or equivalent alpha support.
- Confirm all four corners and the full center channel are transparent.
- Confirm there is no visible chroma-key fringe at 1x and 2x rendering scales.
- Confirm the frame contains no text, logos, fill, watermark, or perspective distortion.
- Confirm bezel detail survives rendering at exactly 144x18 logical pixels.

### Browser And Native QA

- Launch shows the complete roster without pointer input or compact-state flash.
- Inspect every `.track` and confirm a 144x18 logical-pixel bounding box.
- Verify `100% · Sun 12:51 PM`, three Claude rows, and weekly-only Codex fit without clipping, overlap, ellipsis, or scrollbars.
- Verify 0%, normal, low, working, stale, absent, and reduced-motion states.
- Confirm Claude, Codex, and low fills share identical glow geometry and motion timing.
- Confirm the image-generated bezel stays aligned over the energy core from 0% through 100%.
- Verify dragging, restart position persistence, content-height changes, and right-edge containment.
- Capture and inspect the actual Tauri window at Retina scale for native glass, alpha edges, bezel sharpness, and text legibility.
- Confirm only one mana process is running for the final installed smoke test.

## Acceptance Criteria

- The widget reads as a modern premium gaming HUD, not a retro pixel panel or generic dashboard.
- The full provider roster is always visible and never changes size on hover.
- Every usage meter is exactly 144x18 logical pixels and uses the same original generated bezel.
- Meter fill remains numerically accurate, code-driven, and visibly contained from 0% through 100%.
- Full labels, percentages, and normal reset values remain readable without clipping.
- Codex Pro displays one `Weekly` row and no incorrect `5 hour` label.
- Clawd and Nimbus sit beside their respective provider sections and retain their existing animation states.
- Provider, warning, working, stale, absent, and reduced-motion states remain distinct and readable.
- The final transparent bezel is stored in the project and the rejected third-party HUD pieces are removed.
- Polling, credentials, activity state, tray behavior, dragging, saved position, content sizing, and edge containment continue to work.
