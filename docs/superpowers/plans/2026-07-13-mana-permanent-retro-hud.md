# mana Permanent Retro HUD Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace mana's hover-driven compact/expanded widget with one permanent 420px Party Roster whose usage bars share an exact 128x16 pixel frame and universal glow behavior.

**Architecture:** Keep Tauri 2 and the existing vanilla TypeScript renderer. Simplify `src/main.ts` into a single roster lifecycle, keep native window math in `src/window-layout.ts`, and introduce a small pure `src/meter.ts` module for logical fill geometry. Compose the licensed Boss HUD PNG pieces as a CSS overlay while CSS remains responsible for provider color, fill, glow, shimmer, glass, and state styling.

**Tech Stack:** Tauri 2, Rust 2021, vanilla TypeScript 5.6, CSS, Vite 6, Vitest 3, native macOS `HudWindow` vibrancy, CC BY 4.0 PNG assets.

## Global Constraints

- The only visible frontend is the Party Roster; no compact state or compact flash remains.
- Window width is exactly 420 logical pixels; initial configured height is 175 logical pixels and runtime height is measured from `#card.scrollHeight`.
- Every usage track is exactly 128x16 logical pixels and uses one left, two middle, and one right 32x16 Boss HUD frame pieces.
- Every non-empty fill uses identical halo geometry, highlight, shimmer duration, and stepped easing; only provider or low-state hue changes.
- Claude remains cyan-to-indigo, Codex violet-to-magenta, and low mana coral-red.
- Mouse hover may change familiar animation state but must never change native window size.
- Preserve polling, credentials, activity state, tray behavior, saved position, dragging, work-area containment, stale/absent/low states, and reduced motion.
- Use only the three Boss HUD frame PNGs from the supplied asset pack; add CC BY 4.0 credit to `o_lobster` without implying endorsement.
- Keep all dynamic labels escaped and keep weekly-only Codex as a single `Weekly` row.
- Synchronize package, Cargo, and Tauri versions to `0.4.0`.
- Add no runtime dependency and leave the user's untracked `.DS_Store` untouched.

---

## File Structure

- Create `src/meter.ts`: fixed frame and inner-fill dimensions plus clamped percentage-to-pixel conversion.
- Create `src/meter.test.ts`: boundary coverage for fixed meter fill geometry.
- Create `src/vite-env.d.ts`: Vite raw-asset module types used by the permanent-markup test.
- Create `public/hud/boss-bar-left.png`: unchanged CC BY 4.0 left frame piece.
- Create `public/hud/boss-bar-mid.png`: unchanged CC BY 4.0 middle frame piece.
- Create `public/hud/boss-bar-right.png`: unchanged CC BY 4.0 right frame piece.
- Create `THIRD_PARTY_NOTICES.md`: source, creator, license, and composition disclosure.
- Modify `src/window-layout.ts`: permanent roster constants, two-axis work-area clamp, measured height, and serial queue only.
- Modify `src/window-layout.test.ts`: replace collapse behavior with permanent-window geometry coverage.
- Modify `src/main.ts`: render one card, update one set of bars, keep hover only for sprites, and always measure/clamp the native roster.
- Modify `src/view.ts`: remove compact renderer and emit fixed logical pixel fill widths.
- Modify `src/view.test.ts`: remove compact assertions and verify pixel fill markup.
- Modify `src/format.ts` and `src/format.test.ts`: remove compact-only countdown formatting.
- Modify `index.html`: remove `#pill` and retain only the draggable provider roster.
- Modify `src/styles.css`: fixed pixel frames, equal grid columns, shared glow/shimmer, square activity signal, 8px retro glass, and reduced motion.
- Modify `src-tauri/src/lib.rs`: match native vibrancy corner radius to 8px.
- Modify `src-tauri/tauri.conf.json`: permanent initial geometry and v0.4.0.
- Modify `README.md`: permanent-roster behavior and third-party attribution link.
- Modify `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, and `src-tauri/Cargo.lock`: v0.4.0 metadata.

---

### Task 1: Make the Party Roster permanent

**Files:**
- Modify: `src/window-layout.ts`
- Modify: `src/window-layout.test.ts`
- Modify: `src/main.ts`
- Modify: `src/view.ts`
- Modify: `src/view.test.ts`
- Create: `src/vite-env.d.ts`
- Modify: `src/format.ts`
- Modify: `src/format.test.ts`
- Modify: `index.html`
- Modify: `src/styles.css`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Produces: `ROSTER_WIDTH`, `INITIAL_ROSTER_HEIGHT`, `rosterOrigin(origin, size, workArea, scaleFactor)`, `rosterHeight(cardScrollHeight)`, and `createSerialQueue(onError)` from `src/window-layout.ts`.
- Consumes: `cardHtml(snapshot, provider)` and the existing Tauri `currentMonitor`, `outerPosition`, `setSize`, and `setPosition` APIs.
- Removes: `pillHtml`, `fmtCompactCountdown`, collapsed geometry, layout intent, delayed collapse, and collapsed-origin restoration.

- [ ] **Step 1: Write failing permanent-window geometry tests**

Replace the collapsed/hover-specific imports and cases in `src/window-layout.test.ts` with these permanent APIs while retaining the serial-queue rejection test:

```typescript
import { describe, expect, it } from "vitest";
import {
  INITIAL_ROSTER_HEIGHT,
  ROSTER_WIDTH,
  createSerialQueue,
  rosterHeight,
  rosterOrigin,
} from "./window-layout";

