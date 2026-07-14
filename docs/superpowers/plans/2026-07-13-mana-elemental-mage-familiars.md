# Mana Elemental Mage Familiars Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current framed pixel mascots with original, free-standing animated chibi mages: Codex as ice/lightning and Claude as fire/poison.

**Architecture:** Generate one imagegen-authored `4 x 3` RGBA atlas per character, validate the binary and per-cell alpha contract in Vitest, and reuse the current `idle`, `working`, and `hover` state machine through CSS sprite-sheet playback. Keep the glass roster, usage parsing, equal mana bars, native sizing, and provider activity detection unchanged.

**Tech Stack:** Built-in imagegen, the installed imagegen chroma-key helper, FFmpeg, PNG/RGBA, vanilla TypeScript, CSS sprite animation, Vitest, Tauri 2, Rust, and CuaDriver for native macOS QA.

## Global Constraints

- Implement the approved spec at `docs/superpowers/specs/2026-07-13-mana-elemental-mage-familiars-design.md`.
- Use each user attachment only as its corresponding character reference; the output must be original and must not reproduce game logos, named equipment, UI assets, exact costumes, or source-image details.
- Production atlases are exactly `448 x 336` RGBA PNGs in a `4 x 3` grid of `112 x 112` physical cells.
- CSS renders each atlas at `224 x 168`, producing `56 x 56` logical frames.
- Atlas rows are exactly `idle`, `working`, and `hover`; state priority remains `hover > working > idle`.
- Animation durations are exactly `1.15s` idle, `0.68s` working, and `0.82s` hover with `steps(4)`.
- Remove all visible familiar wrapper chrome; a direct sprite drop shadow and the existing provider header activity diamond are allowed.
- The widget remains permanently expanded, `440px` wide, content-height measured, and draggable.
- Claude retains all duration-derived rows; Codex Pro retains exactly one `Weekly` row.
- Mana tracks remain `144 x 20` with `116 x 8` live channels and glow-free zero fills.
- Do not modify usage parsing, polling, credential access, activity detection, tray behavior, persistence, mana calculation, or native activation policy.
- Do not add runtime dependencies.
- Leave the pre-existing untracked `.DS_Store` untouched.
- Release metadata ends synchronized at `0.4.1`.

---

### Task 1: Generate And Validate The Elemental Mage Atlases

**Files:**
- Create: `public/sprites/codex-ice-lightning.png`
- Create: `public/sprites/claude-fire-poison.png`
- Create: `src/sprites.test.ts`
- Temporary, ignored: `.superpowers/imagegen/codex-ice-lightning-source.png`
- Temporary, ignored: `.superpowers/imagegen/claude-fire-poison-source.png`
- Temporary, ignored: `.superpowers/imagegen/codex-ice-lightning-keyed.png`
- Temporary, ignored: `.superpowers/imagegen/claude-fire-poison-keyed.png`

**Interfaces:**
- Consumes: Codex reference `/var/folders/fd/_t3yrky9517_pp1jr863q3w40000gn/T/codex-clipboard-7ba9b12c-ce31-4468-a977-1632ba317c40.png`; Claude reference `/var/folders/fd/_t3yrky9517_pp1jr863q3w40000gn/T/codex-clipboard-db5e444e-d1ce-4e1c-a26c-fa82e7c4b1d8.png`.
- Produces: two `448 x 336` RGBA sprite atlases with transparent four-pixel cell perimeters and stable baselines, ready for `56 x 56` CSS playback.

- [ ] **Step 1: Write the failing binary asset-contract test**

Create `src/sprites.test.ts`:

