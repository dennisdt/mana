# Mana Frame Safe Area And Structural Corners Design

**Date:** 2026-07-27

## Goal

Give the generated application frame enough room to read as an armored
perimeter without crowding the provider roster, mana rows, reset times, or XP
footer. Replace the repeated crest badges at the four corners with the
directional corner pieces already shipped in each rank kit.

## Root Causes

- The frame occupies a 32px corner zone, but `#card` keeps only 16px of
  horizontal padding. Provider art crowds the left frame while reset times can
  approach the right rail.
- The 456px widget width leaves no room to increase that padding without
  shrinking the existing 424px content area.
- Rank rendering assigns the same wide `crest-top` image to every corner.
  Those repeated crests sit on opaque square surfaces with rectangular box
  shadows, so the corners look like detached badges instead of rail joints.
- The correct directional rank corner assets are resolved and exposed as CSS
  variables, but the current corner CSS does not render them.

## Approved Direction

Use structural directional corners and slightly enlarge the native widget.
Preserve the current sprite, meter, label, and timestamp sizes.

Alternatives rejected:

- Refining the repeated crest caps would retain the pasted-on badge shape.
- Generating another corner set is unnecessary because every rank kit already
  contains the correct directional art.

## Window And Safe-Area Geometry

- Increase the authored roster width from 456px to 488px.
- Keep the current 424px usable provider-content width by increasing
  `#card` horizontal padding from 16px to 32px.
- Keep the frame attached to the native window edge; do not restore external
  frame bleed or a translucent outer band.
- Increase the card's vertical padding to `24px 32px 20px`. The top spacing
  keeps sprite overflow away from the top rail, and the bottom spacing keeps
  the provider divider clear of the footer.
- Keep the footer's frame-aware 36px horizontal inset. Increase its vertical
  breathing room to `10px 36px 18px` when frame art is present.
- Continue deriving native height from measured content. The window may grow
  vertically to contain the added padding, but no fixed-height crop is allowed.
- Single-provider and two-provider layouts use the same safe-area rules.

## Structural Corner Rendering

- Rank frames render `corner-tl`, `corner-tr`, `corner-bl`, and `corner-br`
  in their matching corner elements.
- Each corner remains a 32px clipping box. The existing 96px source art is
  displayed at 48px by 48px and aligned toward its outer corner so its socket
  arms are clipped while its structural joint remains visible.
- Horizontal and vertical rails continue beneath the corner art, preserving a
  single connected perimeter.
- Rank `crest-top` art is no longer repeated in the corners. It remains
  available to the asset model for future top-center use, but this change does
  not add a new crest placement.
- Prestige frames keep their directional `corner-joint-*` surfaces and
  prestige rail precedence. When prestige is active, its directional joint is
  the only corner face; rank corner art must not show through it.
- Corner elements use transparent backgrounds. Remove the opaque square face,
  inset rectangular shadows, and ordinary `box-shadow`.
- Apply one restrained `filter: drop-shadow(0 0 4px var(--cap-glow))` so the
  glow follows the alpha silhouette of the corner art or prestige joint.
- Existing frame animation, Reduced Motion behavior, rank fallback logic, and
  generated source assets remain unchanged.

## Rendering And Failure Behavior

- `frameRenderPlan` continues publishing the four directional rank and
  prestige corner URLs.
- A missing or incomplete rank kit continues falling back through the existing
  resolver. This change does not introduce partial corner rendering.
- A prestige kit remains all-or-nothing. If it is incomplete, the existing
  lower-prestige fallback applies.
- Corner rendering must not affect layout measurement; it remains an absolute,
  pointer-inert perimeter layer.

## Test-First Verification

Add failing regression tests before production changes:

- `window-layout.test.ts` expects a 488px authored roster width and scaled
  native width.
- `styles.test.ts` expects 32px card side padding, the approved frame-aware
  footer padding, transparent corner surfaces, alpha-following drop shadows,
  directional corner variables, and no rectangular corner `box-shadow`.
- `frame-renderer.test.ts` proves each rank corner receives its matching
  directional asset and that rank crests are not repeated as corner emblems.
- Existing frame-asset completeness, prestige precedence, provider overflow,
  animation, and Reduced Motion tests remain green.

After automated tests:

- Build the production frontend and Rust application.
- Inspect browser previews for Platinum, Godlike, Prestige I, and Prestige X.
- Check both-provider, Claude-only, and Codex-only layouts.
- Confirm sprites, timestamps, mana bars, and footer copy stay inside the safe
  area at the 488px root width.
- Confirm all four corners connect to both rails without square backing plates,
  protruding socket arms, or rank bleed-through under prestige.
- Rebuild, sign, and launch the verified local `Mana.app`.

## Non-Goals

- Do not redraw or regenerate frame assets.
- Do not resize sprites, auras, mana bars, typography, or provider columns.
- Do not change progression, prestige, activity detection, or persistence.
- Do not reintroduce external window bleed or a separate decorative shell.