describe("permanent roster geometry", () => {
  it("uses the expanded roster from startup", () => {
    expect({ width: ROSTER_WIDTH, height: INITIAL_ROSTER_HEIGHT }).toEqual({
      width: 420,
      height: 175,
    });
  });

  it("keeps an origin that fits the active work area", () => {
    expect(
      rosterOrigin(
        { x: 500, y: 80 },
        { width: 420, height: 175 },
        { x: 0, y: 0, width: 2880, height: 1800 },
        2,
      ),
    ).toEqual({ x: 500, y: 80 });
  });

  it("clamps both axes for a saved compact position near the edge", () => {
    expect(
      rosterOrigin(
        { x: 2300, y: 1700 },
        { width: 420, height: 210 },
        { x: 0, y: 0, width: 2880, height: 1800 },
        2,
      ),
    ).toEqual({ x: 2040, y: 1380 });
  });
});

it("rounds measured card height plus the root border", () => {
  expect(rosterHeight(207.2)).toBe(210);
});
```

- [ ] **Step 2: Run the layout tests and verify RED**

Run:

```bash
npx vitest run src/window-layout.test.ts
```

Expected: FAIL because `ROSTER_WIDTH`, `INITIAL_ROSTER_HEIGHT`, `rosterOrigin`, and `rosterHeight` do not exist.

- [ ] **Step 3: Implement permanent geometry helpers**

Replace compact/expansion state in `src/window-layout.ts` with:

```typescript
export const ROSTER_WIDTH = 420;
export const INITIAL_ROSTER_HEIGHT = 175;

type Point = { x: number; y: number };
type Size = { width: number; height: number };
type Rect = Point & Size;

export function rosterOrigin(
  origin: Point,
  size: Size,
  workArea: Rect,
  scaleFactor: number,
): Point {
  const maximumX = workArea.x + workArea.width - size.width * scaleFactor;
  const maximumY = workArea.y + workArea.height - size.height * scaleFactor;
  return {
    x: Math.max(workArea.x, Math.min(origin.x, maximumX)),
    y: Math.max(workArea.y, Math.min(origin.y, maximumY)),
  };
}

export function rosterHeight(cardScrollHeight: number): number {
  return Math.ceil(cardScrollHeight + 2);
}

