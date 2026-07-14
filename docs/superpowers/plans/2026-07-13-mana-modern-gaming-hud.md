# mana Modern Gaming HUD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace mana's rejected retro treatment with a permanent 440px smoked-glass roster using one original image-generated sci-fi bezel, accurate 144x18 meters, readable values, and modern gaming state treatment.

**Architecture:** Keep Tauri 2 and the current vanilla TypeScript renderer. Imagegen produces one neutral transparent bezel while TypeScript remains authoritative for usage geometry and CSS remains authoritative for provider color, glow, motion, glass, and state styling. Preserve the existing polling and permanent-roster lifecycle, changing only the meter contract, window width, visuals, documentation, and release metadata.

**Tech Stack:** Tauri 2, Rust 2021, vanilla TypeScript 5.6, CSS, Vite 6, Vitest 3, native macOS `HudWindow` vibrancy, built-in imagegen, the installed chroma-key helper, macOS `sips`, Pillow/NumPy for read-only asset validation, and `cua-driver` for native GUI launch/inspection.

## Global Constraints

- The provider roster is permanently expanded; no compact mode or hover-driven geometry returns.
- Window width is exactly 440 logical pixels; runtime height remains measured from `#card.scrollHeight` and initial height remains 175 logical pixels.
- Every usage track is exactly 144x18 logical pixels with a 130x8 fill channel at x=7, y=5.
- `meterFillPixels(percent: number): number` remains the single clamped percentage-to-pixel interface used by initial rendering and live updates.
- The production bezel is original generated art at `public/hud/mana-bar-frame.png`; it contains no text, logo, fill, watermark, perspective, or third-party material.
- Use built-in imagegen and chroma-key removal first. Do not switch to a CLI model or require `OPENAI_API_KEY` without explicit user approval.
- Claude energy remains cyan-to-electric-blue, Codex remains magenta-to-hot-pink, and low mana remains coral-to-red.
- Every non-empty fill uses identical halo geometry and energy-sweep timing; provider or warning hue is the only meter-effect difference.
- Working state affects the familiar portrait and activity signal, not meter geometry or effect intensity.
- Stale styling must not dim labels and values into illegibility.
- Keep Clawd and Nimbus, their provider ownership, and idle/working/hover sprite states.
- Keep duration-aware provider parsing unchanged. Codex Pro weekly-only remains exactly one `Weekly` row; do not add frontend plan-based filtering.
- Preserve polling, credentials, activity detection, tray behavior, dragging, saved position, work-area containment, absent state, countdowns, and renderer escaping.
- Keep the native and CSS glass radius at 8 logical pixels.
- Add no runtime dependency. Leave the user's untracked `.DS_Store` untouched.
- Synchronize package, Cargo, and Tauri release metadata to `0.4.0` only after the visual implementation passes.
- Ask for explicit action-time approval before quitting or replacing `/Applications/mana.app`. Never use `open`, `kill`, or `pkill` for native app control.

---

## File Structure

- Create `public/hud/mana-bar-frame.png`: original 288x36 Retina bezel rendered by CSS at 144x18 logical pixels.
- Create `src/styles.test.ts`: source-level visual contract for the generated asset reference, retired asset removal, readable stale state, and reduced motion.
- Modify `src/meter.ts`: exact outer and channel geometry plus clamped fill conversion.
- Modify `src/meter.test.ts`: boundary and geometry coverage for the 130px channel.
- Modify `src/view.ts`: emit the new fill width and an explicit zero-energy state.
- Modify `src/view.test.ts`: verify the new fill width, zero-energy state, and exactly one weekly Codex row.
- Modify `src/main.ts`: keep zero-energy state synchronized during live updates.
- Modify `src/window-layout.ts`: widen the permanent roster to 440px.
- Modify `src/window-layout.test.ts`: prevent TypeScript/Tauri startup geometry drift and cover migration from an old right-edge position.
- Modify `src-tauri/tauri.conf.json`: widen initial native geometry and later set v0.4.0 metadata.
- Modify `src/styles.css`: replace retro styling with the complete modern glass, meter, portrait, data-state, and reduced-motion system.
- Delete `public/hud/boss-bar-left.png`, `public/hud/boss-bar-mid.png`, and `public/hud/boss-bar-right.png` after CSS no longer references them.
- Modify `README.md`: describe the permanent modern roster and original generated bezel accurately.
- Modify `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`: set v0.4.0 release metadata.
- Preserve `src-tauri/src/lib.rs`: its `HudWindow` vibrancy radius is already 8.0; verify it without creating a no-op diff.

---

### Task 1: Generate and validate the original mana-bar bezel

**Files:**
- Create: `public/hud/mana-bar-frame.png`
- Scratch only: `.superpowers/imagegen/`

