# mana Whimsical Fantasy Gaming HUD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace mana's rejected sci-fi and retro treatments with a permanent 440px smoked-glass roster using one original image-generated fantasy frame, accurate 144x20 meters, readable values, and bright whimsical RPG ornament.

**Architecture:** Keep Tauri 2 and the current vanilla TypeScript renderer. Imagegen produces one silver/gold/crystal frame with an opaque dark recess, while TypeScript remains authoritative for usage geometry and CSS places a live energy core over that recess. Preserve the existing polling and permanent-roster lifecycle, changing only the generated asset, meter contract, window width, visual styling, documentation, and release metadata.

**Tech Stack:** Tauri 2, Rust 2021, vanilla TypeScript 5.6, CSS, Vite 6, Vitest 3, native macOS `HudWindow` vibrancy, built-in imagegen, installed chroma-key helper, macOS `sips`, Pillow/NumPy for read-only pixel validation, in-app browser control, and `cua-driver` for native GUI inspection.

## Global Constraints

- The provider roster is permanently expanded; no compact mode or hover-driven geometry returns.
- Window width is exactly 440 logical pixels; initial height remains 175 and runtime height remains measured from `#card.scrollHeight`.
- Every usage track is exactly 144x20 logical pixels with a 126x8 live core at x=9, y=6.
- `meterFillPixels(percent: number): number` remains the single clamped percentage-to-pixel interface used by initial rendering and live updates.
- The production frame is original generated art at `public/hud/mana-bar-frame.png`, normalized to 288x40 RGBA for Retina rendering.
- Use built-in imagegen and chroma-key removal first. A targeted built-in edit may refine the selected frame. Never switch to CLI or require `OPENAI_API_KEY` without explicit user approval.
- The frame may have an opaque dark center; only the outer canvas must be transparent. Percentage accuracy must never depend on generated center transparency.
- The visual direction is bright friendly fantasy MMORPG ornament: silver structure, restrained gold filigree, small opal crystal caps, and compact leaf or wing motifs. Do not copy MapleStory assets, identity, characters, logos, icons, or exact UI.
- Claude energy remains cyan-to-electric-blue, Codex remains magenta-to-hot-pink, and low mana remains coral-to-red.
- Every non-empty core uses identical halo geometry and magical-glint timing; provider or warning hue is the only core-effect difference.
- Working state affects the familiar portrait and activity gem, not meter geometry or effect intensity.
- Stale styling must not dim labels and values into illegibility.
- Keep Clawd and Nimbus, their provider ownership, and idle/working/hover sprite states.
- Keep duration-aware provider parsing unchanged. Codex Pro weekly-only remains exactly one `Weekly` row; do not add frontend plan-based filtering.
- Preserve polling, credentials, activity detection, tray behavior, dragging, saved position, work-area containment, absent state, countdowns, and renderer escaping.
- Keep native and CSS glass radius at 8 logical pixels.
- Add no runtime dependency. Leave the user's untracked `.DS_Store` untouched.
- Remove the untracked rejected sci-fi candidate before generating the fantasy replacement. Remove the three tracked boss-bar PNGs only after CSS no longer references them.
- Synchronize package, Cargo, and Tauri release metadata to `0.4.0` only after visual implementation passes.
- Ask for explicit action-time approval before quitting or replacing `/Applications/mana.app`. Never use `open`, `kill`, or `pkill` for native app control.

---

## File Structure

- Create `public/hud/mana-bar-frame.png`: original 288x40 fantasy frame rendered at 144x20 logical pixels.
- Create `src/styles.test.ts`: source-level contract for fantasy asset reference, fixed geometry, retired asset removal, readable stale state, and reduced motion.
- Modify `src/meter.ts`: exact outer and live-core geometry plus clamped fill conversion.
- Modify `src/meter.test.ts`: boundary coverage for the 126px core.
- Modify `src/view.ts`: emit the new fill width and explicit zero-energy state.
- Modify `src/view.test.ts`: verify 126px math, zero-energy state, and exactly one weekly Codex row.
- Modify `src/main.ts`: synchronize zero-energy state during live updates.
- Modify `src/window-layout.ts`: widen the roster to 440px.
- Modify `src/window-layout.test.ts`: assert Tauri/TypeScript geometry parity and old-position reclamping.
- Modify `src-tauri/tauri.conf.json`: widen initial native geometry and later set v0.4.0 metadata.
- Modify `src/styles.css`: replace retro styling with smoked glass, fantasy frame integration, portrait brackets, data states, and reduced motion.
- Delete `public/hud/boss-bar-left.png`, `public/hud/boss-bar-mid.png`, and `public/hud/boss-bar-right.png` after integration.
- Modify `README.md`: describe the permanent fantasy roster and original generated frame accurately.
- Modify `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`: set v0.4.0 release metadata.
- Preserve `src-tauri/src/lib.rs`: its `HudWindow` vibrancy radius is already 8.0; verify without no-op churn.

---

### Task 1: Generate and validate the fantasy mana frame

**Files:**
- Replace untracked candidate: `public/hud/mana-bar-frame.png`
- Scratch only: `.superpowers/imagegen/`

**Interfaces:**
- Produces: a 288x40 RGBA PNG at `public/hud/mana-bar-frame.png` with transparent outer corners and an opaque, dark, visually quiet 252x16 center safe rectangle.
- Consumes: built-in `image_gen__imagegen`, optional built-in edit mode, installed `remove_chroma_key.py`, `sips`, and the approved fantasy prompt.
- Preserves: the three tracked boss-bar PNGs until Task 4 changes CSS.

- [ ] **Step 1: Remove only the rejected candidate and prepare ignored scratch**

Run from the repository root:

```bash
if test -e public/hud/mana-bar-frame.png; then
  test "$(git status --porcelain=v1 -- public/hud/mana-bar-frame.png)" = \
    "?? public/hud/mana-bar-frame.png"
  test "$(shasum -a 256 public/hud/mana-bar-frame.png | awk '{print $1}')" = \
    "86936176b619cda243eb0d65559a5645b579c270c4b3c95db6285b1e3fdb5e9f"
  rm public/hud/mana-bar-frame.png
fi
test ! -e public/hud/mana-bar-frame.png
mkdir -p .superpowers/imagegen
git check-ignore -q .superpowers/imagegen
rm -f .superpowers/imagegen/mana-fantasy-frame-*.png
```

Expected: the rejected untracked sci-fi PNG is gone; the three boss-bar PNGs and `.DS_Store` remain untouched.

- [ ] **Step 2: Generate one keyed fantasy frame with built-in imagegen**

Invoke `image_gen__imagegen` with no referenced images and this exact prompt:

```text
Use case: stylized-concept
Asset type: original fantasy game HUD element for a desktop usage widget
Primary request: Create one polished whimsical fantasy mana-bar frame, isolated and viewed perfectly straight-on.
Subject: A symmetrical bright-silver frame with restrained warm-gold filigree, small faceted opal crystal end caps, and subtle compact leaf or wing carvings. Include one plain continuous dark-charcoal recessed center channel for a live magical energy layer placed on top later.
Style/medium: friendly high-end 2D fantasy MMORPG interface asset with softly dimensional painted materials, crisp readable edges, playful polish, and no direct reference to any existing game asset
Composition/framing: one centered horizontal frame, orthographic front view, approximately 7.2:1 outer silhouette, occupying most of the canvas width. Keep all ornament outside the central safe rectangle. The center channel must be uninterrupted and visually simple.
Safe-center geometry: Reserve the centered middle 87.5% of the frame width and middle 40% of the frame height as one uninterrupted opaque dark-charcoal rectangle. No silver, gold, crystal, carving, highlight, seam, post, rune, or bright pixel may enter that rectangle.
Lighting/mood: bright soft fantasy-game highlights with controlled silver shine and subtle jewel sparkle; no cast shadow
Color palette: silver, pearl white, small warm-gold accents, opal highlights, and a dark neutral center only
Scene/backdrop: perfectly flat solid #00ff00 chroma-key outer background for removal. The central recessed channel stays dark charcoal, not green.
Constraints: one frame only; symmetrical; plain continuous center; compact ornament; no perspective; no background variation; no green anywhere in the frame
Avoid: energy fill, text, letters, numbers, logos, icons, characters, scenery, gradients or texture in the outer background, floor plane, cast shadow, contact shadow, reflection, particles, watermark, pixel art, hard sci-fi machinery, grimdark spikes, oversized crystals, oversized wings, oversized bolts, copied game UI
```

Expected: exactly one new built-in output and local path. Do not call a CLI fallback.

- [ ] **Step 3: Stage and inspect the keyed source**

Copy the exact path returned by imagegen:

```bash
cp "$GENERATED_IMAGE" .superpowers/imagegen/mana-fantasy-frame-keyed.png
```

Inspect it with `view_image` at original detail. Require one straight-on frame, friendly silver/gold/crystal fantasy treatment, no copied identity or forbidden content, flat outer green, and a plain dark center. If the exterior is approved but the center contains bright ornament, use one targeted built-in image edit that preserves the complete exterior and replaces only the center with a continuous unornamented dark-charcoal recess.

If the overall object is too tall, has perspective, contains multiple frames, or later fails the 7.0:1 through 7.4:1 silhouette gate, perform one fresh built-in generation retry with no reference image. Reuse the complete approved prompt and add this exact correction:

```text
Composition correction: The prior object was too tall and only about 4:1. Produce an extremely long, thin fantasy status frame whose complete visible silhouette is 7.2:1. Extend the simple center rail horizontally, reduce the vertical height of crystals, leaves, wings, and end caps, and keep every ornament inside a total height no greater than 14% of the visible width. Preserve the same friendly silver, restrained gold, opal, and dark-center art direction.
```

After the retry, replace only `.superpowers/imagegen/mana-fantasy-frame-keyed.png`, re-inspect it, and repeat Steps 4-5. If the fresh retry still fails the silhouette gate, return `BLOCKED`; do not distort it or use a CLI fallback.

- [ ] **Step 4: Remove only the outer chroma background**

```bash
python "${CODEX_HOME:-$HOME/.codex}/skills/.system/imagegen/scripts/remove_chroma_key.py" \
  --input "$PWD/.superpowers/imagegen/mana-fantasy-frame-keyed.png" \
  --out "$PWD/.superpowers/imagegen/mana-fantasy-frame-alpha.png" \
  --auto-key border \
  --soft-matte \
  --transparent-threshold 12 \
  --opaque-threshold 220 \
  --despill
```

Expected: outer canvas is transparent while the dark center remains opaque.

- [ ] **Step 5: Validate and capture alpha bounds without editing pixels**

```bash
python - <<'PY'
from PIL import Image

path = '.superpowers/imagegen/mana-fantasy-frame-alpha.png'
with Image.open(path) as opened:
    image = opened.convert('RGBA')
mask = image.getchannel('A').point(lambda value: 255 if value > 16 else 0)
bbox = mask.getbbox()
if bbox is None:
    raise SystemExit('no visible fantasy frame after chroma removal')
left, top, right, bottom = bbox
width, height = right - left, bottom - top
ratio = width / height
if width < 576 or height < 80:
    raise SystemExit(f'source detail is too small: {width}x{height}')
if not 7.0 <= ratio <= 7.4:
    raise SystemExit(f'fantasy frame silhouette must be near 7.2:1; got {ratio:.2f}:1')
print(left, top, width, height)
PY
```

Expected: four integers, source detail of at least 576x80, and a silhouette ratio from 7.0:1 through 7.4:1 so normalization distortion stays below roughly 3%.

- [ ] **Step 6: Crop and normalize using macOS `sips`**