export function createSerialQueue(onError: (error: unknown) => void) {
  let tail: Promise<void> = Promise.resolve();
  return (task: () => Promise<void>): Promise<void> => {
    tail = tail.then(task).catch((error) => onError(error));
    return tail;
  };
}
```

Keep the existing serial-queue recovery test unchanged. Delete `COLLAPSED_WIDTH`, `COLLAPSED_HEIGHT`, `COLLAPSE_DELAY_MS`, `expandedOrigin`, `expandedHeight`, `collapsedOriginFromExpanded`, `createLayoutIntent`, and `createHoverIntent` plus their tests.

- [ ] **Step 4: Run the focused layout tests and verify GREEN**

Run:

```bash
npx vitest run src/window-layout.test.ts
```

Expected: all permanent geometry and queue tests PASS.

- [ ] **Step 5: Write the failing permanent-markup expectation**

Create `src/vite-env.d.ts` so the existing Vite dependency supplies raw-import types:

```typescript
/// <reference types="vite/client" />
```

Update `src/view.test.ts` to import only `cardHtml`, then import the document as raw markup:

```typescript
import indexMarkup from "../index.html?raw";
```

Add this permanent-roster test:

```typescript
it("mounts the roster without a compact pill", () => {
  expect(indexMarkup).toContain('id="card"');
  expect(indexMarkup).not.toContain('id="pill"');
});
```

Also add this assertion to the weekly-only card test:

```typescript
expect(html).toContain('class="track codex"');
expect(html).not.toContain('class="slot"');
```

Remove the `pillHtml` test block. Run the focused test before production edits:

```bash
npx vitest run src/view.test.ts
```

Expected: FAIL because the current `index.html` still contains `id="pill"`.

- [ ] **Step 6: Remove the compact renderer and formatter**

In `src/view.ts`, delete `GEMS` and `pillHtml`; keep `spriteHtml`, `barHtml`, and `cardHtml` unchanged for now. In `src/format.ts`, delete `fmtCompactCountdown`. In `src/format.test.ts`, remove only the `fmtCompactCountdown` import and describe block.

Replace `index.html` body content with:

```html
<body>
  <div id="root" data-tauri-drag-region="deep">
    <div id="card">
      <section id="card-claude" class="provider-card claude"></section>
      <section id="card-codex" class="provider-card codex"></section>
    </div>
  </div>
</body>
```

- [ ] **Step 7: Simplify the frontend integration to one card**

In `src/main.ts`:

1. Import `fmtAge`, `fmtCountdown`, `manaLeft`, and `planLabel`; remove `fmtCompactCountdown`.
2. Import only `cardHtml` and `Snapshot` from `view`.
3. Import `ROSTER_WIDTH`, `createSerialQueue`, `rosterHeight`, and `rosterOrigin` from `window-layout`.
4. Delete `COLLAPSED`, `layoutIntent`, `collapsedOrigin`, `expandedOffsetX`, `setExpanded`, and `hoverIntent`.
5. Change `applyData` to accept one `card` root and update it directly.
6. Delete all `pill` lookup, rendering, stale, and sprite logic from `renderProvider`.
7. Make `tick()` always use `fmtCountdown`.
8. Keep `hovering`, but wire it only to sprite state:

```typescript
document.body.addEventListener("mouseenter", () => {
  hovering = true;
  updateSprites();
});
document.body.addEventListener("mouseleave", () => {
  hovering = false;
  updateSprites();
});
```

9. Replace `resizeExpandedContent` with this always-on roster sizing function and call it from `renderProvider`:

```typescript
function resizeRosterContent(): void {
  void enqueueSizing(async () => {
    const card = document.getElementById("card")!;
    const height = rosterHeight(card.scrollHeight);
    const win = getCurrentWindow();
    const position = await win.outerPosition();
    const monitor = await currentMonitor();
    const target = monitor
      ? rosterOrigin(
          position,
          { width: ROSTER_WIDTH, height },
          {
            x: monitor.workArea.position.x,
            y: monitor.workArea.position.y,
            width: monitor.workArea.size.width,
            height: monitor.workArea.size.height,
          },
          monitor.scaleFactor,
        )
      : position;
    await win.setSize(new LogicalSize(ROSTER_WIDTH, height));
    await win.setPosition(new PhysicalPosition(target.x, target.y));
  });
}
```

The movement-settle callback only resets `moving` and updates sprites; it no longer calls a collapse controller.

- [ ] **Step 8: Make the card visible by default and synchronize startup geometry**

In `src/styles.css`, delete the entire collapsed roster block and `body.expanded` selectors. Change `#card` to `display: flex` by default.

In `src-tauri/tauri.conf.json`, change the main window dimensions to:

```json
"width": 420,
"height": 175,
```

- [ ] **Step 9: Run permanent-roster verification**

Run:

```bash
npm test
npm run build
git diff --check
```