```ts
// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { readFileSync } from "node:fs";
// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { inflateSync } from "node:zlib";
import { describe, expect, it } from "vitest";

type DecodedPng = {
  width: number;
  height: number;
  pixels: Uint8Array;
};

function uint32(bytes: Uint8Array, offset: number): number {
  return new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint32(offset);
}

function concatBytes(parts: Uint8Array[]): Uint8Array {
  const result = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let offset = 0;
  for (const part of parts) {
    result.set(part, offset);
    offset += part.length;
  }
  return result;
}

function paeth(left: number, up: number, upperLeft: number): number {
  const estimate = left + up - upperLeft;
  const leftDistance = Math.abs(estimate - left);
  const upDistance = Math.abs(estimate - up);
  const upperLeftDistance = Math.abs(estimate - upperLeft);
  if (leftDistance <= upDistance && leftDistance <= upperLeftDistance) return left;
  return upDistance <= upperLeftDistance ? up : upperLeft;
}

function decodeRgba(url: URL): DecodedPng {
  const bytes = new Uint8Array(readFileSync(url));
  expect(Array.from(bytes.subarray(0, 8))).toEqual([137, 80, 78, 71, 13, 10, 26, 10]);
  const width = uint32(bytes, 16);
  const height = uint32(bytes, 20);
  expect(bytes[24]).toBe(8);
  expect(bytes[25]).toBe(6);
  expect(bytes[28]).toBe(0);

  const idat: Uint8Array[] = [];
  for (let cursor = 8; cursor < bytes.length; ) {
    const length = uint32(bytes, cursor);
    const type = String.fromCharCode(...bytes.subarray(cursor + 4, cursor + 8));
    if (type === "IDAT") idat.push(bytes.slice(cursor + 8, cursor + 8 + length));
    cursor += length + 12;
  }

  const filtered = new Uint8Array(inflateSync(concatBytes(idat)));
  const stride = width * 4;
  const pixels = new Uint8Array(stride * height);
  for (let y = 0; y < height; y += 1) {
    const sourceStart = y * (stride + 1);
    const filter = filtered[sourceStart];
    expect(filter).toBeGreaterThanOrEqual(0);
    expect(filter).toBeLessThanOrEqual(4);
    for (let x = 0; x < stride; x += 1) {
      const raw = filtered[sourceStart + 1 + x];
      const target = y * stride + x;
      const left = x >= 4 ? pixels[target - 4] : 0;
      const up = y > 0 ? pixels[target - stride] : 0;
      const upperLeft = y > 0 && x >= 4 ? pixels[target - stride - 4] : 0;
      const value =
        filter === 0 ? raw :
        filter === 1 ? raw + left :
        filter === 2 ? raw + up :
        filter === 3 ? raw + Math.floor((left + up) / 2) :
        raw + paeth(left, up, upperLeft);
      pixels[target] = value & 0xff;
    }
  }
  return { width, height, pixels };
}

function verifyAtlas(relativePath: string): void {
  const image = decodeRgba(new URL(relativePath, import.meta.url));
  expect([image.width, image.height]).toEqual([448, 336]);
  const cell = 112;
  const alphaAt = (x: number, y: number) => image.pixels[(y * image.width + x) * 4 + 3];

  for (let row = 0; row < 3; row += 1) {
    const bottoms: number[] = [];
    for (let column = 0; column < 4; column += 1) {
      let edgeAlpha = 0;
      let visible = 0;
      let bottom = -1;
      for (let y = 0; y < cell; y += 1) {
        for (let x = 0; x < cell; x += 1) {
          const alpha = alphaAt(column * cell + x, row * cell + y);
          if (x < 4 || x >= cell - 4 || y < 4 || y >= cell - 4) {
            edgeAlpha = Math.max(edgeAlpha, alpha);
          }
          if (alpha > 16) {
            visible += 1;
            bottom = Math.max(bottom, y);
          }
        }
      }
      expect(edgeAlpha).toBe(0);
      expect(visible).toBeGreaterThan(600);
      expect(visible).toBeLessThan(10_500);
      expect(bottom).toBeGreaterThan(60);
      bottoms.push(bottom);
    }
    expect(Math.max(...bottoms) - Math.min(...bottoms)).toBeLessThanOrEqual(12);
  }
}

describe("elemental mage sprite atlases", () => {
  it("keeps the Codex atlas aligned, padded, and transparent", () => {
    verifyAtlas("../public/sprites/codex-ice-lightning.png");
  });

  it("keeps the Claude atlas aligned, padded, and transparent", () => {
    verifyAtlas("../public/sprites/claude-fire-poison.png");
  });
});
```