**Interfaces:**
- Produces: a 288x36 RGBA PNG at `public/hud/mana-bar-frame.png` with a transparent 260x16 physical-pixel channel corresponding to the CSS 130x8 logical channel.
- Consumes: built-in `image_gen__imagegen`, the installed `remove_chroma_key.py`, `sips`, and the approved prompt from the design spec.
- Preserves: the three current boss-bar PNGs until Task 4 switches CSS to the generated asset.

- [ ] **Step 1: Establish clean scratch and record the generation boundary**

Run from the repository root:

```bash
mkdir -p .superpowers/imagegen
rm -f .superpowers/imagegen/mana-bar-frame-*.png
touch .superpowers/imagegen/imagegen-started-at
```

Expected: `.superpowers/imagegen/` exists, remains ignored by git, and `git status --short` still reports only `.DS_Store` plus any intentional plan/implementation changes.

- [ ] **Step 2: Generate one keyed bezel with the built-in imagegen tool**

Invoke `image_gen__imagegen` with no referenced images and this exact prompt:

```text
Use case: stylized-concept
Asset type: transparent UI game HUD element for a desktop usage widget
Primary request: Create one original premium sci-fi mana-bar bezel, an isolated horizontal hollow frame viewed perfectly straight-on.
Subject: Precision graphite-alloy frame with restrained dark-chrome highlights, compact angular end caps, fine machined seams, and sparse etched calibration ticks. The center must be completely open for a live energy fill rendered underneath.
Style/medium: polished modern AAA game interface asset, physically plausible hard-surface rendering, crisp and restrained rather than ornate
Composition/framing: one centered ultra-wide bezel, perfectly horizontal and symmetrical, orthographic front view. The visible outer silhouette itself must be approximately 8:1, occupy most of the canvas width, stay in a narrow central band, and remain legible when reduced to 144x18 logical pixels. The completely hollow channel should occupy roughly 90% of the outer width and 44% of the outer height so it remains clear at the required 130x8 logical-pixel bounds.
Lighting/mood: controlled cool rim light that defines the metal without adding a cast shadow
Color palette: neutral charcoal, graphite, gunmetal, and small silver highlights only
Scene/backdrop: perfectly flat solid #00ff00 chroma-key background for removal; the open center of the bezel must also show exactly the same flat #00ff00
Constraints: one frame only; continuous hollow center; crisp silhouette; no perspective; no background variation; no green anywhere in the frame
Avoid: energy fill, text, letters, numbers, logos, icons, gradients in the background, texture in the background, floor plane, cast shadow, contact shadow, reflection, particles, watermark, pixel art, fantasy ornament, oversized bolts
```

Expected: the tool returns exactly one new image and its local generated-image path. Do not call the CLI fallback.

- [ ] **Step 3: Stage and inspect the keyed source**

Copy the exact local path returned by imagegen into ignored scratch as:

```bash
cp "$GENERATED_IMAGE" .superpowers/imagegen/mana-bar-frame-keyed.png
```

`GENERATED_IMAGE` is the exact path returned by the immediately preceding tool call, not a guessed or globally newest file. Inspect `.superpowers/imagegen/mana-bar-frame-keyed.png` with `view_image` at original detail.

Expected: one straight-on hollow 8:1 frame, centered in a narrow horizontal band; outer background and complete inner channel are the same flat green; no forbidden content is present. If composition is wrong, perform one targeted built-in generation retry using the same prompt plus a one-sentence correction naming the observed defect.

- [ ] **Step 4: Remove the chroma key**

Run the installed helper exactly once:

```bash
python "${CODEX_HOME:-$HOME/.codex}/skills/.system/imagegen/scripts/remove_chroma_key.py" \
  --input "$PWD/.superpowers/imagegen/mana-bar-frame-keyed.png" \
  --out "$PWD/.superpowers/imagegen/mana-bar-frame-alpha.png" \
  --auto-key border \
  --soft-matte \
  --transparent-threshold 12 \
  --opaque-threshold 220 \
  --despill
```

Expected: `mana-bar-frame-alpha.png` is created with transparent background and center.

- [ ] **Step 5: Measure the alpha bounds without modifying pixels**

Run this read-only validation and capture its four-number output:

```bash
python - <<'PY'
from PIL import Image

path = ".superpowers/imagegen/mana-bar-frame-alpha.png"
with Image.open(path) as opened:
    image = opened.convert("RGBA")
mask = image.getchannel("A").point(lambda value: 255 if value > 16 else 0)
bbox = mask.getbbox()
if bbox is None:
    raise SystemExit("no visible bezel after chroma-key removal")
left, top, right, bottom = bbox
width, height = right - left, bottom - top
ratio = width / height
if not 7.2 <= ratio <= 8.8:
    raise SystemExit(f"bezel silhouette must be near 8:1; got {ratio:.2f}:1")
print(left, top, width, height)
PY
```

Expected: four integers and an accepted silhouette ratio from 7.2:1 through 8.8:1. A ratio failure is a composition failure and uses the single targeted generation retry from Step 3.

- [ ] **Step 6: Crop and normalize with macOS `sips`**