```bash
read left top crop_width crop_height <<< "$(python - <<'PY'
from PIL import Image
with Image.open('.superpowers/imagegen/mana-fantasy-frame-alpha.png') as opened:
    image = opened.convert('RGBA')
bbox = image.getchannel('A').point(lambda value: 255 if value > 16 else 0).getbbox()
if bbox is None:
    raise SystemExit('no visible fantasy frame after chroma removal')
left, top, right, bottom = bbox
print(left, top, right - left, bottom - top)
PY
)"
sips --cropToHeightWidth "$crop_height" "$crop_width" \
  --cropOffset "$top" "$left" \
  .superpowers/imagegen/mana-fantasy-frame-alpha.png \
  --out .superpowers/imagegen/mana-fantasy-frame-cropped.png
sips --resampleHeightWidth 40 288 \
  .superpowers/imagegen/mana-fantasy-frame-cropped.png \
  --out public/hud/mana-bar-frame.png
```

Expected: 288x40 production PNG with alpha.

- [ ] **Step 7: Run deterministic outer-alpha, center, symmetry, and fringe QA**

```bash
python - <<'PY'
from PIL import Image
import numpy as np

path = 'public/hud/mana-bar-frame.png'
with Image.open(path) as opened:
    assert opened.mode == 'RGBA', opened.mode
    assert opened.size == (288, 40), opened.size
    rgba = np.asarray(opened, dtype=np.int16)

alpha = rgba[:, :, 3]
for patch in (
    alpha[:3, :3], alpha[:3, -3:], alpha[-3:, :3], alpha[-3:, -3:]
):
    assert int(patch.max()) == 0, 'outer corner is not transparent'

visible = alpha > 16
coverage = float(visible.mean())
assert 0.38 <= coverage <= 0.92, f'implausible frame coverage: {coverage:.3f}'
symmetry_error = float(np.mean(visible != visible[:, ::-1]))
assert symmetry_error <= 0.08, f'asymmetric silhouette: {symmetry_error:.3f}'

center = rgba[12:28, 18:270]
center_rgb = center[:, :, :3]
center_alpha = center[:, :, 3]
assert center.shape[:2] == (16, 252), center.shape
assert int(center_alpha.min()) >= 250, 'center safe rectangle is not opaque'
luma = (
    center_rgb[:, :, 0] * 0.2126
    + center_rgb[:, :, 1] * 0.7152
    + center_rgb[:, :, 2] * 0.0722
)
chroma = center_rgb.max(axis=2) - center_rgb.min(axis=2)
assert float(luma.mean()) <= 60, f'center is too bright: {luma.mean():.1f}'
assert float(luma.std()) <= 20, f'center varies too much: {luma.std():.1f}'
assert float(np.percentile(luma, 99)) <= 100, 'bright ornament intrudes into center'
assert float(luma.max()) <= 125, 'center contains an isolated bright pixel'
assert float(np.percentile(chroma, 99)) <= 32, 'center is not neutral charcoal'

red, green, blue = rgba[:, :, 0], rgba[:, :, 1], rgba[:, :, 2]
green_fringe = (
    visible
    & (green > 96)
    & (green > red + 24)
    & (green > blue + 24)
)
assert int(green_fringe.sum()) == 0, \
    f'green fringe pixels: {int(green_fringe.sum())}'

print(
    f'RGBA 288x40; coverage={coverage:.3f}; '
    f'symmetry_error={symmetry_error:.3f}; center_luma={luma.mean():.1f}; '
    'outer corners transparent; no green fringe'
)
PY
file public/hud/mana-bar-frame.png
sips -g pixelWidth -g pixelHeight -g format -g hasAlpha \
  public/hud/mana-bar-frame.png
shasum -a 256 public/hud/mana-bar-frame.png
```

Expected: all assertions pass. Pixel checks are gates for geometry and contamination, not substitutes for visual judgment.

- [ ] **Step 8: Inspect the real 144x20 layering before commit**

Inspect `public/hud/mana-bar-frame.png` with `view_image`, then create ignored `.superpowers/imagegen/frame-preview.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <style>
      html,
      body {
        margin: 0;
        min-height: 100%;
        background: #171923;
      }
      body {
        display: grid;
        min-height: 100vh;
        place-items: center;
      }
      .track {
        position: relative;
        width: 144px;
        height: 20px;
        background: url("/hud/mana-bar-frame.png") center / 100% 100% no-repeat;
      }
      .fill {
        position: absolute;
        top: 6px;
        left: 9px;
        width: 126px;
        height: 8px;
        background: linear-gradient(90deg, #39ddff, #557cff);
        box-shadow: 0 0 4px rgba(57, 221, 255, 0.5), 0 0 10px rgba(57, 221, 255, 0.45);
      }
    </style>
    <title>fantasy frame preview</title>
  </head>
  <body>
    <div class="track"><div class="fill"></div></div>
  </body>
</html>
```

Start `npm run dev -- --host 127.0.0.1 --port 1431 --strictPort`, open `http://127.0.0.1:1431/.superpowers/imagegen/frame-preview.html` in the in-app browser, and capture the actual 144x20 rendering at normal and 2x display scale. Confirm the live core is visible above the opaque recess without covering silver/gold/crystal borders, details survive, and there is no text, logo, character, hard-sci-fi machinery, watermark, copied identity, or green fringe. Stop the exact server with Ctrl-C and remove the preview with `apply_patch`.

- [ ] **Step 9: Commit the production asset**

```bash
git diff --check
git status --short
git add public/hud/mana-bar-frame.png
git commit -m "feat: add original fantasy mana frame"
```

Expected: commit contains only the passing fantasy PNG. Keep ignored imagegen scratch until Task 4 browser QA passes.

---

### Task 2: Implement the 144x20 meter contract and zero-energy state

**Files:**
- Modify: `src/meter.ts`
- Modify: `src/meter.test.ts`
- Modify: `src/view.ts`
- Modify: `src/view.test.ts`
- Modify: `src/main.ts`