- [ ] **Step 2: Run the focused test and verify the red state**

Run:

```bash
npx vitest run src/sprites.test.ts
```

Expected: FAIL because both new atlas paths are absent.

- [ ] **Step 3: Generate the Codex atlas source with built-in imagegen**

Load the Codex reference with `view_image`, then call the built-in imagegen tool with that file as the edit/reference target and this prompt:

```text
Use case: stylized-concept
Asset type: production 4-column by 3-row animation sprite atlas for a macOS fantasy usage widget
Input image: use the attached cloud-terminal wizard only as the Codex character reference
Primary request: reinterpret the character as an original chibi ice and lightning mage; keep the friendly cloud-terminal identity but create a new royal-blue, icy-cyan, and restrained gold costume, a new crystalline staff, and an original silhouette
Grid: exactly 4 equal columns and 3 equal rows, twelve cells total, no gutters, no borders, no labels; one complete character pose centered in each cell at the same scale and baseline
Row 1: four-frame calm breathing or floating idle loop
Row 2: four-frame compact spell-casting loop, staff motion plus crisp ice crystal and blue-white lightning shapes
Row 3: four-frame upbeat victory loop, staff raised plus a small snowflake sparkle
Style: polished hand-painted 2D chibi side-scrolling fantasy RPG sprite, clean dark outline, oversized head, compact body, readable at small size, consistent character model in every cell
Scene/backdrop: perfectly flat solid #ff00ff chroma-key background across the entire canvas
Constraints: each full pose including staff and effects stays inside its cell with generous empty padding; opaque or near-opaque crisp effects; consistent lighting, costume, staff, proportions, face, outline thickness, and ground line
Avoid: exact source costume, game logos, named equipment, copied UI, text, watermark, checkerboard, cell dividers, shadows on the background, gradients, reflections, floor plane, smoke, translucent fog, motion blur, cropped staff, duplicated limbs, or #ff00ff in the character
```

Render the result inline and record the built-in tool's returned local source as `CODEX_SOURCE`; Step 5 copies it into ignored project scratch.

- [ ] **Step 4: Generate the Claude atlas source with built-in imagegen**

Load the Claude reference with `view_image`, then call the built-in imagegen tool with that file as the edit/reference target and this prompt:

```text
Use case: stylized-concept
Asset type: production 4-column by 3-row animation sprite atlas for a macOS fantasy usage widget
Input image: use the attached warm star-shaped familiar wizard only as the Claude character reference
Primary request: reinterpret the character as an original chibi fire and poison mage; keep the friendly warm star-like identity but create a new ivory, ember-orange, charcoal, gold, and violet costume, a new curved wooden staff, and an original silhouette
Grid: exactly 4 equal columns and 3 equal rows, twelve cells total, no gutters, no borders, no labels; one complete character pose centered in each cell at the same scale and baseline
Row 1: four-frame calm breathing idle loop with gentle robe or leaf movement
Row 2: four-frame compact spell-casting loop alternating crisp ember-orange fire and violet poison energy around the staff
Row 3: four-frame cheerful victory loop with a small flame-and-leaf flourish
Style: polished hand-painted 2D chibi side-scrolling fantasy RPG sprite, clean dark outline, oversized head, compact body, readable at small size, consistent character model in every cell
Scene/backdrop: perfectly flat solid #ff00ff chroma-key background across the entire canvas
Constraints: each full pose including staff and effects stays inside its cell with generous empty padding; opaque or near-opaque crisp effects; consistent lighting, costume, staff, proportions, face, outline thickness, and ground line
Avoid: exact source costume, game logos, named equipment, copied UI, text, watermark, checkerboard, cell dividers, shadows on the background, gradients, reflections, floor plane, smoke, translucent fog, motion blur, cropped staff, duplicated limbs, or #ff00ff in the character
```

Render the result inline and record the built-in tool's returned local source as `CLAUDE_SOURCE`; Step 5 copies it into ignored project scratch.

- [ ] **Step 5: Normalize both sources and remove the chroma key**