Read the same alpha bounds, then use `sips` for all production image transforms:

```bash
read left top crop_width crop_height <<< "$(python - <<'PY'
from PIL import Image
with Image.open('.superpowers/imagegen/mana-bar-frame-alpha.png') as opened:
    image = opened.convert('RGBA')
bbox = image.getchannel('A').point(lambda value: 255 if value > 16 else 0).getbbox()
if bbox is None:
    raise SystemExit('no visible bezel after chroma-key removal')
left, top, right, bottom = bbox
print(left, top, right - left, bottom - top)
PY
)"
sips --cropToHeightWidth "$crop_height" "$crop_width" \
  --cropOffset "$top" "$left" \
  .superpowers/imagegen/mana-bar-frame-alpha.png \
  --out .superpowers/imagegen/mana-bar-frame-cropped.png
sips --resampleHeightWidth 36 288 \
  .superpowers/imagegen/mana-bar-frame-cropped.png \
  --out public/hud/mana-bar-frame.png
```

Expected: `public/hud/mana-bar-frame.png` is exactly 288x36 physical pixels and retains alpha.

- [ ] **Step 7: Run deterministic alpha, channel, symmetry, and fringe QA**

Run this read-only pixel validation:

```bash
python - <<'PY'
from PIL import Image
import numpy as np

path = 'public/hud/mana-bar-frame.png'
with Image.open(path) as opened:
    assert opened.mode == 'RGBA', opened.mode
    assert opened.size == (288, 36), opened.size
    rgba = np.asarray(opened, dtype=np.int16)

alpha = rgba[:, :, 3]
for patch in (
    alpha[:3, :3], alpha[:3, -3:], alpha[-3:, :3], alpha[-3:, -3:]
):
    assert int(patch.max()) == 0, 'corner is not transparent'

assert int(alpha[10:26, 14:274].max()) == 0, \
    'the complete 130x8 logical live channel is not transparent'

visible = alpha > 16
coverage = float(visible.mean())
assert 0.08 <= coverage <= 0.65, f'implausible bezel coverage: {coverage:.3f}'

symmetry_error = float(np.mean(visible != visible[:, ::-1]))
assert symmetry_error <= 0.10, f'asymmetric silhouette: {symmetry_error:.3f}'

red, green, blue = rgba[:, :, 0], rgba[:, :, 1], rgba[:, :, 2]
green_fringe = (alpha > 0) & (green > red + 16) & (green > blue + 16)
assert int(green_fringe.sum()) == 0, \
    f'green fringe pixels: {int(green_fringe.sum())}'

print(
    f'RGBA 288x36; coverage={coverage:.3f}; '
    f'symmetry_error={symmetry_error:.3f}; channel/corners clear; no green fringe'
)
PY
file public/hud/mana-bar-frame.png
sips -g pixelWidth -g pixelHeight -g format -g hasAlpha \
  public/hud/mana-bar-frame.png
shasum -a 256 public/hud/mana-bar-frame.png
```

Expected: all assertions pass; `sips` reports 288x36 PNG with alpha; SHA-256 is recorded in the task report.

If fringe alone fails, rerun the chroma helper once to `mana-bar-frame-alpha-contract.png` with the same arguments plus `--edge-contract 1`, then repeat Steps 5-7 against that sibling. If the transparent channel still fails after the single targeted generation retry, stop and request approval before any true-transparency CLI fallback.

- [ ] **Step 8: Visually inspect and commit the generated asset**

Inspect `public/hud/mana-bar-frame.png` with `view_image` at original detail. Confirm: neutral graphite material, continuous hollow channel, no fill/text/logo/watermark/perspective, no green fringe, and retained detail at its 2x production size.

Run:

```bash
git diff --check
git status --short
git add public/hud/mana-bar-frame.png
git commit -m "feat: add original sci-fi mana bezel"
```

Expected: the commit contains only the new production PNG. Keep `.superpowers/imagegen/` ignored until final visual integration passes.

---

### Task 2: Implement accurate modern meter geometry and zero-energy state

**Files:**
- Modify: `src/meter.ts`
- Modify: `src/meter.test.ts`
- Modify: `src/view.ts`
- Modify: `src/view.test.ts`
- Modify: `src/main.ts`

**Interfaces:**
- Produces: `METER_WIDTH = 144`, `METER_HEIGHT = 18`, `METER_INSET_X = 7`, `METER_INSET_Y = 5`, `METER_INNER_WIDTH = 130`, `METER_CHANNEL_HEIGHT = 8`, and unchanged `meterFillPixels(percent): number`.
- Produces: `data-empty="true|false"` on `.fill` for both initial markup and live updates.
- Consumes: `manaLeft(usedPercent)` and existing provider snapshots.

- [ ] **Step 1: Write failing meter geometry tests**

Replace `src/meter.test.ts` with:

```typescript
import { describe, expect, it } from "vitest";
import {
  METER_CHANNEL_HEIGHT,
  METER_HEIGHT,
  METER_INNER_WIDTH,
  METER_INSET_X,
  METER_INSET_Y,
  METER_WIDTH,
  meterFillPixels,
} from "./meter";

describe("modern mana meter", () => {
  it("uses the approved frame and live-channel dimensions", () => {
    expect({
      width: METER_WIDTH,
      height: METER_HEIGHT,
      insetX: METER_INSET_X,
      insetY: METER_INSET_Y,
      innerWidth: METER_INNER_WIDTH,
      channelHeight: METER_CHANNEL_HEIGHT,
    }).toEqual({
      width: 144,
      height: 18,
      insetX: 7,
      insetY: 5,
      innerWidth: 130,
      channelHeight: 8,
    });
  });

  it.each([
    [-1, 0],
    [0, 0],
    [1, 1],
    [29, 38],
    [30, 39],
    [50, 65],
    [55, 72],
    [99, 129],
    [100, 130],
    [101, 130],
  ])("maps %s percent to %s channel pixels", (percent, pixels) => {
    expect(meterFillPixels(percent)).toBe(pixels);
  });
});
```

- [ ] **Step 2: Strengthen renderer regressions before production changes**

In the weekly-only test in `src/view.test.ts`, change the expected fill width to `59px`, assert exactly one row, and add a zero-energy case:

```typescript
expect(html).toContain('class="fill" data-empty="false" style="width:59px"');
expect(html.match(/class="row"/g)).toHaveLength(1);
```

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

The weekly fixture is 55% used, so it has 45% mana left and rounds to 59 of 130 pixels.

- [ ] **Step 3: Run focused tests and verify RED**

Run:

```bash
npx vitest run src/meter.test.ts src/view.test.ts
```

Expected: FAIL because the current meter is 128x16 with a 122px channel and the renderer has no `data-empty` attribute.

- [ ] **Step 4: Implement the meter constants**

Replace `src/meter.ts` with:

```typescript
export const METER_WIDTH = 144;
export const METER_HEIGHT = 18;
export const METER_INSET_X = 7;
export const METER_INSET_Y = 5;
export const METER_INNER_WIDTH = METER_WIDTH - METER_INSET_X * 2;
export const METER_CHANNEL_HEIGHT = METER_HEIGHT - METER_INSET_Y * 2;

export function meterFillPixels(percent: number): number {
  const clamped = Math.max(0, Math.min(100, percent));
  return Math.round((clamped / 100) * METER_INNER_WIDTH);
}
```

- [ ] **Step 5: Emit and synchronize the zero-energy state**

Change `barHtml` in `src/view.ts` to emit the explicit state:

```typescript
function barHtml(snapshot: Snapshot, bar: Bar, index: number): string {
  const left = manaLeft(bar.used_percent);
  const pixels = meterFillPixels(left);
  return `<div class="track ${snapshot.provider}${left < 30 ? " low" : ""}" data-bar="${index}"><div class="fill" data-empty="${left <= 0}" style="width:${pixels}px"></div></div>`;
}
```

In `applyData` in `src/main.ts`, replace the one-line fill update with:

```typescript
const fill = track.querySelector<HTMLElement>(".fill");
if (fill) {
  fill.style.width = `${meterFillPixels(left)}px`;
  fill.dataset.empty = String(left <= 0);
}
```

Do not filter or relabel provider rows in the frontend.

- [ ] **Step 6: Run focused and full frontend verification**

Run:

```bash
npx vitest run src/meter.test.ts src/view.test.ts
npm test
npm run build
git diff --check
```

Expected: focused tests pass, all 33 frontend tests pass after the new zero-energy case, TypeScript/Vite build succeeds, and diff check is clean.

- [ ] **Step 7: Commit the meter contract**

```bash
git add src/meter.ts src/meter.test.ts src/view.ts src/view.test.ts src/main.ts
git commit -m "feat: define modern mana meter geometry"
```

Expected: one focused commit containing no CSS or native geometry changes.

---

### Task 3: Widen the permanent native roster to 440px

**Files:**
- Modify: `src/window-layout.ts`
- Modify: `src/window-layout.test.ts`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: `ROSTER_WIDTH = 440` while retaining `INITIAL_ROSTER_HEIGHT = 175`.
- Consumes: existing `rosterOrigin`, `rosterHeight`, `createSerialQueue`, and `src/main.ts` use of `ROSTER_WIDTH`.
- Verifies: Tauri startup dimensions cannot drift from the TypeScript runtime constants.

- [ ] **Step 1: Write failing cross-config geometry tests**

Add this import to `src/window-layout.test.ts`:

```typescript
import tauriConfig from "../src-tauri/tauri.conf.json";
```

Replace the startup case and add the migration case:

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

Update the existing in-bounds case to use `{ width: ROSTER_WIDTH, height: INITIAL_ROSTER_HEIGHT }`. Update the existing both-axes clamp case to use `{ width: ROSTER_WIDTH, height: 210 }` and expect `{ x: 2000, y: 1380 }`.