**Interfaces:**
- Produces: `METER_WIDTH = 144`, `METER_HEIGHT = 20`, `METER_INSET_X = 9`, `METER_INSET_Y = 6`, `METER_CHANNEL_WIDTH = 126`, `METER_CHANNEL_HEIGHT = 8`, and unchanged `meterFillPixels(percent): number`.
- Produces: `data-empty="true|false"` on `.fill` during initial render and live updates.
- Consumes: `manaLeft(usedPercent)` and existing snapshots.

- [ ] **Step 1: Write failing meter tests**

Replace `src/meter.test.ts` with:

```typescript
import { describe, expect, it } from "vitest";
import {
  METER_CHANNEL_HEIGHT,
  METER_HEIGHT,
  METER_CHANNEL_WIDTH,
  METER_INSET_X,
  METER_INSET_Y,
  METER_WIDTH,
  meterFillPixels,
} from "./meter";

describe("fantasy mana meter", () => {
  it("uses the approved frame and live-core dimensions", () => {
    expect({
      width: METER_WIDTH,
      height: METER_HEIGHT,
      insetX: METER_INSET_X,
      insetY: METER_INSET_Y,
      channelWidth: METER_CHANNEL_WIDTH,
      channelHeight: METER_CHANNEL_HEIGHT,
    }).toEqual({
      width: 144,
      height: 20,
      insetX: 9,
      insetY: 6,
      channelWidth: 126,
      channelHeight: 8,
    });
  });

  it.each([
    [-1, 0],
    [0, 0],
    [1, 1],
    [29, 37],
    [30, 38],
    [50, 63],
    [55, 69],
    [99, 125],
    [100, 126],
    [101, 126],
  ])("maps %s percent to %s core pixels", (percent, pixels) => {
    expect(meterFillPixels(percent)).toBe(pixels);
  });
});
```

- [ ] **Step 2: Strengthen renderer regression cases**

In the weekly-only renderer test, change fill expectation to `57px` and assert one row:

```typescript
expect(html).toContain('class="fill" data-empty="false" style="width:57px"');
expect(html.match(/class="row"/g)).toHaveLength(1);
```

Add:

```typescript
it("marks a depleted meter so zero width cannot retain a glow", () => {
  const depleted = {
    ...weeklyOnly,
    bars: [{ ...weeklyOnly.bars[0], used_percent: 100 }],
  };
  const html = cardHtml(depleted, "codex");
  expect(html).toContain('class="fill" data-empty="true" style="width:0px"');
});
```

The fixture is 55% used, leaving 45%; `round(126 * 0.45) = 57`.

- [ ] **Step 3: Run focused tests and verify RED**

```bash
npx vitest run src/meter.test.ts src/view.test.ts
```

Expected: FAIL on old 128x16/122px geometry and missing `data-empty`.

- [ ] **Step 4: Implement exact meter constants**

Replace `src/meter.ts` with:

```typescript
export const METER_WIDTH = 144;
export const METER_HEIGHT = 20;
export const METER_INSET_X = 9;
export const METER_INSET_Y = 6;
export const METER_CHANNEL_WIDTH = METER_WIDTH - METER_INSET_X * 2;
export const METER_CHANNEL_HEIGHT = METER_HEIGHT - METER_INSET_Y * 2;

export function meterFillPixels(percent: number): number {
  const clamped = Math.max(0, Math.min(100, percent));
  return Math.round((clamped / 100) * METER_CHANNEL_WIDTH);
}
```

- [ ] **Step 5: Emit and synchronize `data-empty`**

Change `barHtml` in `src/view.ts` to:

```typescript
function barHtml(snapshot: Snapshot, bar: Bar, index: number): string {
  const left = manaLeft(bar.used_percent);
  const pixels = meterFillPixels(left);
  return `<div class="track ${snapshot.provider}${left < 30 ? " low" : ""}" data-bar="${index}"><div class="fill" data-empty="${pixels === 0}" style="width:${pixels}px"></div></div>`;
}
```

In `applyData` in `src/main.ts`:

```typescript
const fill = track.querySelector<HTMLElement>(".fill");
if (fill) {
  const pixels = meterFillPixels(left);
  fill.style.width = `${pixels}px`;
  fill.dataset.empty = String(pixels === 0);
}
```

Do not filter or relabel provider rows in the frontend.

- [ ] **Step 6: Verify and commit**

```bash
npx vitest run src/meter.test.ts src/view.test.ts
npm test
npm run build
git diff --check
git add src/meter.ts src/meter.test.ts src/view.ts src/view.test.ts src/main.ts
git commit -m "feat: define fantasy mana meter geometry"
```

Expected: focused/full tests and build pass; one meter-contract commit.

---

### Task 3: Widen the permanent roster to 440px

**Files:**
- Modify: `src/window-layout.ts`
- Modify: `src/window-layout.test.ts`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: `ROSTER_WIDTH = 440`; retains `INITIAL_ROSTER_HEIGHT = 175`.
- Verifies: Tauri startup dimensions match runtime constants and old 420px right-edge positions reclamp.

- [ ] **Step 1: Write failing cross-config tests**

Add:

```typescript
import tauriConfig from "../src-tauri/tauri.conf.json";
```

Replace startup expectation and add migration coverage:

```typescript
it("uses the wider expanded roster from startup", () => {
  expect({ width: ROSTER_WIDTH, height: INITIAL_ROSTER_HEIGHT }).toEqual({
    width: 440,
    height: 175,
  });
  const mainWindow = tauriConfig.app.windows.find(({ label }) => label === "main");
  expect(mainWindow).toMatchObject({
    width: ROSTER_WIDTH,
    height: INITIAL_ROSTER_HEIGHT,
  });
});

it("reclamps an old 420px right-edge position for the wider roster", () => {
  expect(
    rosterOrigin(
      { x: 2040, y: 80 },
      { width: ROSTER_WIDTH, height: 210 },
      { x: 0, y: 0, width: 2880, height: 1800 },
      2,
    ),
  ).toEqual({ x: 2000, y: 80 });
});
```