`CODEX_SOURCE` and `CLAUDE_SOURCE` are the exact local paths returned by Steps 3 and 4. Run:

```bash
mkdir -p .superpowers/imagegen public/sprites
cp "$CODEX_SOURCE" .superpowers/imagegen/codex-ice-lightning-source.png
cp "$CLAUDE_SOURCE" .superpowers/imagegen/claude-fire-poison-source.png

ffmpeg -y -i .superpowers/imagegen/codex-ice-lightning-source.png \
  -vf "crop='min(iw,ih*4/3)':'min(ih,iw*3/4)',scale=448:336:flags=lanczos,format=rgb24" \
  -frames:v 1 .superpowers/imagegen/codex-ice-lightning-keyed.png
ffmpeg -y -i .superpowers/imagegen/claude-fire-poison-source.png \
  -vf "crop='min(iw,ih*4/3)':'min(ih,iw*3/4)',scale=448:336:flags=lanczos,format=rgb24" \
  -frames:v 1 .superpowers/imagegen/claude-fire-poison-keyed.png

python "${CODEX_HOME:-$HOME/.codex}/skills/.system/imagegen/scripts/remove_chroma_key.py" \
  --input .superpowers/imagegen/codex-ice-lightning-keyed.png \
  --out public/sprites/codex-ice-lightning.png \
  --key-color '#ff00ff' --soft-matte --transparent-threshold 12 \
  --opaque-threshold 220 --despill --force
python "${CODEX_HOME:-$HOME/.codex}/skills/.system/imagegen/scripts/remove_chroma_key.py" \
  --input .superpowers/imagegen/claude-fire-poison-keyed.png \
  --out public/sprites/claude-fire-poison.png \
  --key-color '#ff00ff' --soft-matte --transparent-threshold 12 \
  --opaque-threshold 220 --despill --force
```

- [ ] **Step 6: Inspect the atlases and perform the single allowed targeted retry if needed**

Inspect both production PNGs with `view_image` at original detail. Require exactly twelve complete poses, stable character identity, correct row semantics, consistent cell alignment, transparent outer areas, no magenta fringe, and no cropped staff or spell shape.

If one atlas fails character consistency or grid alignment, make one imagegen edit using the failed generated source and this targeted prompt, then repeat Step 5 only for that character:

```text
Preserve the approved character design, palette, costume, staff, lighting, and flat #ff00ff background. Correct only the 4-column by 3-row atlas consistency: exactly twelve equal cells, same character scale and baseline, one complete pose per cell, generous cell padding, no gutters or labels, and no cropped equipment. Keep the three row actions unchanged.
```

- [ ] **Step 7: Run asset gates and commit**

Run:

```bash
sips -g pixelWidth -g pixelHeight -g format -g hasAlpha \
  public/sprites/codex-ice-lightning.png \
  public/sprites/claude-fire-poison.png
npx vitest run src/sprites.test.ts
git diff --check
```

Expected: both images report `448 x 336`, PNG, alpha; 2 focused tests pass; diff check is silent.

Commit:

```bash
git add public/sprites/codex-ice-lightning.png \
  public/sprites/claude-fire-poison.png src/sprites.test.ts
git commit -m "feat: add elemental mage sprite atlases"
```

---

### Task 2: Integrate Free-Standing Mage Animations

**Files:**
- Modify: `src/view.ts`
- Modify: `src/view.test.ts`
- Modify: `src/styles.css`
- Modify: `src/styles.test.ts`
- Modify: `src/sprites.test.ts`
- Delete: `public/sprites/clawd.png`
- Delete: `public/sprites/nimbus.png`
- Delete: `scripts/gen-sprites.py`

**Interfaces:**
- Consumes: `codex-ice-lightning.png` and `claude-fire-poison.png`; existing `data-provider` and `data-state` state-machine attributes.
- Produces: semantic `codex-mage` and `claude-mage` sprite classes rendered as free-standing `56 x 56` animated characters.

- [ ] **Step 1: Update markup and stylesheet tests first**

In `src/view.test.ts`, replace the old class expectations:

```ts
expect(html).toContain('class="sprite codex-mage"');
```

and:

```ts
expect(html).toContain('class="sprite claude-mage"');
```

In `src/styles.test.ts`, add this test:

```ts
it("renders free-standing illustrated mage atlases", () => {
  expect(styles).toContain('url("/sprites/claude-fire-poison.png")');
  expect(styles).toContain('url("/sprites/codex-ice-lightning.png")');
  expect(styles).not.toContain("clawd.png");
  expect(styles).not.toContain("nimbus.png");
  expect(styles).not.toContain(".familiar-slot::before");
  expect(styles).not.toContain(".familiar-slot::after");
  expect(styles).not.toContain("image-rendering: pixelated");
  expect(styles).toMatch(/#card section\s*\{[^}]*grid-template-columns:\s*60px minmax\(0, 1fr\)/s);
  expect(styles).toMatch(/\.sprite\s*\{[^}]*width:\s*56px[^}]*height:\s*56px[^}]*background-size:\s*224px 168px[^}]*animation:\s*sprite-run 1\.15s steps\(4\) infinite/s);
  expect(styles).toMatch(/\.sprite\[data-state="working"\]\s*\{[^}]*background-position-y:\s*-56px[^}]*animation-duration:\s*0\.68s/s);
  expect(styles).toMatch(/\.sprite\[data-state="hover"\]\s*\{[^}]*background-position-y:\s*-112px[^}]*animation-duration:\s*0\.82s/s);
  expect(styles).toMatch(/@keyframes sprite-run\s*\{[^}]*background-position-x:\s*0[^}]*\}[^}]*background-position-x:\s*-224px/s);
});
```

In the existing reduced-motion test, remove both expectations containing `.provider-card[data-working] .familiar-slot::after`. Keep the sprite, fill glint, and activity-signal expectations.

At the top of `src/sprites.test.ts`, change the filesystem import to:

```ts
// @ts-expect-error Vitest runs in Node, while the app intentionally omits Node types.
import { existsSync, readFileSync } from "node:fs";
```

Then add:

```ts
it("retires the deterministic pixel familiar assets", () => {
  expect(existsSync(new URL("../public/sprites/clawd.png", import.meta.url))).toBe(false);
  expect(existsSync(new URL("../public/sprites/nimbus.png", import.meta.url))).toBe(false);
  expect(existsSync(new URL("../scripts/gen-sprites.py", import.meta.url))).toBe(false);
});
```

- [ ] **Step 2: Run focused tests and verify they fail for the intended reasons**

Run:

```bash
npx vitest run src/view.test.ts src/styles.test.ts src/sprites.test.ts
```

Expected: FAIL on old sprite classes, old dimensions/URLs, visible wrapper pseudo-elements, and the still-present retired files.

- [ ] **Step 3: Rename the runtime sprite classes**

Replace `spriteHtml` in `src/view.ts` with:

```ts
function spriteHtml(provider: string): string {
  const className = provider === "claude" ? "claude-mage" : "codex-mage";
  return `<div class="sprite ${className}" data-provider="${provider}" data-state="idle" aria-hidden="true"></div>`;
}
```

Do not change `updateSprites`, `spriteState`, activity listeners, hover listeners, or move timing in `src/main.ts`.

- [ ] **Step 4: Remove the familiar chrome and install the new atlas geometry**

Apply these exact structural changes in `src/styles.css`:

```css
#card section {
  display: grid;
  grid-template-columns: 60px minmax(0, 1fr);
  min-width: 0;
  gap: 10px;
  padding: 6px 0 14px;
}

.familiar-slot {
  position: relative;
  display: flex;
  width: 60px;
  min-height: 56px;
  align-items: center;
  justify-content: center;
}
```

Delete the `.familiar-slot::before`, `.familiar-slot::after`, their declaration block, and the working animation selector for `.provider-card[data-working] .familiar-slot::after`. Keep only:

```css
.provider-card[data-working] .activity-signal {
  animation: status-pulse 1.6s ease-in-out infinite;
}
```

