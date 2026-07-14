# Mana Elemental Mage Familiars Design

**Date:** 2026-07-13
**Status:** Approved design, pending implementation plan

## Goal

Replace the current pixel mascots with original, animated chibi mage characters informed by the user-provided references and the visual language of bright Korean side-scrolling fantasy RPGs. Codex becomes an ice/lightning mage and Claude becomes a fire/poison mage. The characters stand freely beside their provider sections without portrait frames or pedestals.

The existing glass HUD, permanent expanded roster, usage rows, mana-bar geometry, provider status logic, and window positioning remain unchanged.

## Creative Direction

The references define each character's role, silhouette, palette, and magical vocabulary. The production art must remain original: do not reproduce game logos, named equipment, UI elements, copyrighted character costumes, or exact source-image details.

Both characters use a polished 2D chibi game-sprite treatment rather than the current hard-edged pixel art. They have oversized heads, compact bodies, clean dark outlines, painted highlights, and readable silhouettes at widget scale.

### Codex: Ice/Lightning Mage

- Preserve the cloud-terminal identity from the first reference without copying its exact costume.
- Use a deep royal-blue and icy-cyan robe with restrained gold trim.
- Include a crystalline staff, frost motifs, and sharp blue-white electrical accents.
- Idle frames show a calm breathing or floating motion.
- Working frames show a compact ice/lightning cast with the staff and a controlled spark burst.
- Hover frames show an upbeat victory pose with the staff raised and a small snowflake sparkle.

### Claude: Fire/Poison Mage

- Preserve the warm star-like familiar identity from the second reference without copying its exact costume.
- Use ivory, warm orange, charcoal, and gold, with violet poison accents.
- Include a curved wooden staff and small leaf or ember motifs.
- Idle frames show calm breathing and a gentle robe or leaf movement.
- Working frames show alternating ember-orange fire and violet poison energy around the staff.
- Hover frames show a cheerful victory pose with a small flame-and-leaf flourish.

## Asset Contract

Use the built-in image generation workflow with each attached image as the corresponding character reference. Generate on a perfectly flat `#ff00ff` chroma-key background because both characters use blue, green-adjacent, and warm colors. The background must have no shadows, gradients, texture, grid, floor plane, or reflections. Effects must use crisp opaque or near-opaque shapes so local background removal does not destroy spell edges.

Create two production atlases:

- `public/sprites/codex-ice-lightning.png`
- `public/sprites/claude-fire-poison.png`

Each atlas is a `4 x 3` grid:

- Physical size: `448 x 336` RGBA PNG.
- Cell size: `112 x 112` physical pixels.
- CSS frame size: `56 x 56` logical pixels at Retina scale.
- Columns: animation frames 1 through 4.
- Row 1: `idle` breathing loop.
- Row 2: `working` spell-casting loop.
- Row 3: `hover` victory loop.

Every frame keeps the character centered on the same baseline with consistent scale, silhouette, costume, staff, outline thickness, and lighting. Particles must stay inside the cell with at least four physical pixels of transparent padding. No frame may contain text, logos, checkerboards, cell labels, separators, watermarks, cast shadows, or cropped equipment.

Normalize the accepted generated sources into the exact atlas geometry, remove the chroma key with the installed imagegen helper, and validate alpha, dimensions, cell alignment, corner transparency, subject coverage, and visible fringe before integration. If either character drifts materially between frames, perform one targeted imagegen iteration rather than correcting character identity with hand-painted replacements.

The obsolete assets `public/sprites/clawd.png` and `public/sprites/nimbus.png` are deleted only after all runtime and test references use the new atlases. Delete `scripts/gen-sprites.py` because it deterministically regenerates the retired pixel mascots and would otherwise overwrite the new art.

## UI Integration

Keep the existing provider-owned familiar wrapper in the markup as a layout anchor, but remove all visible wrapper chrome:

- No `.familiar-slot::before` portrait panel.
- No `.familiar-slot::after` pedestal or diamond.
- No border, background, clipping shape, or container glow around either character.
- A restrained provider-colored drop shadow may be applied directly to the sprite.

Rename the sprite classes to match their roles:

- Claude: `sprite claude-mage`
- Codex: `sprite codex-mage`

Render the wrapper as a `60px` layout column with a `56 x 56` sprite. Use the source atlases at half scale so the artwork remains sharp on Retina displays. Remove `image-rendering: pixelated`; the new assets are smooth illustrated sprites.

The current state machine remains authoritative:

- `idle` when the provider is inactive and the widget is not hovered or moving.
- `working` while local provider activity is detected.
- `hover` while the pointer is over the widget or for the existing short window-move interval.
- Priority remains `hover > working > idle`.

The CSS uses the atlas rows for state selection and `steps(4)` for horizontal playback. Target timing:

- Idle: `1.15s`, calm and readable.
- Working: `0.68s`, energetic spell loop.
- Hover: `0.82s`, celebratory without flashing.

`prefers-reduced-motion: reduce` freezes the current state on its first frame. Stale status continues to mute only energy and portrait-adjacent status accents; character art and text remain readable.

The provider header activity diamond remains. Its existing working pulse is independent of the removed familiar container chrome.

## Layout

Increase the provider grid's familiar column from `44px` to `60px` and retain a compact gap to the provider content. The characters may extend visually into their section's internal whitespace but must not overlap the header, usage labels, bars, values, section divider, or widget border.

The native window remains `440px` wide and content-measured in height. The free-standing `56px` Codex character is expected to make the one-row Codex section slightly taller; the existing serialized content-resize path must calculate and apply that height without reintroducing a compact view.

All usage behavior remains binding:

- Claude shows all returned duration-derived rows.
- Codex Pro shows exactly one `Weekly` row.
- Every mana track remains `144 x 20` with a `116 x 8` live channel.
- Empty bars retain no fill glow.
- Text stays untruncated and non-overlapping at `440px`.

## Testing And Visual QA

Add or update automated coverage for:

- New semantic sprite classes and asset URLs.
- Exact `56 x 56` logical frame and `224 x 168` CSS background geometry.
- State row offsets `0`, `-56px`, and `-112px`.
- `steps(4)` animation and the three durations.
- Removed portrait and pedestal pseudo-elements.
- Reduced-motion freeze behavior.
- PNG width `448`, height `336`, RGBA color type, and transparent corners.
- Absence of runtime references to `clawd.png`, `nimbus.png`, or `gen-sprites.py`.

Run the existing frontend and Rust suites and build gates. Use a browser fixture at the native `440px` width to inspect idle, working, and hover rows for both characters, including exact element rectangles and overflow. Inspect the generated atlases and a frame-expanded contact sheet before building the native release.

Build and install `v0.4.1`, backing up the current `/Applications/mana.app` first. Launch exactly one installed instance and capture the native Retina window. Confirm the free-standing characters are sharp, fully visible, animated in the correct provider sections, and do not reduce the readability of labels, bars, percentages, or reset times.

## Release Scope

Expected implementation scope:

- Add the two generated mage atlases.
- Remove the two retired pixel atlases and their deterministic generator.
- Update sprite markup, CSS, and focused tests.
- Update release documentation and synchronize version metadata to `0.4.1`.
- Rebuild, replace, and launch the installed macOS app.

Do not change usage parsing, polling, credential access, activity detection, tray behavior, persistence, mana calculations, or native window activation policy.