Expected: frontend tests PASS, TypeScript/Vite build exits 0, and diff check emits no output.

- [ ] **Step 10: Commit the permanent roster**

```bash
git add index.html src/main.ts src/view.ts src/view.test.ts src/vite-env.d.ts src/format.ts src/format.test.ts src/window-layout.ts src/window-layout.test.ts src/styles.css src-tauri/tauri.conf.json
git commit -m "feat: keep the party roster permanently expanded"
```

---

### Task 2: Build the fixed pixel-meter primitive

**Files:**
- Create: `src/meter.ts`
- Create: `src/meter.test.ts`
- Create: `public/hud/boss-bar-left.png`
- Create: `public/hud/boss-bar-mid.png`
- Create: `public/hud/boss-bar-right.png`
- Modify: `src/view.ts`
- Modify: `src/view.test.ts`
- Modify: `src/main.ts`
- Modify: `src/styles.css`

**Interfaces:**
- Produces: `METER_WIDTH`, `METER_HEIGHT`, `METER_INNER_WIDTH`, and `meterFillPixels(percent)` from `src/meter.ts`.
- Consumes: remaining-mana percentages from `manaLeft` and fixed pixel widths in both initial renderer markup and live updates.
- Asset sources: the three 32x16 Boss HUD pieces named in the approved spec.

- [ ] **Step 1: Write failing meter boundary tests**

Create `src/meter.test.ts`:

```typescript
import { describe, expect, it } from "vitest";
import {
  METER_HEIGHT,
  METER_INNER_WIDTH,
  METER_WIDTH,
  meterFillPixels,
} from "./meter";

describe("fixed pixel meter", () => {
  it("uses the approved frame and interior dimensions", () => {
    expect({ width: METER_WIDTH, height: METER_HEIGHT, inner: METER_INNER_WIDTH })
      .toEqual({ width: 128, height: 16, inner: 122 });
  });

  it.each([
    [-1, 0],
    [0, 0],
    [1, 1],
    [29, 35],
    [30, 37],
    [55, 67],
    [99, 121],
    [100, 122],
    [101, 122],
  ])("maps %s percent to %s interior pixels", (percent, pixels) => {
    expect(meterFillPixels(percent)).toBe(pixels);
  });
});
```

- [ ] **Step 2: Run the meter tests and verify RED**

Run:

```bash
npx vitest run src/meter.test.ts
```

Expected: FAIL because `src/meter.ts` does not exist.

- [ ] **Step 3: Implement logical meter geometry**

Create `src/meter.ts`:

```typescript
export const METER_WIDTH = 128;
export const METER_HEIGHT = 16;
export const METER_INSET_X = 3;
export const METER_INNER_WIDTH = METER_WIDTH - METER_INSET_X * 2;

export function meterFillPixels(percent: number): number {
  const clamped = Math.max(0, Math.min(100, percent));
  return Math.round((clamped / 100) * METER_INNER_WIDTH);
}
```

- [ ] **Step 4: Run the meter tests and verify GREEN**

Run:

```bash
npx vitest run src/meter.test.ts
```

Expected: all meter dimension and boundary cases PASS.

- [ ] **Step 5: Copy the licensed frame pieces unchanged**

Create `public/hud/`, then copy only these files:

```bash
mkdir -p public/hud
cp "/Volumes/SAMSUNG_SSD_990_PRO/Home/Downloads/Another Metroidvania Asset Pack ver. 1.7/User Interface/Boss Hud/health_bar_icon_left.png" public/hud/boss-bar-left.png
cp "/Volumes/SAMSUNG_SSD_990_PRO/Home/Downloads/Another Metroidvania Asset Pack ver. 1.7/User Interface/Boss Hud/health_bar_icon_mid.png" public/hud/boss-bar-mid.png
cp "/Volumes/SAMSUNG_SSD_990_PRO/Home/Downloads/Another Metroidvania Asset Pack ver. 1.7/User Interface/Boss Hud/health_bar_icon_right.png" public/hud/boss-bar-right.png
```

Verify byte-for-byte identity:

```bash
shasum -a 256 \
  "/Volumes/SAMSUNG_SSD_990_PRO/Home/Downloads/Another Metroidvania Asset Pack ver. 1.7/User Interface/Boss Hud/health_bar_icon_left.png" \
  public/hud/boss-bar-left.png \
  "/Volumes/SAMSUNG_SSD_990_PRO/Home/Downloads/Another Metroidvania Asset Pack ver. 1.7/User Interface/Boss Hud/health_bar_icon_mid.png" \
  public/hud/boss-bar-mid.png \
  "/Volumes/SAMSUNG_SSD_990_PRO/Home/Downloads/Another Metroidvania Asset Pack ver. 1.7/User Interface/Boss Hud/health_bar_icon_right.png" \
  public/hud/boss-bar-right.png
```

Expected: each source/destination pair has the same SHA-256 value.

- [ ] **Step 6: Drive initial and live fill widths from the pure helper**

Import `meterFillPixels` in `src/view.ts` and change `barHtml` to:

```typescript
function barHtml(snapshot: Snapshot, bar: Bar, index: number): string {
  const left = manaLeft(bar.used_percent);
  const pixels = meterFillPixels(left);
  return `<div class="track ${snapshot.provider}${left < 30 ? " low" : ""}" data-bar="${index}"><div class="fill" style="width:${pixels}px"></div></div>`;
}
```

Import `meterFillPixels` in `src/main.ts` and change live fill updates to:

```typescript
if (fill) fill.style.width = `${meterFillPixels(left)}px`;
```

Update the weekly-only renderer test (`used_percent: 55`, therefore 45% remaining) with:

```typescript
expect(html).toContain('class="fill" style="width:55px"');
```

- [ ] **Step 7: Implement the fixed frame and equal columns**

Replace the meter CSS with these structural rules before the retro polish task:

```css
.track {
  position: relative;
  width: 128px;
  min-width: 128px;
  height: 16px;
  overflow: visible;
  background: transparent;
}

.track::before,
.track::after {
  position: absolute;
  content: "";
  pointer-events: none;
}

.track::before {
  inset: 5px 3px;
  z-index: 0;
  background: rgba(8, 13, 26, 0.72);
}

.track::after {
  inset: 0;
  z-index: 3;
  background-image:
    url("/hud/boss-bar-left.png"),
    url("/hud/boss-bar-mid.png"),
    url("/hud/boss-bar-mid.png"),
    url("/hud/boss-bar-right.png");
  background-position: 0 0, 32px 0, 64px 0, 96px 0;
  background-repeat: no-repeat;
  image-rendering: pixelated;
}

.fill {
  position: absolute;
  top: 5px;
  left: 3px;
  z-index: 1;
  height: 6px;
  overflow: hidden;
  border-radius: 0;
}

.row {
  grid-template-columns: 52px 128px minmax(0, 1fr);
}
```

Remove the old repeating segment overlay and rounded track/fill geometry.

- [ ] **Step 8: Run meter integration verification**

Run:

```bash
npm test
npm run build
git diff --check
```

Expected: frontend tests PASS, the build exits 0, and diff check emits no output.

- [ ] **Step 9: Commit the fixed pixel meters**

```bash
git add public/hud src/meter.ts src/meter.test.ts src/view.ts src/view.test.ts src/main.ts src/styles.css
git commit -m "feat: add equal pixel-framed mana meters"
```

---

### Task 3: Apply the shared glow and retro glass system

**Files:**
- Modify: `src/styles.css`
- Modify: `src-tauri/src/lib.rs`

**Interfaces:**
- Consumes: the fixed 128x16 `.track`, 6px `.fill`, provider classes, `.low`, `.stale`, `[data-working]`, sprite states, and three frame images from Task 2.
- Produces: one visual system with 8px glass radius, 4px dither grid, square signals, universal fill glow/shimmer, and reduced-motion fallback.

- [ ] **Step 1: Capture the pre-change visual contract failure**

With the Vite server running, inspect the rendered roster and record these current computed values:

```javascript
({
  rootRadius: getComputedStyle(document.querySelector("#root")).borderRadius,
  tracks: [...document.querySelectorAll(".row .track")].map((el) => el.getBoundingClientRect().width),
  signalRadius: getComputedStyle(document.querySelector(".activity-signal")).borderRadius,
  fillAnimations: [...document.querySelectorAll(".fill")].map(
    (el) => getComputedStyle(el, "::before").animationName,
  ),
})
```