Remove `--portrait-bg` from `.claude` and `.codex`. Replace the sprite rules with:

```css
.sprite {
  position: relative;
  z-index: 1;
  flex: none;
  width: 56px;
  height: 56px;
  background-repeat: no-repeat;
  background-size: 224px 168px;
  animation: sprite-run 1.15s steps(4) infinite;
}

.sprite.claude-mage {
  background-image: url("/sprites/claude-fire-poison.png");
  filter:
    drop-shadow(0 2px 1px rgba(0, 0, 0, 0.42))
    drop-shadow(0 0 5px rgba(255, 106, 67, 0.24));
}

.sprite.codex-mage {
  background-image: url("/sprites/codex-ice-lightning.png");
  filter:
    drop-shadow(0 2px 1px rgba(0, 0, 0, 0.42))
    drop-shadow(0 0 5px rgba(88, 204, 255, 0.26));
}

.sprite[data-state="idle"] {
  background-position-y: 0;
}

.sprite[data-state="working"] {
  background-position-y: -56px;
  animation-duration: 0.68s;
}

.sprite[data-state="hover"] {
  background-position-y: -112px;
  animation-duration: 0.82s;
}
```

Replace `@keyframes sprite-run` with:

```css
@keyframes sprite-run {
  from {
    background-position-x: 0;
  }
  to {
    background-position-x: -224px;
  }
}
```

Replace the stale familiar selector with only the remaining status accent:

```css
.stale .activity-signal {
  opacity: 0.58;
  filter: saturate(0.35);
}
```

In `@media (prefers-reduced-motion: reduce)`, remove `.provider-card[data-working] .familiar-slot::after` and retain:

```css
.sprite,
.fill::before,
.provider-card[data-working] .activity-signal {
  animation: none;
}
```

- [ ] **Step 5: Delete the retired pixel assets and generator**

Run:

```bash
rm public/sprites/clawd.png public/sprites/nimbus.png scripts/gen-sprites.py
```

- [ ] **Step 6: Run focused and full frontend gates**

Run:

```bash
npx vitest run src/view.test.ts src/styles.test.ts src/sprites.test.ts
npm test
npm run build
rg -n "clawd\.png|nimbus\.png|sprite clawd|sprite nimbus" \
  src public index.html scripts README.md || true
git diff --check
```

Expected: focused and full tests pass; build passes; runtime/source search is empty; diff check is silent.

- [ ] **Step 7: Commit the integration**

```bash
git add src/view.ts src/view.test.ts src/styles.css src/styles.test.ts \
  src/sprites.test.ts public/sprites scripts/gen-sprites.py
git commit -m "feat: animate elemental mage familiars"
```

---

### Task 3: Browser QA, Release, Replace, And Launch

