# Mana Compact Integrated Frame Design

**Date:** 2026-07-23

## Goal

Restore Mana's compact widget footprint while keeping the generated fantasy
frame identity. Characters must sit inside their animated auras, corner motifs
must read as part of the perimeter, lifetime output must appear only as an XP
bar tooltip, and the native window must not expose a translucent halo outside
the widget.

## Root Causes

- `FRAME_BLEED = 24` makes the native window 48px wider and taller than the
  authored roster. macOS vibrancy fills that entire window while `#glass`
  remains inset, exposing a translucent rectangle around the frame.
- Generated 48px corner images include horizontal and vertical socket arms.
  Rendering the complete image makes those arms protrude past the rail joint.
- Body-wide hover sets `data-hovering`, replacing the visible level/prestige
  label whenever the pointer is anywhere over the widget.
- Aura atlases place their ground effect near the bottom of a 96px cell. The
  current center registration leaves a visible gap below the character.

## Window And Glass

- Set the frame bleed to zero and return the native widget width to the
  existing 456px roster width.
- Size the native window from the content layout box without adding external
  frame space.
- Make `#glass` fill the root window. The CSS glass and native vibrancy then
  share one boundary and one 8px corner radius.
- Keep the content grid, meter widths, typography, and provider visibility
  behavior unchanged.

## Integrated Perimeter

- Keep the generated rank and prestige rail/corner assets.
- Render each corner inside a 32px square while retaining the existing 48px
  image scale. Directional background positioning clips away the horizontal
  and vertical socket extensions while preserving the central gem/starburst.
- Place horizontal and vertical rail centers at 16px from the corresponding
  window edges.
- Extend rails beneath the corner motifs to their centers. The motif is drawn
  above the rail, producing a single continuous perimeter without detached
  extensions.
- Preserve the existing rank/prestige asset resolution, Prestige VII-X light
  animation, asymmetric animation timing, and Reduced Motion behavior.
- The generated footer prestige crest remains the only prestige badge.

## Character And Aura Registration

- Raise the aura layer 8px without moving the provider grid or
  character sprites.
- Keep Claude and Codex animation phases independent.
- Confirm both characters appear planted inside the aura ground across visible
  high-tier frames.
- Visual overflow must not contribute extra height to native window sizing.

## Lifetime Output Tooltip

- Keep the level, rank, and prestige label visible at all times.
- Remove the body-hover footer copy swap.
- Anchor a custom tooltip to the XP bar. It appears only while the XP bar is
  hovered and displays the exact comma-formatted lifetime output count.
- The tooltip floats above the XP bar without resizing or shifting the footer.
- Retain the lifetime output value in the DOM for tooltip accessibility, but
  keep it visually hidden when the XP bar is not hovered.

## Verification

No TDD or test authoring is required. Verify through production-oriented
checks:

- TypeScript/Vite production build and Rust release check.
- Browser screenshots for two-provider, Claude-only, and Codex-only layouts.
- DOM geometry confirming a 456px root, glass flush to root, stable footer
  geometry, and XP-tooltip-only lifetime output.
- Pixel inspection confirming rail centers meet all four isolated corner
  motifs without protruding arms.
- Standard and Reduced Motion inspection.
- Build-only macOS application bundle with `LSUIElement=true`; do not install
  or launch it against live progression.