Expected before this task: root radius is `16px`, signal is circular, or fills do not all report the same shimmer animation. Save the result in the task report as RED evidence.

- [ ] **Step 2: Apply the retro token and container system**

Update `src/styles.css` tokens and structural styling to these exact decisions:

```css
:root {
  --void: #080d1a;
  --ink: #edf3ff;
  --ink-dim: #a8b5cc;
  --frame: #b9c8dc;
  --frame-hi: #f4f8ff;
  --claude-1: #48d6ff;
  --claude-2: #6f7cff;
  --claude-glow: rgba(72, 214, 255, 0.52);
  --codex-1: #c865ff;
  --codex-2: #e45ab7;
  --codex-glow: rgba(228, 90, 183, 0.5);
  --low-1: #fb7185;
  --low-2: #f43f5e;
  --low-glow: rgba(244, 63, 94, 0.7);
  --hud-radius: 8px;
}

#root {
  border-radius: var(--hud-radius);
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.09), transparent 16%),
    linear-gradient(135deg, rgba(18, 30, 53, 0.38), rgba(8, 13, 26, 0.58));
  box-shadow:
    inset 0 1px 0 rgba(255, 255, 255, 0.18),
    inset 0 -1px 0 rgba(4, 7, 16, 0.72),
    inset 0 0 34px rgba(91, 119, 195, 0.1);
}

#root::before {
  background-size: 4px 4px;
  opacity: 0.2;
}
```

Use the existing neutral glass and provider palette. Make the provider divider a hard one-pixel rule, use the monospace stack for the entire card, keep provider names 13px, labels/reset text 11px, and percentages 12px. Make `.activity-signal` 6x6px with `border-radius: 0`.

- [ ] **Step 3: Give every fill the same glow and stepped shimmer**

Use one fill rule for every provider and low state:

```css
.fill {
  background: linear-gradient(90deg, var(--c1), var(--c2));
  box-shadow:
    0 0 4px var(--glow),
    0 0 10px var(--glow);
  transition: width 0.45s steps(8, end);
}

.fill::before {
  position: absolute;
  inset: 0;
  background:
    linear-gradient(180deg, rgba(255, 255, 255, 0.72) 0 1px, transparent 1px),
    repeating-linear-gradient(90deg, transparent 0 5px, rgba(255, 255, 255, 0.2) 5px 6px);
  content: "";
  animation: mana-shimmer 1.2s steps(6, end) infinite;
}

@keyframes mana-shimmer {
  to {
    transform: translateX(6px);
  }
}
```

Delete `low-pulse`, `working-scan`, and their selectors. Keep provider and low classes limited to `--c1`, `--c2`, and `--glow` values. Activity remains the sprite plus square signal.

- [ ] **Step 4: Match the native glass radius and reduced motion**

In `src-tauri/src/lib.rs`, change the `apply_vibrancy` radius argument from `Some(14.0)` to `Some(8.0)`.

Update reduced motion to:

```css
@media (prefers-reduced-motion: reduce) {
  .sprite,
  .fill::before {
    animation: none;
  }

  .fill {
    transition: none;
  }
}
```

- [ ] **Step 5: Verify the computed visual contract GREEN**

Run the same browser expression from Step 1.

Expected:

```json
{
  "rootRadius": "8px",
  "tracks": [128, 128, 128, 128],
  "signalRadius": "0px",
  "fillAnimations": ["mana-shimmer", "mana-shimmer", "mana-shimmer", "mana-shimmer"]
}
```

The actual number of tracks follows live provider rows, but every entry must equal 128 and every non-empty fill must report `mana-shimmer`.

- [ ] **Step 6: Run frontend and Rust verification**

Run:

```bash
npm test
npm run build
rustfmt --edition 2021 --check src-tauri/src/lib.rs
cd src-tauri && cargo test && cargo check
git diff --check
```

Expected: all frontend and Rust tests PASS, both builds/checks exit 0, and diff check emits no output.

- [ ] **Step 7: Commit the retro glass system**

```bash
git add src/styles.css src-tauri/src/lib.rs
git commit -m "feat: apply the retro glass mana HUD"
```