**Files:**
- Temporary, ignored: `.superpowers/mage-sprite-qa.html`
- Modify: `README.md`
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/Cargo.lock`
- Modify: `src-tauri/tauri.conf.json`

**Interfaces:**
- Consumes: integrated free-standing mage sprites and the existing `440px` content-measured roster.
- Produces: browser/native visual evidence and a backed-up, installed, single-instance `mana v0.4.1`.

- [ ] **Step 1: Create the ignored browser fixture**

Create `.superpowers/mage-sprite-qa.html` with:

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <link rel="stylesheet" href="/src/styles.css" />
    <title>mana mage sprite QA</title>
  </head>
  <body>
    <div id="root">
      <main id="card">
        <section id="card-claude" class="provider-card claude">
          <div class="familiar-slot"><div class="sprite claude-mage" data-provider="claude" aria-hidden="true"></div></div>
          <div class="provider-content">
            <div class="head"><strong>Claude</strong><span class="plan">Max</span><span class="activity-signal"></span><span class="age"></span></div>
            <div class="rows">
              <div class="row"><span class="lbl">5 hour</span><div class="track claude"><div class="fill" data-empty="false" style="width:116px"></div></div><span class="val"><b>100%</b><span> · 4h 34m</span></span></div>
              <div class="row"><span class="lbl">Weekly</span><div class="track claude"><div class="fill" data-empty="false" style="width:42px"></div></div><span class="val"><b>36%</b><span> · Tue 1:59 PM</span></span></div>
              <div class="row"><span class="lbl">Fable</span><div class="track claude"><div class="fill" data-empty="true" style="width:0"></div></div><span class="val"><b>0%</b><span> · Tue 1:59 PM</span></span></div>
            </div>
          </div>
        </section>
        <section id="card-codex" class="provider-card codex">
          <div class="familiar-slot"><div class="sprite codex-mage" data-provider="codex" aria-hidden="true"></div></div>
          <div class="provider-content">
            <div class="head"><strong>Codex</strong><span class="plan">Pro</span><span class="activity-signal"></span><span class="age"></span></div>
            <div class="rows">
              <div class="row"><span class="lbl">Weekly</span><div class="track codex"><div class="fill" data-empty="false" style="width:94px"></div></div><span class="val"><b>81%</b><span> · Mon 5:40 PM</span></span></div>
            </div>
          </div>
        </section>
      </main>
    </div>
    <script>
      const state = new URLSearchParams(location.search).get("state") || "idle";
      document.querySelectorAll(".sprite").forEach((sprite) => { sprite.dataset.state = state; });
      document.querySelectorAll(".provider-card").forEach((card) => { card.toggleAttribute("data-working", state === "working"); });
    </script>
  </body>
</html>
```

- [ ] **Step 2: Start Vite and inspect all three production states**

Run:

```bash
npm run dev -- --host 127.0.0.1 --port 1431
```

Use the in-app browser control skill to open these URLs at an initial `440 x 280` viewport:

```text
http://127.0.0.1:1431/.superpowers/mage-sprite-qa.html?state=idle
http://127.0.0.1:1431/.superpowers/mage-sprite-qa.html?state=working
http://127.0.0.1:1431/.superpowers/mage-sprite-qa.html?state=hover
```

On each URL, evaluate `Math.ceil(document.getElementById("card").scrollHeight + 2)` and set the browser viewport to `440px` by that returned content height before capturing or measuring.

For each state, capture a screenshot and run this browser evaluation:

```js
(() => {
  const root = document.querySelector("#root").getBoundingClientRect();
  const sections = [...document.querySelectorAll("section")].map((section) => {
    const sprite = section.querySelector(".sprite").getBoundingClientRect();
    const content = section.querySelector(".provider-content").getBoundingClientRect();
    return {
      sprite: [sprite.width, sprite.height],
      noOverlap: sprite.right <= content.left,
      insideRoot: sprite.left >= root.left && sprite.right <= root.right && sprite.top >= root.top && sprite.bottom <= root.bottom,
    };
  });
  return {
    rootWidth: root.width,
    horizontalOverflow: document.documentElement.scrollWidth > document.documentElement.clientWidth,
    tracks: [...document.querySelectorAll(".track")].map((track) => {
      const rect = track.getBoundingClientRect();
      return [rect.width, rect.height];
    }),
    codexLabels: [...document.querySelectorAll("#card-codex .lbl")].map((label) => label.textContent),
    sections,
    animations: [...document.querySelectorAll(".sprite")].map((sprite) => getComputedStyle(sprite).animation),
  };
})()
```

Expected: root width `440`; no overflow; all tracks `[144, 20]`; Codex labels exactly `["Weekly"]`; both sprites `[56, 56]`, inside the root, and left of provider content; animations contain `steps(4)` and the state-appropriate duration. Screenshots must show free-standing characters with no portrait, pedestal, background panel, clipping, overlap, or magenta fringe.

Wait at least `300ms` and capture a second screenshot for each state. Confirm a visible frame change. Inspect the loaded CSSOM reduced-motion rule and require `.sprite` under `prefers-reduced-motion: reduce` with `animation: none`.

- [ ] **Step 3: Update release copy and version metadata**

Run:

```bash
npm version 0.4.1 --no-git-tag-version
```

Update `README.md` opening copy to:

```markdown
Gamer mana bars for your AI subscriptions. This tiny always-on-top macOS widget
shows how much Claude Code and Codex usage you have left across session, weekly,
and model-scoped limits in a permanently expanded smoked-glass Party Roster.
Free-standing elemental mages sit beside their providers: Claude channels fire
and poison, while Codex casts ice and lightning. An original silver, gold, and
crystal fantasy frame holds equal-length live energy cores and complete reset
times. Codex limits are named from their actual duration, including weekly-only
Pro accounts.
```

Replace the `## Familiars` body with:

```markdown
Claude's fire/poison mage and Codex's ice/lightning mage are original illustrated
chibi characters generated as Retina sprite atlases. They breathe while idle,
cast while their CLI is actively running (local process check every 5s - nothing
leaves the machine), and celebrate whenever you hover or drag the widget. The
characters stand freely beside their provider bands without portrait frames.
Animations and the magical energy glint respect your Reduced Motion setting.
```

Change the package version in `src-tauri/Cargo.toml` and the app version in `src-tauri/tauri.conf.json` from `0.4.0` to `0.4.1`, then run:

```bash
cargo check --manifest-path src-tauri/Cargo.toml
```

Expected: Cargo updates only the local `mana` package entry in `src-tauri/Cargo.lock` to `0.4.1` and `cargo check` passes.

- [ ] **Step 4: Run the complete release gate**

Run:

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
git diff --check
git status --short
```

Expected: all frontend tests pass, all 18 Rust tests pass, both builds pass, diff check is silent, and status contains only intended release files plus the pre-existing `.DS_Store`.

- [ ] **Step 5: Commit the release metadata**

```bash
git add README.md package.json package-lock.json src-tauri/Cargo.toml \
  src-tauri/Cargo.lock src-tauri/tauri.conf.json
git commit -m "chore: release mana v0.4.1"
```

- [ ] **Step 6: Build and verify the macOS bundle**

Run:

```bash
npm run tauri build
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c 'Print :CFBundleVersion' \
  src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' \
  src-tauri/target/release/bundle/macos/mana.app/Contents/Info.plist
```

Expected: `0.4.1`, `0.4.1`, and `com.vantasoft.mana`.

- [ ] **Step 7: Back up and replace the approved installed app**

Read the CuaDriver skill. Use its accessibility snapshot and tray interaction to quit the running `com.vantasoft.mana` app; do not use `open`, `kill`, `pkill`, `killall`, AppleScript activation, or a direct binary launch. Then run:

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

Record `app_backup`, `state_backup`, and the returned process ID.

- [ ] **Step 8: Perform native Retina QA**

Use CuaDriver to list and capture the installed `mana` window without making another app frontmost. Require:

- Exactly one `/Applications/mana.app/Contents/MacOS/mana` process.
- Installed `CFBundleShortVersionString` `0.4.1`.
- Logical window width `440` with measured content height and no clipping.
- Both free-standing mage characters fully visible beside their own sections.
- Sharp illustrated edges, transparent surroundings, no magenta fringe, and no portrait or pedestal.
- One Codex `Weekly` row and all Claude rows, labels, bars, percentages, and reset times readable.
- Animation visible over two captures taken at least `300ms` apart.
- No regression to equal mana-bar length, zero-fill glow, glass radius, always-on-top behavior, or right-edge clamping.

Run the read-only confirmation:

```bash
/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' \
  /Applications/mana.app/Contents/Info.plist
test "$(pgrep -x mana | wc -l | tr -d ' ')" = "1"
ps -p "$(pgrep -x mana)" -o pid=,args=
```

Expected: version `0.4.1`, one process, and executable `/Applications/mana.app/Contents/MacOS/mana`.

- [ ] **Step 9: Run the final clean-tree and evidence gate**

Stop the Vite and CuaDriver daemon sessions without stopping the installed app. Remove only ignored imagegen and browser-QA scratch created by this plan. Then run:

```bash
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
cargo check --manifest-path src-tauri/Cargo.toml
git diff --check
git status --short
```

Expected: frontend and Rust suites pass, builds pass, diff check is silent, installed mana remains running, and the only untracked path is the untouched `.DS_Store`.