- [ ] **Step 2: Run the layout tests and verify RED**

```bash
npx vitest run src/window-layout.test.ts
```

Expected: FAIL because both TypeScript and Tauri still declare width 420.

- [ ] **Step 3: Change the two authoritative width declarations**

In `src/window-layout.ts`:

```typescript
export const ROSTER_WIDTH = 440;
export const INITIAL_ROSTER_HEIGHT = 175;
```

In the main window entry in `src-tauri/tauri.conf.json`:

```json
"width": 440,
"height": 175
```

Do not modify `src/main.ts`; it already consumes `ROSTER_WIDTH` for both resizing and work-area clamping. Do not modify `src-tauri/src/lib.rs`; the native radius is already 8.0 and window state already restores position only.

- [ ] **Step 4: Run focused and build verification**

```bash
npx vitest run src/window-layout.test.ts
npm run build
git diff --check
```

Expected: all six layout/queue tests pass, build succeeds, and diff check is clean.

- [ ] **Step 5: Commit the wider geometry**

```bash
git add src/window-layout.ts src/window-layout.test.ts src-tauri/tauri.conf.json
git commit -m "feat: widen the permanent mana roster"
```

Expected: one geometry-only commit with Tauri version still at 0.3.0.

---

### Task 4: Integrate the modern glass HUD and generated bezel

**Files:**
- Create: `src/styles.test.ts`
- Modify: `src/styles.css`
- Delete: `public/hud/boss-bar-left.png`
- Delete: `public/hud/boss-bar-mid.png`
- Delete: `public/hud/boss-bar-right.png`
- Scratch only: `.superpowers/visual-contract.html`

**Interfaces:**
- Consumes: `public/hud/mana-bar-frame.png`, the fixed meter constants, existing provider classes, `.familiar-slot`, `.activity-signal`, `.head`, `.rows`, `.row`, `.fill[data-empty]`, `.stale`, and `[data-working]`.
- Produces: fixed 144x18 CSS tracks, a shared energy effect, modern glass shell, portrait sockets, readable state treatment, and complete reduced-motion shutdown.
- Preserves: existing DOM structure and sprite sheet animation rows.

- [ ] **Step 1: Write a failing stylesheet contract**

Create `src/styles.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import styles from "./styles.css?raw";

describe("modern gaming HUD stylesheet", () => {
  it("uses only the original generated bezel", () => {
    expect(styles).toContain('url("/hud/mana-bar-frame.png")');
    expect(styles).not.toContain("boss-bar-");
    expect(styles).not.toContain("background-size: 4px 4px");
  });

  it("declares the fixed logical meter box", () => {
    expect(styles).toContain("--meter-width: 144px");
    expect(styles).toContain("--meter-height: 18px");
    expect(styles).toContain("--meter-channel-width: 130px");
    expect(styles).toContain("--meter-channel-height: 8px");
  });

  it("keeps stale text readable and covers reduced motion", () => {
    expect(styles).not.toMatch(/\.stale\s*\{[^}]*\bfilter\s*:/s);
    expect(styles).toContain(".stale .fill");
    expect(styles).toContain("@media (prefers-reduced-motion: reduce)");
    expect(styles).toContain(".provider-card[data-working] .familiar-slot::after");
  });
});
```

- [ ] **Step 2: Run the stylesheet test and verify RED**

```bash
npx vitest run src/styles.test.ts
```

Expected: FAIL because CSS still references the three boss-bar pieces and the retro grid.

- [ ] **Step 3: Replace `src/styles.css` with the modern system**

Replace the complete stylesheet with:

```css
:root {
  --ink: #f3f7ff;
  --ink-dim: #9ca9bd;
  --line: rgba(161, 185, 218, 0.28);
  --claude-1: #38ddff;
  --claude-2: #527cff;
  --claude-glow: rgba(56, 221, 255, 0.48);
  --codex-1: #d75cff;
  --codex-2: #ff5ba8;
  --codex-glow: rgba(215, 92, 255, 0.46);
  --low-1: #ff6b7d;
  --low-2: #ff365d;
  --low-glow: rgba(255, 54, 93, 0.58);
  --hud-radius: 8px;
  --meter-width: 144px;
  --meter-height: 18px;
  --meter-inset-x: 7px;
  --meter-inset-y: 5px;
  --meter-channel-width: 130px;
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
  border: 1px solid rgba(142, 181, 233, 0.34);
  border-radius: var(--hud-radius);
  background: linear-gradient(180deg, rgba(27, 35, 49, 0.72), rgba(7, 11, 18, 0.84));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.14),
    inset 0 -1px 0 rgba(0, 0, 0, 0.5),
    inset 0 0 28px rgba(35, 67, 111, 0.1),
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
    linear-gradient(90deg, rgba(114, 205, 255, 0.52), rgba(114, 205, 255, 0.52)) left top / 20px 1px no-repeat,
    linear-gradient(180deg, rgba(114, 205, 255, 0.52), rgba(114, 205, 255, 0.52)) left top / 1px 9px no-repeat,
    linear-gradient(90deg, rgba(240, 111, 255, 0.42), rgba(240, 111, 255, 0.42)) right bottom / 20px 1px no-repeat,
    linear-gradient(180deg, rgba(240, 111, 255, 0.42), rgba(240, 111, 255, 0.42)) right bottom / 1px 9px no-repeat;
  opacity: 0.72;
}

#root::after {
  top: 0;
  right: 24px;
  left: 24px;
  height: 1px;
  background: linear-gradient(90deg, transparent, rgba(231, 244, 255, 0.68), transparent);
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
  --portrait-bg: rgba(56, 221, 255, 0.11);
}

.codex {
  --c1: var(--codex-1);
  --c2: var(--codex-2);
  --glow: var(--codex-glow);
  --portrait-bg: rgba(215, 92, 255, 0.1);
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
  border: 1px solid rgba(190, 211, 238, 0.2);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.045), var(--portrait-bg));
  clip-path: polygon(0 7px, 7px 0, 33px 0, 40px 7px, 40px 35px, 33px 42px, 7px 42px, 0 35px);
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.08),
    0 0 12px var(--glow);
}

.familiar-slot::after {
  bottom: 0;
  left: 10px;
  width: 24px;
  height: 1px;
  background: var(--c1);
  box-shadow: 0 0 7px var(--glow);
  opacity: 0.28;
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
  grid-template-columns: max-content minmax(0, 1fr) 6px max-content;
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
  width: 6px;
  height: 6px;
  border: 1px solid rgba(207, 224, 244, 0.18);
  border-radius: 1px;
  background: rgba(156, 169, 189, 0.2);
  box-sizing: border-box;
  transform: rotate(45deg);
}

.provider-card[data-working] .activity-signal {
  border-color: rgba(255, 255, 255, 0.45);
  background: var(--c1);
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
  width: var(--meter-width);
  min-width: var(--meter-width);
  height: var(--meter-height);
  overflow: visible;
}

.track::before,
.track::after {
  position: absolute;
  content: "";
  pointer-events: none;
}

.track::before {
  top: var(--meter-inset-y);
  left: var(--meter-inset-x);
  z-index: 0;
  width: var(--meter-channel-width);
  height: var(--meter-channel-height);
  border-radius: 1px;
  background: rgba(2, 5, 10, 0.76);
  box-shadow:
    inset 0 1px 2px rgba(0, 0, 0, 0.84),
    inset 0 0 0 1px rgba(177, 205, 239, 0.05);
}

.track::after {
  inset: 0;
  z-index: 3;
  background: url("/hud/mana-bar-frame.png") center / 100% 100% no-repeat;
}

.fill {
  position: absolute;
  top: var(--meter-inset-y);
  left: var(--meter-inset-x);
  z-index: 1;
  height: var(--meter-channel-height);
  overflow: hidden;
  border-radius: 1px;
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
  background: linear-gradient(100deg, transparent 0 34%, rgba(255, 255, 255, 0.65) 50%, transparent 66%);
  transform: translateX(-110%);
  animation: energy-sweep 2.4s ease-in-out infinite;
}

.fill::after {
  top: 0;
  right: 0;
  left: 0;
  height: 1px;
  background: rgba(255, 255, 255, 0.64);
}

.fill[data-empty="true"] {
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

.stale .fill {
  opacity: 0.48;
  filter: saturate(0.42);
}

.stale .sprite,
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

@keyframes energy-sweep {
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

- [ ] **Step 4: Run the stylesheet contract and frontend build**

```bash
npx vitest run src/styles.test.ts src/meter.test.ts src/view.test.ts
npm test
npm run build
```

Expected: all stylesheet, meter, renderer, and full frontend tests pass; Vite emits the generated bezel and both sprite sheets without missing-asset warnings.

- [ ] **Step 5: Remove the retired third-party frames**

Only after Step 4 is green:

```bash
rm public/hud/boss-bar-left.png \
  public/hud/boss-bar-mid.png \
  public/hud/boss-bar-right.png
rg -n "boss-bar-|health_bar_icon|o_lobster|creativecommons.org/licenses/by/4.0" \
  . --glob '!node_modules/**' --glob '!target/**' --glob '!docs/superpowers/**' \
  --glob '!.git/**'
