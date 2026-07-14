# mana v0.4 - Whimsical Fantasy Gaming HUD

2026-07-13 - approved by Dennis

Approved direction: replace the rejected sci-fi bezel with an original bright fantasy-RPG interface inspired by the warmth, ornament, and readability of whimsical side-scrolling MMO UI. Keep the modern smoked-glass container, permanent expanded roster, equal-length bars, and existing pixel mascots.

This document supersedes the visual direction and generated-asset contract in `2026-07-13-mana-modern-gaming-hud-design.md`. It does not copy MapleStory logos, characters, icons, typography, or exact UI assets. The reference is used only for high-level qualities: friendly fantasy ornament, jewel-toned magic, bright readability, and playful polish.

## Goal

Make mana feel like a polished fantasy-game status panel rather than a retro pixel HUD, generic dashboard, or hard sci-fi instrument. The widget must expose all provider usage at a glance, render every meter at the same length, keep complete percentages and reset times readable, and use original image-generated artwork.

## Scope

### In scope

- Keep the roster permanently expanded and draggable.
- Use a 440px logical window with content-measured height and monitor-edge containment.
- Generate one original fantasy mana-bar frame with silver structure, restrained gold filigree, crystal end caps, and subtle leaf or wing motifs.
- Render every meter at exactly 144x20 logical pixels.
- Place a code-driven 116x8 live energy core at x=14, y=6 over the generated frame's dark recessed center.
- Keep meter percentage, color, low state, zero state, glow, and motion controlled by application code.
- Keep Claude and Codex energy colors distinct while applying the same glow and magical glint to every non-empty core.
- Preserve Clawd and Nimbus as pixel familiars beside their provider sections.
- Restyle the shell, portrait slots, dividers, type, stale state, absent state, and activity signals as one modern fantasy system.
- Keep Codex Pro weekly-only as one `Weekly` row with no synthesized `5 hour` row.
- Remove the rejected sci-fi candidate and the imported metroidvania boss-bar pieces.

### Out of scope

- Copying any MapleStory asset, character, logo, icon, typeface, or exact frame composition.
- Replacing or regenerating Clawd and Nimbus.
- Changing provider polling, authentication, credentials, activity detection, tray behavior, or saved-position behavior.
- Adding compact mode, settings, pin controls, sounds, notifications, clicks, or keyboard controls.
- Runtime single-instance enforcement.
- New providers, inferred limits, or framework migration.

## Visual Direction

The visual language combines a quiet macOS glass utility with a bright, friendly fantasy-game focal layer.

- The shell remains smoked charcoal glass with an 8px radius, thin silver border, restrained top highlight, and sparse gold corner accents.
- The fantasy bar is the richest element: polished silver, small warm-gold filigree, faceted opal crystal end caps, and subtle leaf or wing carving.
- Ornament stays compact and symmetrical. The bar must read clearly at utility size and may not become a miniature illustration.
- The center is a simple dark recessed channel. It contains no runes, plates, posts, icon, or decorative object that competes with the live energy core.
- Claude energy is cyan-to-electric-blue. Codex energy is magenta-to-hot-pink. Low mana is coral-to-red.
- Provider headings use the system sans-serif stack. Labels, percentages, and countdowns use a tabular system-monospace stack.
- Provider bands remain unframed full-width sections separated by a hairline. There are no cards inside the glass container.
- The palette balances neutral glass, silver, small gold details, provider energy colors, warning coral, and the existing mascot colors. Gold is an accent rather than the dominant background hue.

## Layout

- Window width: 440 logical pixels.
- Initial height: 175 logical pixels; runtime height is measured from rendered content.
- Shell radius: 8 logical pixels, matched by native vibrancy.
- Shell padding: 14px 16px.
- Provider layout: `44px minmax(0, 1fr)` with an 11px gap.
- Usage-row layout: `52px 144px minmax(0, 1fr)` with 8px gaps.
- Meter frame: exactly 144x20 logical pixels.
- Live core: exactly 116x8 logical pixels positioned 14px from the left and 6px from the top.
- The final column is right-aligned and must fit `100% · Sun 12:51 PM` without clipping, ellipsis, overlap, or horizontal scroll.
- Each familiar occupies a 44px portrait rail beside its own provider band.

## Generated Fantasy Frame

Use built-in imagegen to create one original neutral frame. The final production asset is a 288x40 Retina PNG at `public/hud/mana-bar-frame.png`, rendered by CSS at 144x20 logical pixels.

### Artwork requirements

