# Mana Compact Integrated Frame Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Restore Mana's compact 456px widget, integrate motif-only corners with the rails, align characters with their auras, and move lifetime output into an XP-bar tooltip.

**Architecture:** Keep the existing generated frame and animation pipeline, but change its presentation geometry. The native window and glass share one boundary; 48px corner art is clipped inside 32px motif boxes; rails meet beneath those motifs; the footer keeps stable level copy while the XP bar owns a non-layout tooltip.

**Tech Stack:** Tauri v2, TypeScript 5.6, vanilla HTML/CSS, Vite 6, browser preview QA.

## Global Constraints

- Do not write or run tests.
- Keep provider visibility, meter widths, progression math, saved progress, and generated frame source assets unchanged.
- The native root width is exactly 456px.
- The glass fills the root with the existing 8px radius.
- Corner motifs render in 32px boxes from existing 48px generated corner images.
- Rail centers sit 16px from the corresponding edges and continue beneath the motifs.
- Raise both aura layers exactly 8px.
- Lifetime output is hidden except for the XP-bar tooltip.
- Do not install or launch the built application against live progress.

---

### Task 1: Compact The Native Window And Integrate The Perimeter

**Files:**
- Modify: `src/window-layout.ts`
- Modify: `src/styles.css`

**Interfaces:**
- `scaledRosterSize(contentHeight, scale)` returns the exact native root size.
- Existing `--frame-rank-*` and `--frame-prestige-*` URLs remain unchanged.

- [ ] **Step 1: Remove external frame bleed**

Set the authored window width to the roster width and remove added frame space:

```ts
export const ROSTER_WIDTH = 456;
export const FRAME_BLEED = 0;
export const WINDOW_WIDTH = ROSTER_WIDTH;
```

`scaledRosterSize` must return `rosterHeight(contentHeight)` without adding
external decoration height.

- [ ] **Step 2: Make glass and vibrancy share one boundary**

Update the glass geometry:

```css
#glass {
  inset: 0;
  width: 100%;
  height: 100%;
  overflow: visible;
}
```

Keep the existing radius, background, and glass shadows.

- [ ] **Step 3: Clip socket arms and meet rails beneath motifs**

Use these production coordinates:

```css
.frame-rail--top,
.frame-rail--bottom {
  right: 16px;
  left: 16px;
}

.frame-rail--top { top: 8px; }
.frame-rail--bottom { bottom: 8px; }

.frame-rail--right,
.frame-rail--left {
  top: 16px;
  bottom: 16px;
}

.frame-rail--right { right: 8px; }
.frame-rail--left { left: 8px; }

.frame-corner {
  width: 32px;
  height: 32px;
  overflow: hidden;
  background-size: 48px 48px;
}
```

Position directional corner backgrounds toward their corresponding outer
corner. Apply the same directional positions and 48px size to the animated
corner mask. Move rail ornaments onto the same 8px rail coordinates. Hide the
top frame crest so the compact frame is defined by rails and corner motifs
without overlapping provider content.

- [ ] **Step 4: Verify static geometry**

Run:

```bash
npm run build
git diff --check
```

Expected: production frontend build succeeds and whitespace validation is
clean.

- [ ] **Step 5: Commit**

```bash
git add src/window-layout.ts src/styles.css
git commit -m "fix: compact generated frame perimeter"
```

---

### Task 2: Align Aura Ground And Exclude Visual Overflow From Sizing

**Files:**
- Modify: `src/styles.css`
- Modify: `src/main.ts`
- Modify: `src/preview.ts`

**Interfaces:**
- Aura atlases remain 96px CSS cells and keep their existing frame timing.
- Native and preview sizing both consume the content layout height.

- [ ] **Step 1: Raise the aura**

Change the aura anchor without moving the sprite:

```css
.aura {
  top: calc(50% - 8px);
}
```

- [ ] **Step 2: Measure layout height instead of overflowing effects**

In the native resize path and browser preview, pass `content.offsetHeight` to
the existing roster sizing helpers rather than `content.scrollHeight`.

- [ ] **Step 3: Verify**