```

Expected: `rg` returns no production or README attribution references. Historical design/plan documents remain untouched.

- [ ] **Step 6: Build a temporary visual-contract harness**

Create ignored `.superpowers/visual-contract.html` with this exact markup:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="/src/styles.css" />
    <title>mana visual contract</title>
  </head>
  <body>
    <div id="root">
      <div id="card">
        <section class="provider-card claude" data-working>
          <div class="familiar-slot"><div class="sprite clawd" data-state="working"></div></div>
          <div class="provider-content">
            <div class="head"><strong>Claude</strong><span class="plan">Max</span><span class="activity-signal"></span><span class="age"></span></div>
            <div class="rows">
              <div class="row"><span class="lbl">5 hour</span><div class="track claude"><div class="fill" data-empty="false" style="width:130px"></div></div><span class="val"><b>100%</b><span> · Sun 12:51 PM</span></span></div>
              <div class="row"><span class="lbl">Weekly</span><div class="track claude low"><div class="fill" data-empty="false" style="width:38px"></div></div><span class="val"><b>29%</b><span> · Tue 1:59 PM</span></span></div>
              <div class="row"><span class="lbl">Fable</span><div class="track claude low"><div class="fill" data-empty="true" style="width:0px"></div></div><span class="val"><b>0%</b><span> · Tue 1:59 PM</span></span></div>
            </div>
          </div>
        </section>
        <section class="provider-card codex stale">
          <div class="familiar-slot"><div class="sprite nimbus" data-state="idle"></div></div>
          <div class="provider-content">
            <div class="head"><strong>Codex</strong><span class="plan">Pro</span><span class="activity-signal"></span><span class="age">2m ago</span></div>
            <div class="rows">
              <div class="row"><span class="lbl">Weekly</span><div class="track codex"><div class="fill" data-empty="false" style="width:59px"></div></div><span class="val"><b>45%</b><span> · Sun 12:51 PM</span></span></div>
            </div>
          </div>
        </section>
      </div>
    </div>
  </body>
</html>
```

- [ ] **Step 7: Perform computed browser and screenshot QA**

Start a repo-owned Vite server in a persistent terminal session:

```bash
npm run dev -- --host 127.0.0.1 --port 1431 --strictPort
```

Use the in-app-browser control skill to open `http://127.0.0.1:1431/.superpowers/visual-contract.html` at a 440px-wide viewport. Measure with browser JavaScript and require:

```javascript
const tracks = [...document.querySelectorAll(".track")];
const values = [...document.querySelectorAll(".row .val")];
({
  rootWidth: document.querySelector("#root").getBoundingClientRect().width,
  tracks: tracks.map((element) => {
    const rect = element.getBoundingClientRect();
    return [rect.width, rect.height];
  }),
  valuesFit: values.every((element) => element.scrollWidth <= element.clientWidth),
  noHorizontalOverflow: document.documentElement.scrollWidth <= 440,
  bezel: getComputedStyle(tracks[0], "::after").backgroundImage,
  staleTextColor: getComputedStyle(document.querySelector(".stale .row .val")).color,
});
```

Expected: `rootWidth` 440; every track `[144, 18]`; `valuesFit` and `noHorizontalOverflow` true; bezel URL ends in `mana-bar-frame.png`; stale value color remains the normal dim text color. Capture normal and reduced-motion screenshots, inspect the 0%, 29%, 45%, and 100% rows, and verify the 0% bar has no glow.

Also confirm computed `animation-name` for every non-empty `.fill::before` is `energy-sweep`, regardless of provider/low state, and that reduced-motion emulation returns `none` for the sweep, sprite, signal, and portrait pulse.

- [ ] **Step 8: Clean scratch, stop the server, and commit visual integration**

Remove `.superpowers/visual-contract.html` with `apply_patch`, stop the exact Vite terminal session with Ctrl-C, and confirm no repo-owned Vite process remains. Then run:

```bash
npm test
npm run build
git diff --check
git status --short
git add src/styles.css src/styles.test.ts public/hud
git commit -m "feat: apply the modern gaming mana HUD"
```

Expected: commit includes the stylesheet/test, generated bezel already tracked from Task 1, and deletion of only the three retired boss-bar PNGs. `.DS_Store` remains untouched.

---

### Task 5: Document, package, install, and verify mana v0.4.0

**Files:**
- Modify: `README.md`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: synchronized v0.4.0 metadata and an accurate permanent-roster README.
- Consumes: all passing frontend/native tests and the completed modern HUD.
- Preserves: native radius 8.0, bundle identifier `com.vantasoft.mana`, app target, and read-only credential behavior.

- [ ] **Step 1: Rewrite stale README presentation copy**

Replace the README opening paragraph with:

```markdown
Gamer mana bars for your AI subscriptions. This tiny always-on-top macOS widget
shows how much Claude Code and Codex usage you have left across session, weekly,
and model-scoped limits in a permanently expanded smoked-glass Party Roster.
Clawd and Nimbus sit beside their providers, while an original sci-fi bezel
frames equal-length live energy meters and complete reset times. Codex limits
are named from their actual duration, including weekly-only Pro accounts.
```

Replace the Familiars paragraph with:

```markdown
Clawd (Claude's boxy mascot) and Nimbus (Codex's cloud-bot) occupy illuminated
portrait slots beside their provider bands. They idle with the occasional
blink, squint `> <` while their CLI is actively running (local process check
every 5s - nothing leaves the machine), and celebrate whenever you hover or
drag the widget. Original pixel art is generated by `scripts/gen-sprites.py`.
Animations and the energy sweep respect your Reduced Motion setting.
```