- One isolated horizontal fantasy mana-bar frame, viewed perfectly straight-on with no perspective.
- Friendly high-polish fantasy MMORPG UI treatment: bright silver structure, restrained warm-gold filigree, small faceted opal crystal end caps, and compact leaf or wing motifs.
- Symmetrical silhouette and consistent thickness so the same frame works for every provider and row.
- A plain, continuous, dark charcoal recessed center large enough to hold the complete 116x8 live core.
- Outer background becomes transparent after chroma-key removal. The center remains opaque and dark; it does not need transparency.
- No energy fill, text, letters, numbers, logos, icons, characters, scenery, cast shadow, reflection, particles, watermark, or green material.
- No hard-sci-fi machinery, pixel-art treatment, grimdark spikes, oversized gems, oversized wings, chunky bolts, or copied game asset.
- Detail must remain recognizable at 144x20 logical pixels and crisp at 2x Retina scale.

### Generation prompt

Use case: `stylized-concept`

```text
Asset type: original fantasy game HUD element for a desktop usage widget
Primary request: Create one polished whimsical fantasy mana-bar frame, isolated and viewed perfectly straight-on.
Subject: A symmetrical bright-silver frame with restrained warm-gold filigree, small faceted opal crystal end caps, and subtle compact leaf or wing carvings. Include one plain continuous dark-charcoal recessed center channel for a live magical energy layer placed on top later.
Style/medium: friendly high-end 2D fantasy MMORPG interface asset with softly dimensional painted materials, crisp readable edges, playful polish, and no direct reference to any existing game asset
Composition/framing: one centered horizontal frame, orthographic front view, approximately 7.2:1 outer silhouette, occupying most of the canvas width. Keep all ornament outside the central safe rectangle. The center channel must be uninterrupted and visually simple.
Lighting/mood: bright soft fantasy-game highlights with controlled silver shine and subtle jewel sparkle; no cast shadow
Color palette: silver, pearl white, small warm-gold accents, opal highlights, and a dark neutral center only
Scene/backdrop: perfectly flat solid #00ff00 chroma-key outer background for removal. The central recessed channel stays dark charcoal, not green.
Constraints: one frame only; symmetrical; plain continuous center; compact ornament; no perspective; no background variation; no green anywhere in the frame
Avoid: energy fill, text, letters, numbers, logos, icons, characters, scenery, gradients or texture in the outer background, floor plane, cast shadow, contact shadow, reflection, particles, watermark, pixel art, hard sci-fi machinery, grimdark spikes, oversized crystals, oversized wings, oversized bolts, copied game UI
```

### Asset pipeline

1. Generate the source with built-in imagegen and no reference image.
2. Inspect it for one straight-on fantasy frame, forbidden content, a plain dark center, and readable ornament.
3. Copy the selected keyed source into ignored `.superpowers/imagegen/` scratch.
4. Remove only the flat outer green background with the installed `remove_chroma_key.py` helper using border sampling, soft matte, and despill.
5. Measure alpha bounds without changing pixels, crop with macOS `sips`, and normalize to 288x40.
6. Validate RGBA mode, transparent corners, plausible frame coverage, symmetry, no green fringe, and a sufficiently dark central safe rectangle.
7. Inspect the final 288x40 PNG and its real CSS rendering at 144x20 before committing it.
8. Record the final prompt, built-in mode, production path, dimensions, and SHA-256 in the closeout.

The center is deliberately not required to be transparent. CSS places the live core over the dark recess, so image-generation variance at the inner edge cannot corrupt percentage math. A targeted built-in edit may refine ornament or clear the center while preserving the approved exterior. Do not switch to a CLI model without explicit approval.

## Meter Construction

- `.track` owns a fixed 144x20 logical box and displays the generated PNG as its frame/background layer.
- `.fill` is a separate 116x8 code-driven layer at x=14, y=6 above the dark center but inside the decorative border.
- Shared TypeScript constants define width, height, insets, channel width, and channel height. Initial rendering and live updates use the same `meterFillPixels()` function.
- A value of 0% renders no energy or glow. A value of 100% fills all 116 channel pixels without covering the frame.
- Every non-empty fill receives the same two-layer glow, narrow highlight, and slow magical glint. Only provider or warning hue changes.
- The generated silver/gold frame remains identical across every row.
- Working state does not change meter geometry, glow strength, or animation timing.

## Familiars And Provider Bands

- Clawd remains beside Claude; Nimbus remains beside Codex.
- Each familiar sits in a compact gemstone portrait bracket formed by silver lines, one small gold accent, and provider-colored backlight.
- The portrait bracket is part of the unframed provider band, not a nested card.
- Existing idle, working, and hover/move sprite rows remain unchanged.
- Working state pulses the small activity gem and portrait backlight.
- Sprite pixels remain crisp while the surrounding frame and glass remain smooth, making the mascots deliberate fantasy companions rather than a retro theme.

## Data And State Behavior

- `ok`: full-color familiar, header, frame, energy core, percentage, and reset value.
- `working`: same meter as `ok`; familiar animation, portrait backlight, and activity gem indicate work.
- `low`: coral-to-red core with the same glow geometry and glint timing.
- `stale`: preserve label/value contrast; mute only energy and portrait accents while showing fetched age.
- `absent`: keep familiar and provider header, then show one readable login hint without a placeholder meter.
- Unknown reset timestamps omit only the reset suffix.
- Codex Pro renders exactly one `Weekly` row. Frontend code must not filter legitimate provider rows by plan name.