Run:

```bash
npm run build
cargo check --manifest-path src-tauri/Cargo.toml --release
git diff --check
```

Expected: frontend and Rust release checks succeed.

- [ ] **Step 4: Commit**

```bash
git add src/styles.css src/main.ts src/preview.ts
git commit -m "fix: register characters with aura ground"
```

---

### Task 3: Replace Body-Wide Lifetime Copy With XP Tooltip

**Files:**
- Modify: `index.html`
- Modify: `src/main.ts`
- Modify: `src/preview.ts`
- Modify: `src/styles.css`

**Interfaces:**
- `lifetimeOutputLabel(decimal)` remains the exact formatting function.
- `hover=true` in the preview deterministically opens the XP tooltip.

- [ ] **Step 1: Move lifetime copy into the XP bar**

Use stable footer markup:

```html
<span class="progress-copy">
  <span class="level"></span>
</span>
<div class="xpbar" tabindex="0" aria-describedby="lifetime-output-tooltip">
  <div class="xpfill"></div>
  <span
    id="lifetime-output-tooltip"
    class="lifetime-output"
    role="tooltip"
  ></span>
</div>
```

- [ ] **Step 2: Remove body-wide copy swapping**

Keep body hover only for sprite animation state. Remove `data-hovering`
assignment and all CSS that replaces `.level` on root hover.

- [ ] **Step 3: Style a non-layout tooltip**

Position `.lifetime-output` absolutely above the XP bar. It must remain hidden
by default and become visible for `.xpbar:hover`, `.xpbar:focus-visible`, or
preview-only `data-tooltip-open="true"`. Use the existing HUD ink, glass, gold
edge, and monospace data typography. Tooltip visibility must not alter footer,
XP bar, or root bounds.

- [ ] **Step 4: Update preview determinism**

Populate the tooltip inside `.xpbar` and set:

```ts
xpbar.dataset.tooltipOpen = String(options.hovering);
```

- [ ] **Step 5: Verify**

Run:

```bash
npm run build
git diff --check
```

Expected: build and static checks succeed.

- [ ] **Step 6: Commit**

```bash
git add index.html src/main.ts src/preview.ts src/styles.css
git commit -m "fix: show lifetime output as xp tooltip"
```

---

### Task 4: Browser QA, Screenshot, And Build-Only Bundle

**Files:**
- Replace: `docs/images/mana-widget.png`

**Interfaces:**
- Preview route remains `preview.html`.
- Build artifact remains `src-tauri/target/release/bundle/macos/Mana.app`.

- [ ] **Step 1: Start one preview server**

```bash
npm run dev -- --host 127.0.0.1 --port 1421
```

- [ ] **Step 2: Inspect required states**

Inspect:

```text
preview.html?rank=godlike&prestige=10&providers=both&outputTokens=18446744073709551615
preview.html?rank=godlike&prestige=10&providers=both&outputTokens=18446744073709551615&hover=true
preview.html?rank=godlike&prestige=10&providers=claude&motion=reduced
preview.html?rank=godlike&prestige=10&providers=codex&motion=reduced
```

Verify a 456px root, glass/root equality, motif-only corners, connected rails,
no outer translucent band, characters inside aura ground, stable rest/tooltip
geometry, exact lifetime output, and frozen Reduced Motion effects.

- [ ] **Step 3: Replace the README screenshot**

Capture the verified two-provider rest state without browser chrome at its
exact widget bounds and save it as `docs/images/mana-widget.png`.

- [ ] **Step 4: Build without launching**

```bash
npm run tauri build -- --bundles app
plutil -lint src-tauri/target/release/bundle/macos/Mana.app/Contents/Info.plist
plutil -extract LSUIElement raw -o - src-tauri/target/release/bundle/macos/Mana.app/Contents/Info.plist
git diff --check
```

Expected: bundle succeeds, plist is valid, `LSUIElement=true`, and the native
application is neither installed nor launched.

- [ ] **Step 5: Commit**

```bash
git add docs/images/mana-widget.png
git commit -m "docs: show compact integrated mana frame"
```