---

### Task 4: Add attribution, version v0.4.0, and release verification

**Files:**
- Create: `THIRD_PARTY_NOTICES.md`
- Modify: `README.md`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: final permanent roster, imported Boss HUD assets, and current package/bundle metadata.
- Produces: legally complete attribution, synchronized v0.4.0 metadata, release app bundle, and native QA evidence.

- [ ] **Step 1: Add the complete asset notice**

Create `THIRD_PARTY_NOTICES.md`:

```markdown
# Third-Party Notices

## Another Metroidvania Asset Pack

The meter frame pieces in `public/hud/` are from **Another Metroidvania Asset
Pack** by **o_lobster**: <https://o-lobster.itch.io>.

Licensed under the Creative Commons Attribution 4.0 International License:
<https://creativecommons.org/licenses/by/4.0/>.

The PNG files are copied unchanged. mana composes one left piece, two middle
pieces, and one right piece in CSS as a 128x16 usage frame. The colored fills,
glow, and animation are original mana styling. Use of the assets does not imply
endorsement by o_lobster.
```

- [ ] **Step 2: Update README behavior and attribution**

Replace compact/hover wording with:

```markdown
Gamer mana bars for your AI subscriptions. This tiny always-on-top macOS widget
keeps a glass Party Roster visible with Claude Code and Codex usage across
session, weekly, and model-scoped limits. Every limit uses the same pixel HUD
frame and full reset time; Codex labels come from their actual duration.
```

In the Familiars section, describe Clawd and Nimbus as leading their permanent provider bands and celebrating on hover or drag. Add:

```markdown
## Asset credit

Pixel meter frames are from Another Metroidvania Asset Pack by
[o_lobster](https://o-lobster.itch.io), licensed under
[CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). See
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for details.
```

- [ ] **Step 3: Synchronize version metadata**

Set `0.4.0` in:

- root `package.json`
- root `package-lock.json` package records
- `src-tauri/Cargo.toml`
- mana package entry in `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

Run:

```bash
npm install --package-lock-only --ignore-scripts
cd src-tauri && cargo check
```

Expected: lockfiles remain synchronized and both commands exit 0.

- [ ] **Step 4: Run the complete automated verification gate**

Run:

```bash
npm test
npm run build
rustfmt --edition 2021 --check src-tauri/src/lib.rs
cd src-tauri && cargo test && cargo check
git diff --check
```

Expected: all frontend tests PASS, all Rust tests PASS, both builds/checks exit 0, and diff check emits no output.

- [ ] **Step 5: Build the macOS release bundle**

Run:

```bash
npm run tauri build
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
```

Expected: Tauri build exits 0 and PlistBuddy prints `0.4.0`.

- [ ] **Step 6: Perform browser and native visual QA**

Without starting a second mana widget alongside the installed one:

1. Use Vite/browser inspection for computed width, radius, animation-name, overflow, and reduced-motion checks.
2. Before native launch, ask permission to quit the single installed mana process and replace `/Applications/mana.app` with the v0.4.0 release through a reversible timestamped `/tmp` backup.
3. After approval, install the release bundle, launch `com.vantasoft.mana` through `cua-driver`, and verify `/Applications/mana.app` reports v0.4.0. If approval is withheld, leave v0.2.1 untouched and report installed-native QA as pending.
4. Capture the real Retina window and verify full reset values, equal 128px tracks, crisp frame pixels, distinct provider colors, uniform glow, no compact flash, no scrollbar, and no overlap.
5. Exercise 0%, 1%, 29%, 30%, 99%, 100%, stale, absent, working, and reduced-motion states with temporary development fixtures that are removed before final verification.
6. Verify right-edge containment and drag persistence.
7. Confirm only one mana process remains running.

- [ ] **Step 7: Commit release metadata and attribution**

```bash
git add THIRD_PARTY_NOTICES.md README.md package.json package-lock.json src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/tauri.conf.json
git commit -m "chore: release permanent retro HUD v0.4.0"
```

- [ ] **Step 8: Run final clean-state verification**

Run:

```bash
git status --short
git log -5 --oneline
```

Expected: only the user's pre-existing untracked `.DS_Store` remains; the four feature/release commits and this plan/spec history are present.