## Motion And Accessibility

- The magical glint is slow, restrained, and identical across all non-empty provider and low-state fills.
- Active pulsing is limited to the activity gem and portrait backlight.
- Existing sprite animation remains the most expressive motion.
- Under `prefers-reduced-motion: reduce`, magical glint, active pulse, and sprite animation stop. Static energy, glow, labels, and status colors remain.
- No meaning depends on glow, crystal color, or animation alone; text remains authoritative.

## Window And Interaction Behavior

- The roster is visible at launch and never collapses.
- Hover and mouse exit update familiar state only; neither changes window geometry.
- Deep drag-region behavior remains available across non-interactive shell space.
- Content-height measurement and serialized native resize queue remain.
- Startup and subsequent resizes clamp the 440px window to the active monitor work area.
- Saved positions created by narrower builds must not leave the wider window beyond the right edge.
- No click target or control is added.

## Implementation Boundaries

- Keep Tauri 2, vanilla TypeScript, HTML, and CSS.
- Keep provider polling, event payloads, renderer escaping, countdown updates, activity events, and saved-position behavior.
- Keep the permanent expanded layout already present in the branch.
- Update meter and window constants, Tauri initial width, CSS, and tests together.
- Remove the rejected untracked sci-fi candidate before generating the fantasy asset at the same production path.
- Remove `public/hud/boss-bar-left.png`, `boss-bar-mid.png`, and `boss-bar-right.png` only after CSS references the new fantasy frame.
- Remove retro grid, stepped fill transition, all-monospace card typography, and global stale filter.
- Do not add third-party attribution for original generated artwork.
- Synchronize package and bundle metadata to v0.4.0 after all visual checks pass.
- Leave the user's untracked `.DS_Store` untouched.

## Verification

### Automated

- Write failing tests before changing meter geometry, zero-energy state, or window width.
- Verify percentage-to-pixel output at 0%, 1%, 29%, 30%, 50%, 99%, and 100% using the 116px channel.
- Verify Codex weekly-only remains exactly one `Weekly` row.
- Verify the stylesheet references only the generated fantasy frame, defines 144x20 meter geometry, avoids a global stale filter, and covers reduced motion.
- Keep renderer escaping, absent state, content height, work-area clamp, and serialized-queue recovery coverage.
- Run `npm test`, `npm run build`, `cargo test`, `cargo check`, and `git diff --check`.

### Asset QA

- Confirm the final PNG is 288x40 RGBA with transparent outer corners.
- Confirm the outer silhouette is approximately 7.2:1, symmetrical, and free of visible green fringe.
- Confirm the central 232x16 physical safe rectangle corresponding to the 116x8 logical core is dark, visually quiet, and contains no bright ornament.
- Confirm there is no text, logo, character, fill, watermark, perspective, hard-sci-fi machinery, or copied asset.
- Confirm silver, gold, crystal, and carved details survive at 144x20 logical pixels.

### Browser And Native QA

- Launch shows the complete roster without pointer input or compact-state flash.
- Every `.track` reports a 144x20 logical bounding box.
- `100% · Sun 12:51 PM`, three Claude rows, and weekly-only Codex fit without clipping, overlap, ellipsis, or scrollbars.
- Verify 0%, normal, low, working, stale, absent, and reduced-motion states.
- Confirm all non-empty cores share identical glow geometry and glint timing.
- Confirm the live core stays visually inside the generated frame from 0% through 100%.
- Verify dragging, restart position persistence, content-height changes, and right-edge containment.
- Capture and inspect the actual Tauri window at Retina scale for glass, alpha edges, frame sharpness, and text legibility.
- Confirm only one mana process is running for the final installed smoke test.

## Acceptance Criteria

- The widget reads as an original bright whimsical fantasy-game HUD, not hard sci-fi, retro pixel UI, or generic dashboard.
- The frame evokes friendly fantasy MMO ornament without copying MapleStory assets or identity.
- The full provider roster is always visible and never changes size on hover.
- Every usage meter is exactly 144x20 logical pixels and uses the same original generated frame.
- Meter fill remains numerically accurate, code-driven, and visibly contained from 0% through 100%.
- Full labels, percentages, and normal reset values remain readable without clipping.
- Codex Pro displays one `Weekly` row and no incorrect `5 hour` label.
- Clawd and Nimbus remain beside their provider sections with existing animation states.
- Provider, warning, working, stale, absent, and reduced-motion states remain distinct and readable.
- The rejected sci-fi candidate and third-party boss-bar pieces do not ship.
- Polling, credentials, activity state, tray behavior, dragging, saved position, content sizing, and edge containment continue to work.