Use `ROSTER_WIDTH` in existing origin cases and update the both-axes expected x to `2000`.

- [ ] **Step 2: Verify RED, implement, and verify GREEN**

```bash
npx vitest run src/window-layout.test.ts
```

Expected RED on 420. Then set:

```typescript
export const ROSTER_WIDTH = 440;
export const INITIAL_ROSTER_HEIGHT = 175;
```

and in `src-tauri/tauri.conf.json`:

```json
"width": 440,
"height": 175
```

Run:

```bash
npx vitest run src/window-layout.test.ts
npm run build
git diff --check
```

Expected: layout/queue tests and build pass. Do not change `main.ts` or `lib.rs`.

- [ ] **Step 3: Commit geometry**

```bash
git add src/window-layout.ts src/window-layout.test.ts src-tauri/tauri.conf.json
git commit -m "feat: widen the permanent mana roster"
```

---

### Task 4: Integrate the fantasy glass HUD

**Files:**
- Create: `src/styles.test.ts`
- Modify: `src/styles.css`
- Delete: `public/hud/boss-bar-left.png`
- Delete: `public/hud/boss-bar-mid.png`
- Delete: `public/hud/boss-bar-right.png`
- Scratch only: `.superpowers/visual-contract.html`

**Interfaces:**
- Consumes: generated frame, fixed meter constants, existing provider/familiar/state classes, and `.fill[data-empty]`.
- Produces: 144x20 fantasy tracks, shared magical energy effects, modern smoked glass, portrait brackets, readable stale/absent states, and reduced-motion coverage.
- Preserves: existing DOM and sprite-sheet animation rows.

- [ ] **Step 1: Write a failing stylesheet contract**

Create `src/styles.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import styles from "./styles.css?raw";

describe("fantasy gaming HUD stylesheet", () => {
  it("uses only the original generated frame", () => {
    expect(styles).toContain('url("/hud/mana-bar-frame.png")');
    expect(styles).not.toContain("boss-bar-");
    expect(styles).not.toContain("background-size: 4px 4px");
  });

  it("declares fixed frame and live-core geometry", () => {
    expect(styles).toContain("--meter-width: 144px");
    expect(styles).toContain("--meter-height: 20px");
    expect(styles).toContain("--meter-channel-width: 126px");
    expect(styles).toContain("--meter-channel-height: 8px");
    expect(styles).toContain('.fill[data-empty="true"]');
  });

  it("keeps stale text readable and covers reduced motion", () => {
    expect(styles).not.toMatch(/\.stale\s*\{[^}]*\bfilter\s*:/s);
    expect(styles).toContain(".stale .fill");
    expect(styles).toContain("@media (prefers-reduced-motion: reduce)");
    expect(styles).toContain(".provider-card[data-working] .familiar-slot::after");
  });
});
```

- [ ] **Step 2: Run stylesheet test and verify RED**

```bash
npx vitest run src/styles.test.ts
```

Expected: FAIL on old boss assets, retro grid, and old geometry.

- [ ] **Step 3: Replace `src/styles.css` with the fantasy system**

Use this complete stylesheet:

```css
:root {
  --ink: #f7f5ff;
  --ink-dim: #a9afc1;
  --silver: #dbe7f4;
  --gold: #f2c968;
  --line: rgba(219, 231, 244, 0.24);
  --claude-1: #39ddff;
  --claude-2: #557cff;
  --claude-glow: rgba(57, 221, 255, 0.48);
  --codex-1: #d75cff;
  --codex-2: #ff5ba8;
  --codex-glow: rgba(215, 92, 255, 0.46);
  --low-1: #ff6c7f;
  --low-2: #ff365d;
  --low-glow: rgba(255, 54, 93, 0.58);
  --hud-radius: 8px;
  --meter-width: 144px;
  --meter-height: 20px;
  --meter-inset-x: 9px;
  --meter-inset-y: 6px;
  --meter-channel-width: 126px;
  --meter-channel-height: 8px;
}

html,
body {
  height: 100%;
  margin: 0;
  overflow: hidden;
  background: transparent;
  color: var(--ink);
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  cursor: default;
  user-select: none;
  -webkit-user-select: none;
}

#root {
  position: relative;
  isolation: isolate;
  display: flex;
  box-sizing: border-box;
  width: 100%;
  height: 100vh;
  min-height: 100%;
  overflow: hidden;
  border: 1px solid rgba(205, 221, 242, 0.34);
  border-radius: var(--hud-radius);
  background: linear-gradient(180deg, rgba(29, 34, 47, 0.75), rgba(8, 10, 17, 0.86));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.15),
    inset 0 -1px 0 rgba(0, 0, 0, 0.52),
    inset 0 0 30px rgba(86, 71, 119, 0.08),
    0 10px 28px rgba(0, 0, 0, 0.24);
}

#root::before,
#root::after {
  position: absolute;
  z-index: 0;
  content: "";
  pointer-events: none;
}

#root::before {
  inset: 5px;
  background:
    linear-gradient(90deg, rgba(219, 231, 244, 0.58), rgba(219, 231, 244, 0.58)) left top / 20px 1px no-repeat,
    linear-gradient(180deg, rgba(219, 231, 244, 0.58), rgba(219, 231, 244, 0.58)) left top / 1px 9px no-repeat,
    linear-gradient(90deg, rgba(242, 201, 104, 0.52), rgba(242, 201, 104, 0.52)) right bottom / 20px 1px no-repeat,
    linear-gradient(180deg, rgba(242, 201, 104, 0.52), rgba(242, 201, 104, 0.52)) right bottom / 1px 9px no-repeat;
}

#root::after {
  top: 0;
  right: 24px;
  left: 24px;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(249, 246, 222, 0.72), transparent);
}

#card {
  position: relative;
  z-index: 1;
  display: flex;
  flex: 1;
  flex-direction: column;
  box-sizing: border-box;
  width: 100%;
  min-width: 0;
  padding: 14px 16px;
}

#card section {
  display: grid;
  grid-template-columns: 44px minmax(0, 1fr);
  min-width: 0;
  gap: 11px;
  padding: 6px 0 14px;
}

#card section + section {
  padding-top: 14px;
  padding-bottom: 6px;
  border-top: 1px solid var(--line);
}

.claude {
  --c1: var(--claude-1);
  --c2: var(--claude-2);
  --glow: var(--claude-glow);
  --portrait-bg: rgba(57, 221, 255, 0.1);
}

.codex {
  --c1: var(--codex-1);
  --c2: var(--codex-2);
  --glow: var(--codex-glow);
  --portrait-bg: rgba(215, 92, 255, 0.09);
}

.low {
  --c1: var(--low-1);
  --c2: var(--low-2);
  --glow: var(--low-glow);
}

.familiar-slot {
  position: relative;
  display: flex;
  width: 44px;
  min-height: 44px;
  align-items: center;
  justify-content: center;
}

.familiar-slot::before,
.familiar-slot::after {
  position: absolute;
  content: "";
  pointer-events: none;
}

.familiar-slot::before {
  width: 40px;
  height: 42px;
  border: 1px solid rgba(219, 231, 244, 0.24);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.05), var(--portrait-bg));
  clip-path: polygon(0 7px, 7px 0, 33px 0, 40px 7px, 40px 35px, 33px 42px, 7px 42px, 0 35px);
  box-shadow:
    inset 0 0 0 1px rgba(242, 201, 104, 0.08),
    0 0 12px var(--glow);
}

.familiar-slot::after {
  bottom: 0;
  left: 16px;
  width: 12px;
  height: 3px;
  background: linear-gradient(90deg, var(--silver), var(--gold));
  clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%);
  box-shadow: 0 0 7px var(--glow);
  opacity: 0.3;
}

.provider-card[data-working] .familiar-slot::after,
.provider-card[data-working] .activity-signal {
  animation: status-pulse 1.6s ease-in-out infinite;
}

.provider-content,
.rows,
.row {
  min-width: 0;
}

.provider-content {
  display: grid;
  gap: 8px;
}

.head {
  display: grid;
  grid-template-columns: max-content minmax(0, 1fr) 7px max-content;
  min-width: 0;
  align-items: center;
  gap: 7px;
  color: var(--ink-dim);
  font-size: 11px;
  line-height: 1;
  font-variant-numeric: tabular-nums;
}

.head strong {
  color: var(--ink);
  font-size: 13px;
  font-weight: 750;
  letter-spacing: 0;
  text-transform: uppercase;
}

.head .plan,
.head .age {
  white-space: nowrap;
}

.head .plan {
  min-width: 0;
}

.activity-signal {
  width: 7px;
  height: 7px;
  background: rgba(169, 175, 193, 0.22);
  clip-path: polygon(50% 0, 100% 50%, 50% 100%, 0 50%);
  box-shadow: inset 0 0 0 1px rgba(219, 231, 244, 0.16);
}

.provider-card[data-working] .activity-signal {
  background: linear-gradient(135deg, var(--silver), var(--c1) 55%, var(--c2));
  box-shadow: 0 0 8px var(--glow);
}

.rows {
  display: grid;
  gap: 7px;
}

.row {
  display: grid;
  grid-template-columns: 52px var(--meter-width) minmax(0, 1fr);
  align-items: center;
  gap: 8px;
  color: var(--ink-dim);
  font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
  font-size: 11px;
  line-height: 1.15;
  font-variant-numeric: tabular-nums;
}

.row .lbl,
.row .val {
  white-space: nowrap;
}

.row .val {
  min-width: 0;
  color: var(--ink-dim);
  text-align: right;
}

.row .val b {
  color: var(--ink);
  font-size: 12px;
  font-weight: 680;
}

.track {
  position: relative;
  isolation: isolate;
  width: var(--meter-width);
  min-width: var(--meter-width);
  height: var(--meter-height);
  overflow: visible;
  background: url("/hud/mana-bar-frame.png") center / 100% 100% no-repeat;
}

.fill {
  position: absolute;
  top: var(--meter-inset-y);
  left: var(--meter-inset-x);
  z-index: 1;
  max-width: var(--meter-channel-width);
  height: var(--meter-channel-height);
  overflow: hidden;
  border-radius: 2px;
  background: linear-gradient(90deg, var(--c1), var(--c2));
  box-shadow:
    0 0 4px var(--glow),
    0 0 10px var(--glow);
  transition: width 0.45s cubic-bezier(0.22, 0.8, 0.28, 1);
}

.fill::before,
.fill::after {
  position: absolute;
  content: "";
  pointer-events: none;
}

.fill::before {
  inset: 0;
  background: linear-gradient(102deg, transparent 0 34%, rgba(255, 255, 255, 0.68) 50%, transparent 66%);
  transform: translateX(-110%);
  animation: magic-glint 3.2s ease-in-out infinite;
}

.fill::after {
  top: 0;
  right: 0;
  left: 0;
  height: 1px;
  background: rgba(255, 255, 255, 0.66);
}

.fill[data-empty="true"] {
  opacity: 0;
  box-shadow: none;
}

.fill[data-empty="true"]::before,
.fill[data-empty="true"]::after {
  display: none;
}

.sprite {
  position: relative;
  z-index: 1;
  flex: none;
  width: 32px;
  height: 32px;
  background-repeat: no-repeat;
  background-size: 128px 96px;
  image-rendering: pixelated;
  animation: sprite-run 0.9s steps(4) infinite;
}

.sprite.clawd {
  background-image: url("/sprites/clawd.png");
  filter: drop-shadow(0 0 6px rgba(217, 119, 87, 0.48));
}

.sprite.nimbus {
  background-image: url("/sprites/nimbus.png");
  filter: drop-shadow(0 0 6px rgba(126, 240, 255, 0.38));
}

.sprite[data-state="idle"] {
  background-position-y: 0;
}

.sprite[data-state="working"] {
  background-position-y: -32px;
  animation-duration: 0.5s;
}

.sprite[data-state="hover"] {
  background-position-y: -64px;
  animation-duration: 0.6s;
}

.stale .fill:not([data-empty="true"]) {
  opacity: 0.48;
  filter: saturate(0.42);
}

.stale .familiar-slot::before,
.stale .familiar-slot::after,
.stale .activity-signal {
  opacity: 0.58;
  filter: saturate(0.35);
}

.empty {
  color: var(--ink-dim);
  font-size: 11px;
  line-height: 1.4;
}

@keyframes magic-glint {
  0%,
  42% {
    transform: translateX(-110%);
  }
  76%,
  100% {
    transform: translateX(110%);
  }
}

@keyframes status-pulse {
  0%,
  100% {
    opacity: 0.42;
  }
  50% {
    opacity: 1;
  }
}

@keyframes sprite-run {
  from {
    background-position-x: 0;
  }
  to {
    background-position-x: -128px;
  }
}

@media (prefers-reduced-motion: reduce) {
  .sprite,
  .fill::before,
  .provider-card[data-working] .familiar-slot::after,
  .provider-card[data-working] .activity-signal {
    animation: none;
  }

  .fill {
    transition: none;
  }
}
```