Do not add third-party attribution; the shipped bezel is original generated art and the asset-pack files have been removed.

- [ ] **Step 2: Set all project-owned release metadata to 0.4.0**

Run:

```bash
npm version 0.4.0 --no-git-tag-version
```

Change the project version in `src-tauri/Cargo.toml` to:

```toml
version = "0.4.0"
```

Change the project version in `src-tauri/tauri.conf.json` to:

```json
"version": "0.4.0"
```

Refresh only the local Cargo package lock record through normal Cargo resolution:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Do not bulk-replace unrelated dependency versions containing `0.3.0` or `0.4.0`.

- [ ] **Step 3: Verify metadata consistency**

```bash
node -e 'const p=require("./package.json"),l=require("./package-lock.json"); if(p.version!=="0.4.0"||l.version!=="0.4.0"||l.packages[""].version!=="0.4.0") process.exit(1)'
cargo metadata --manifest-path src-tauri/Cargo.toml --no-deps --format-version 1 \
  | jq -er '.packages[] | select(.name=="mana") | select(.version=="0.4.0") | .version'
node -e 'const c=require("./src-tauri/tauri.conf.json"); if(c.version!=="0.4.0"||c.app.windows.find(w=>w.label==="main").width!==440) process.exit(1)'
```

Expected: Cargo prints `0.4.0`; both Node checks exit 0.

- [ ] **Step 4: Run the complete source verification gate**

```bash
npm test
npm run build
rustfmt --edition 2021 --check src-tauri/src/lib.rs
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
git diff --check
```

Expected: 36 frontend tests after `styles.test.ts`, 18 Rust tests, TypeScript/Vite build, Cargo check, changed-file Rust formatting, and diff check all pass. Do not use full-repo `cargo fmt --check` as a new gate because unrelated pre-existing Rust formatting drift is outside this redesign.

- [ ] **Step 5: Commit documentation and release metadata**

```bash
git add README.md package.json package-lock.json \
  src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json
git commit -m "chore: release mana v0.4.0"
```

Expected: one release commit with no `.DS_Store` and no generated scratch.

- [ ] **Step 6: Build and inspect the release bundle**

```bash
npm run tauri build
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion' \
  src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' \
  src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
```

Expected: `0.4.0`, `0.4.0`, and `com.vantasoft.mana`. Do not add strict codesign verification: local ad-hoc bundles are not sealed as a distribution artifact.

- [ ] **Step 7: Request action-time permission for native replacement**

Before quitting or replacing anything, report the current installed version and running process count:

```bash
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  /Applications/mana.app/Contents/Info.plist
pgrep -fl '/Applications/mana.app/Contents/MacOS/mana' || true
```

Ask the user explicitly for permission to quit the installed mana app, replace `/Applications/mana.app`, and launch the v0.4.0 build. If permission is not granted, stop here and report installed/native QA as pending.

- [ ] **Step 8: Back up, replace, and launch through `cua-driver` after approval**

After approval, read and follow the `cua-driver` skill. Quit mana through its tray menu using `cua-driver`; do not use `kill`, `pkill`, or Activity Monitor. Create reversible backups and install:

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

Expected: the old bundle and window state have timestamped backups, v0.4.0 is installed, and launch returns one native mana process/window without foreground activation.

- [ ] **Step 9: Perform native Retina and migration QA**

Use `cua-driver` to snapshot the mana window and save a native screenshot under `/tmp`. Require:

- window width is 440 logical points and height matches rendered content;
- right edge is inside the active monitor after loading a position saved by a narrower build;
- native and CSS 8px corners align with no opaque fringe;
- the generated bezel is sharp at Retina scale and aligned around every live channel;
- `100% · Sun 12:51 PM` and normal reset values fit without clipping;
- Codex Pro shows one `Weekly` row and no incorrect `5 hour` row;
- 0%, low, working, stale, absent, and reduced-motion presentations remain readable;
- dragging and relaunch preserve or clamp position;
- tray Show/Hide and Quit remain functional;
- no second mana widget is started during verification.

Verify installed metadata and one process:

```bash
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  /Applications/mana.app/Contents/Info.plist
test "$(pgrep -x mana | wc -l | tr -d ' ')" = "1"
```

Expected: installed version `0.4.0` and exactly one process.

- [ ] **Step 10: Final clean-tree and evidence check**

Remove `.superpowers/imagegen/` scratch after the production PNG and native screenshot have passed. Re-run:

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
git diff --check
git status --short
```

Expected: all gates pass, the working tree contains only the user's untracked `.DS_Store`, the installed app is v0.4.0, and the closeout records the final imagegen prompt, built-in mode, production asset path, 288x36 physical dimensions, SHA-256, browser screenshot result, and native Retina result.