- [ ] **Step 4: Verify tests/build, then remove retired frames**

```bash
npx vitest run src/styles.test.ts src/meter.test.ts src/view.test.ts
npm test
npm run build
rm public/hud/boss-bar-left.png \
  public/hud/boss-bar-mid.png \
  public/hud/boss-bar-right.png
rg -n "boss-bar-|health_bar_icon|o_lobster|creativecommons.org/licenses/by/4.0" \
  . --glob '!node_modules/**' --glob '!target/**' --glob '!docs/superpowers/**' \
  --glob '!.git/**'
```

Expected: tests/build pass before deletion; `rg` returns no production or README references after deletion.

- [ ] **Step 5: Create the ignored visual-contract harness**

Create `.superpowers/visual-contract.html`:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="/src/styles.css" />
    <title>mana fantasy visual contract</title>
  </head>
  <body>
    <div id="root">
      <div id="card">
        <section class="provider-card claude" data-working>
          <div class="familiar-slot"><div class="sprite clawd" data-state="working"></div></div>
          <div class="provider-content">
            <div class="head"><strong>Claude</strong><span class="plan">Max</span><span class="activity-signal"></span><span class="age"></span></div>
            <div class="rows">
              <div class="row"><span class="lbl">5 hour</span><div class="track claude"><div class="fill" data-empty="false" style="width:126px"></div></div><span class="val"><b>100%</b><span> · Sun 12:51 PM</span></span></div>
              <div class="row"><span class="lbl">Weekly</span><div class="track claude low"><div class="fill" data-empty="false" style="width:37px"></div></div><span class="val"><b>29%</b><span> · Tue 1:59 PM</span></span></div>
              <div class="row"><span class="lbl">Fable</span><div class="track claude low"><div class="fill" data-empty="true" style="width:0px"></div></div><span class="val"><b>0%</b><span> · Tue 1:59 PM</span></span></div>
            </div>
          </div>
        </section>
        <section class="provider-card codex stale">
          <div class="familiar-slot"><div class="sprite nimbus" data-state="idle"></div></div>
          <div class="provider-content">
            <div class="head"><strong>Codex</strong><span class="plan">Pro</span><span class="activity-signal"></span><span class="age">2m ago</span></div>
            <div class="rows">
              <div class="row"><span class="lbl">Weekly</span><div class="track codex"><div class="fill" data-empty="false" style="width:57px"></div></div><span class="val"><b>45%</b><span> · Sun 12:51 PM</span></span></div>
            </div>
          </div>
        </section>
      </div>
    </div>
  </body>
</html>
```

- [ ] **Step 6: Run browser computed and screenshot QA**

Start:

```bash
npm run dev -- --host 127.0.0.1 --port 1431 --strictPort
```

Open the harness with the in-app browser at 440px width. Require:

```javascript
const tracks = [...document.querySelectorAll(".track")];
const values = [...document.querySelectorAll(".row .val")];
const fills = [...document.querySelectorAll(".fill")];
({
  rootWidth: document.querySelector("#root").getBoundingClientRect().width,
  tracks: tracks.map((element) => {
    const rect = element.getBoundingClientRect();
    return [rect.width, rect.height];
  }),
  valuesFit: values.every((element) => element.scrollWidth <= element.clientWidth),
  noHorizontalOverflow: document.documentElement.scrollWidth <= 440,
  frame: getComputedStyle(tracks[0]).backgroundImage,
  fills: fills.map((element) => {
    const fill = element.getBoundingClientRect();
    const track = element.parentElement.getBoundingClientRect();
    const style = getComputedStyle(element);
    return {
      left: fill.left - track.left,
      top: fill.top - track.top,
      height: fill.height,
      rightInside: fill.right <= track.left + 135,
      opacity: style.opacity,
      shadow: style.boxShadow,
    };
  }),
  codexLabels: [...document.querySelectorAll(".codex .lbl")].map((element) => element.textContent),
});
```

Expected: root 440; every track `[144, 20]`; values fit; no overflow; frame URL ends in `mana-bar-frame.png`; every fill starts at `[9, 6]`, is 8px high, and remains inside the core boundary; `codexLabels` is exactly `["Weekly"]`; the empty fill has opacity `0` and shadow `none`. Also compare label/track/value rectangles and require both declared 8px gaps without overlap. Capture normal and reduced-motion screenshots. Confirm fantasy details remain visible, the core stays inside the recess from 0%-100%, all non-empty glints use `magic-glint` with `3.2s` and identical timing, toggling `data-working` does not change track/fill rectangles or glint, stale text retains normal color/opacity, and reduced motion disables glint, sprites, signal pulse, and portrait pulse while keeping static non-empty energy visible.

- [ ] **Step 7: Clean scratch and commit visual integration**

Remove the harness with `apply_patch`, stop the exact Vite session with Ctrl-C, and confirm no repo-owned server remains. Then:

```bash
npm test
npm run build
git diff --check
git add src/styles.css src/styles.test.ts public/hud
git commit -m "feat: apply the whimsical fantasy mana HUD"
```

Expected: visual commit includes stylesheet/test and deletion of only retired boss frames. `.DS_Store` remains untouched.

---

### Task 5: Document, package, install, and verify v0.4.0

**Files:**
- Modify: `README.md`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: synchronized v0.4.0 metadata, accurate README, release bundle, and installed/native verification.
- Preserves: bundle identifier, 8px native radius, read-only credentials, and all runtime behavior.

- [ ] **Step 1: Rewrite README presentation copy**

Replace the opening with:

```markdown
Gamer mana bars for your AI subscriptions. This tiny always-on-top macOS widget
shows how much Claude Code and Codex usage you have left across session, weekly,
and model-scoped limits in a permanently expanded smoked-glass Party Roster.
Clawd and Nimbus sit beside their providers, while an original silver, gold,
and crystal fantasy frame holds equal-length live energy cores and complete
reset times. Codex limits are named from their actual duration, including
weekly-only Pro accounts.
```

Replace the Familiars paragraph with:

```markdown
Clawd (Claude's boxy mascot) and Nimbus (Codex's cloud-bot) occupy illuminated
fantasy portrait brackets beside their provider bands. They idle with the
occasional blink, squint `> <` while their CLI is actively running (local
process check every 5s - nothing leaves the machine), and celebrate whenever
you hover or drag the widget. Original pixel art is generated by
`scripts/gen-sprites.py`. Animations and the magical energy glint respect your
Reduced Motion setting.
```

Do not add third-party attribution for original generated art.

- [ ] **Step 2: Set project metadata to 0.4.0**

```bash
npm version 0.4.0 --no-git-tag-version
```

Set `version = "0.4.0"` in `src-tauri/Cargo.toml` and `"version": "0.4.0"` in `src-tauri/tauri.conf.json`, then:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Do not bulk-replace dependency versions.

- [ ] **Step 3: Verify metadata and full source gates**

```bash
node -e 'const p=require("./package.json"),l=require("./package-lock.json"); if(p.version!=="0.4.0"||l.version!=="0.4.0"||l.packages[""].version!=="0.4.0") process.exit(1)'
cargo metadata --manifest-path src-tauri/Cargo.toml --no-deps --format-version 1 \
  | jq -er '.packages[] | select(.name=="mana") | select(.version=="0.4.0") | .version'
node -e 'const c=require("./src-tauri/tauri.conf.json"); if(c.version!=="0.4.0"||c.app.windows.find(w=>w.label==="main").width!==440) process.exit(1)'
npm test
npm run build
rustfmt --edition 2021 --check src-tauri/src/lib.rs
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
git diff --check
```

Expected: all metadata, frontend, Rust, build, format, and diff gates pass. Do not add full-repo `cargo fmt --check` as a new gate for unrelated pre-existing formatting.

- [ ] **Step 4: Commit release metadata and documentation**

```bash
git add README.md package.json package-lock.json \
  src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json
git commit -m "chore: release mana v0.4.0"
```

- [ ] **Step 5: Build and inspect the release bundle**

```bash
npm run tauri build
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion' \
  src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' \
  src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
```

Expected: `0.4.0`, `0.4.0`, `com.vantasoft.mana`. Do not invent strict codesign as a local release gate.

- [ ] **Step 6: Request action-time install approval**

Report current installed version and process count:

```bash
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  /Applications/mana.app/Contents/Info.plist
pgrep -fl '/Applications/mana.app/Contents/MacOS/mana' || true
```

Ask explicit permission to quit installed mana, replace `/Applications/mana.app`, and launch v0.4.0. If withheld, report installed/native QA pending.

- [ ] **Step 7: Back up, replace, and launch after approval**

Read the `cua-driver` skill. Quit mana through its tray menu; do not use `kill`, `pkill`, or `open`. Then:

```bash
stamp="$(date +%Y%m%d-%H%M%S)"
app_backup="/tmp/mana.app.$stamp.backup"
state="$HOME/Library/Application Support/com.vantasoft.mana/.window-state.json"
state_backup="/tmp/mana-window-state.$stamp.json"
test ! -e /Applications/mana.app || mv /Applications/mana.app "$app_backup"
test ! -f "$state" || cp "$state" "$state_backup"
if ! ditto src-tauri/target/release/bundle/macos/mana.app /Applications/mana.app; then
  test ! -e "$app_backup" || mv "$app_backup" /Applications/mana.app
  exit 1
fi
cua-driver launch_app '{"bundle_id":"com.vantasoft.mana"}'
```

- [ ] **Step 8: Perform native Retina and migration QA**

Use `cua-driver` to snapshot the window. Require 440 logical width; measured content height; on-screen right edge; aligned 8px native/CSS corners; sharp silver/gold/crystal frame; full longest values; one Codex `Weekly` row; readable 0%, low, working, stale, absent, and reduced-motion states; drag/relaunch position persistence; tray Show/Hide and Quit; and no second widget.

```bash
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  /Applications/mana.app/Contents/Info.plist
test "$(pgrep -x mana | wc -l | tr -d ' ')" = "1"
```

Expected: installed v0.4.0 and exactly one process.

- [ ] **Step 9: Final clean-tree and evidence gate**

Remove ignored imagegen scratch after browser/native approval. Re-run:

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
git diff --check
git status --short
```

Expected: all gates pass; only `.DS_Store` remains untracked. Closeout records final imagegen prompt, built-in mode, asset path, 288x40 dimensions, SHA-256, browser screenshot, and native Retina result.
