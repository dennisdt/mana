# Mana Frame Safe Area And Structural Corners Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enlarge Mana just enough to preserve its current content scale inside a real frame-safe area and replace repeated corner crests with the generated directional corner joints.

**Architecture:** Keep the existing absolute perimeter and dynamic native-height pipeline. Expand the authored width and card insets while preserving the current 424px provider-content width, then simplify corner composition so rank corners render their directional assets and active prestige renders its directional joint surface as the only corner face.

**Tech Stack:** TypeScript 5.6, vanilla CSS, Vitest 3, Vite 6, Tauri 2, Rust.

## Global Constraints

- The authored roster width is exactly 488px.
- The usable provider-content width remains 424px.
- `#card` padding is exactly `24px 32px 20px`.
- Frame-aware footer padding is exactly `10px 36px 18px`.
- Rank and prestige corner boxes remain 32px by 32px.
- Existing 96px corner sources render at 48px by 48px and align toward their matching outer corners.
- Corner backgrounds are transparent and use `filter: drop-shadow(0 0 4px var(--cap-glow))`.
- Active prestige corner joints fully replace rank corner faces.
- Do not resize sprites, auras, meters, typography, or provider columns.
- Do not alter progression, persistence, activity detection, generated frame assets, or external frame bleed.

---

## File Map

- `src/window-layout.ts`: authored native width and dynamic sizing contract.
- `src-tauri/tauri.conf.json`: first-paint native window dimensions.
- `src/window-layout.test.ts`: native geometry regression coverage.
- `src/styles.css`: roster safe area and structural corner presentation.
- `src/styles.test.ts`: stylesheet geometry and corner-composition regression coverage.
- `src/frame-renderer.ts`: maps resolved frame assets into perimeter CSS variables and datasets.
- `src/frame-renderer.test.ts`: proves directional corner selection and removal of repeated rank crests.

### Task 0: Restore A Green Test Baseline For The Pulled Frame Pipeline

**Files:**
- Modify: `src/frame-assets.test.ts`
- Modify: `src/frame-renderer.test.ts`
- Modify: `src/generated-assets.test.ts`
- Modify: `src/preview.test.ts`
- Modify: `src/progress-view.test.ts`
- Modify: `src/rank-decoration.test.ts`
- Modify: `src/styles.test.ts`
- Modify: `src/window-layout.test.ts`

**Interfaces:**
- Consumes: production behavior already present at pulled commit `9211719`.
- Produces: regression expectations for the current zero-bleed frame,
  directional prestige joints, preview tooltip options, output-carrying
  prestige copy, and compact aura registration.
- Does not modify production source or generated assets.

- [ ] **Step 1: Record the failing baseline**

Run:

```bash
npm test
```

Expected: 21 failures across the eight listed test files. Preserve this output
as evidence that these failures predate the safe-area implementation.

- [ ] **Step 2: Reconcile frame registry and renderer expectations**

In `src/frame-assets.test.ts`:

- Expect a resolved prestige model to preserve
  `model.rank?.crestTop === "/frames/ranks/godlike/crest-top.png"`.
- Include `corner-joint-tl`, `corner-joint-tr`, `corner-joint-bl`, and
  `corner-joint-br` in the expected prestige probe sequence.

In `src/frame-renderer.test.ts`:

- Add all four `cornerSurfaces` paths to the prestige fixture.
- Remove the obsolete `data-prestige-text` node expectation and fake child.
- Expect empty prestige ornament lanes to replace rank lanes with zero counts.
- Expect `--frame-crest` to retain the rank crest while
  `--progress-prestige-crest` receives the prestige crest.
- Expect `data-corner-surface="true"` on the applied perimeter.
- Expect empty ornament-lane HTML when the active prestige kit has no
  ornaments.

- [ ] **Step 3: Reconcile generated-asset contracts**

In `src/generated-assets.test.ts`, define the prestige contract as:

```ts
const prestigePieces = [
  ...BASE_PIECES,
  "crest-top",
  "corner-joint-tl",
  "corner-joint-tr",
  "corner-joint-bl",
  "corner-joint-br",
] as const;
```

Rename the test to say "eleven-piece contract".

Extend `assertKit` with a `connectedCorners = false` parameter. For a
prestige kit, pass `true` and require every `corner-*` piece to have nonzero
edge alpha instead of applying the detached-art `edgeAlpha(image) === 0`
assertion:

```ts
} else if (connectedCorners && piece.startsWith("corner-")) {
  expect(edgeAlpha(image), piece).toBeGreaterThan(0);
} else {
  expect(edgeAlpha(image), piece).toBe(0);
}
```

In `src/rank-decoration.test.ts`, add the four directional corner-joint files
at 96px by 96px. Keep seam assertions for rails, but treat both `corner-*` and
`corner-joint-*` pieces as connected sockets whose edge alpha may be nonzero.
Continue requiring visible pixels and exact dimensions for every piece.

- [ ] **Step 4: Reconcile preview and progression expectations**

In each exact-object expectation in `src/preview.test.ts`, add:

```ts
hovering: false,
outputTokens: "12345678",
```

Keep existing partial-object assertions unchanged.

In `src/progress-view.test.ts`, expect the current output-preserving prestige
copy:

```ts
"The curve steepens. Surplus output carries forward into Prestige 2."
```

- [ ] **Step 5: Reconcile compact glass, frame, aura, and sizing expectations**

In `src/styles.test.ts`:

- Expect `#glass` to use `inset: 0`, `width: 100%`, and `height: 100%`.
- Expect 32px corner boxes and 48px corner background art.
- Expect rails to extend to 16px from each neighboring edge.
- Remove the deleted prestige text plaque test.
- Expect the aura anchor to be `top: calc(50% - 12px)`.
- Keep assertions for transparent root, fixed rail art dimensions, ornaments,
  animation scope, Reduced Motion, and provider overflow.

In `src/window-layout.test.ts`, first align the pulled zero-bleed 456px
baseline:

```ts
expect(FRAME_BLEED).toBe(0);
expect(WINDOW_WIDTH).toBe(456);
expect(scaledRosterSize(207.2, 1)).toEqual({ width: 456, height: 210 });
```

Expect the current right-edge clamp to be `x: 1968`.

- [ ] **Step 6: Verify the reconciled baseline**

Run:

```bash
npm test
```

Expected: all current tests pass before any production change for this feature.

- [ ] **Step 7: Commit the baseline reconciliation**

```bash
git add src/frame-assets.test.ts src/frame-renderer.test.ts src/generated-assets.test.ts src/preview.test.ts src/progress-view.test.ts src/rank-decoration.test.ts src/styles.test.ts src/window-layout.test.ts
git commit -m "test: align generated frame regression suite"
```

### Task 1: Expand The Native Safe Area Without Shrinking Content

**Files:**
- Modify: `src/window-layout.test.ts`
- Modify: `src/styles.test.ts`
- Modify: `src/window-layout.ts`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src/styles.css`

**Interfaces:**
- Produces: `ROSTER_WIDTH = 488`, `WINDOW_WIDTH = 488`, and `scaledRosterSize(height, scale)` returning that width.
- Produces: frame-aware layout with 424px of usable provider content.
- Consumes: existing `rosterHeight()` and zero `FRAME_BLEED`.

- [ ] **Step 1: Replace stale geometry expectations with failing 488px tests**

In `src/window-layout.test.ts`, update the first geometry test and fixed-zoom
expectations to assert the actual zero-bleed model plus the new width:

```ts
expect({ width: ROSTER_WIDTH, height: INITIAL_ROSTER_HEIGHT }).toEqual({
  width: 488,
  height: 175,
});
expect(FRAME_BLEED).toBe(0);
expect(WINDOW_WIDTH).toBe(488);
expect(mainWindow).toMatchObject({
  width: 488,
  height: 175,
});
expect(scaledRosterSize(207.2, 1)).toEqual({ width: 488, height: 210 });
expect(scaledRosterSize(207.2, WIDGET_ZOOM)).toEqual({
  width: 488,
  height: 210,
});
```

Keep the existing work-area tests, but change their compact right-edge
expectation from `x: 1872` to `x: 1904`, because
`2880 - (488 * 2) = 1904`.

- [ ] **Step 2: Add failing stylesheet safe-area tests**

Add this test to the main stylesheet describe block in `src/styles.test.ts`:

```ts
it("keeps provider content and the footer inside the generated frame safe area", () => {
  expect(styles).toMatch(
    /#card\s*\{[^}]*padding:\s*24px 32px 20px/s,
  );
  expect(styles).toMatch(
    /#root\[data-frame-art="true"\] #progress\s*\{[^}]*padding:\s*10px 36px 18px/s,
  );
  expect(styles).toMatch(
    /#card section\s*\{[^}]*grid-template-columns:\s*70px minmax\(0, 1fr\)/s,
  );
  expect(styles).toContain("--meter-width: 144px");
});
```

- [ ] **Step 3: Run focused tests and verify RED**

Run:

```bash
npm test -- src/window-layout.test.ts src/styles.test.ts
```

Expected: FAIL because the authored width is still 456px, first-paint config
is 504px, and card/footer padding still uses the compact pre-frame values.

- [ ] **Step 4: Implement the new width and padding**

In `src/window-layout.ts`:

```ts
export const ROSTER_WIDTH = 488;
export const INITIAL_ROSTER_HEIGHT = 175;
export const FRAME_BLEED = 0;
export const WINDOW_WIDTH = ROSTER_WIDTH;
```

In the `main` window entry in `src-tauri/tauri.conf.json`:

```json
"width": 488,
"height": 175
```

In `src/styles.css`:

```css
#card {
  padding: 24px 32px 20px;
}

#root[data-frame-art="true"] #progress {
  padding: 10px 36px 18px;
}
```

Do not change section columns, gaps, sprite dimensions, meter dimensions, or
the base non-frame footer rule.

- [ ] **Step 5: Run focused tests and verify GREEN**

Run:

```bash
npm test -- src/window-layout.test.ts src/styles.test.ts
```

Expected: PASS.

- [ ] **Step 6: Commit the safe-area geometry**

```bash
git add src/window-layout.test.ts src/styles.test.ts src/window-layout.ts src-tauri/tauri.conf.json src/styles.css
git commit -m "fix: add generated frame safe area"
```

### Task 2: Render Directional Structural Corners

**Files:**
- Modify: `src/frame-renderer.test.ts`
- Modify: `src/styles.test.ts`
- Modify: `src/frame-renderer.ts`
- Modify: `src/styles.css`

**Interfaces:**
- Consumes: `ResolvedFrameDecoration.rank.corners`,
  `ResolvedFrameDecoration.prestige.cornerSurfaces`, and the existing
  all-or-nothing frame resolver.
- Produces: `--frame-rank-corner-{tl,tr,bl,br}` and
  `--frame-corner-surface-{tl,tr,bl,br}` as the only corner-face sources.
- Preserves: `data-corner-surface` for active prestige selection.
- Removes: `--frame-corner-emblem`, `hasCornerEmblem`, and
  `data-corner-emblem`.

- [ ] **Step 1: Add failing renderer tests for directional rank corners**

In `src/frame-renderer.test.ts`, add:

```ts
it("uses matching directional rank corners without repeating the rank crest", () => {
  const plan = frameRenderPlan(
    decoration({ prestige: null, prestigeText: "" }),
  );

  expect(plan.cssVariables).toMatchObject({
    "--frame-rank-corner-tl": 'url("/rank/corner-tl.png")',
    "--frame-rank-corner-tr": 'url("/rank/corner-tr.png")',
    "--frame-rank-corner-bl": 'url("/rank/corner-bl.png")',
    "--frame-rank-corner-br": 'url("/rank/corner-br.png")',
  });
  expect(plan.cssVariables).not.toHaveProperty("--frame-corner-emblem");
  expect(plan).not.toHaveProperty("hasCornerEmblem");
});
```

Extend the stable-node test:

```ts
expect(perimeter.dataset).not.toHaveProperty("cornerEmblem");
```

- [ ] **Step 2: Add failing structural-corner stylesheet tests**

Add a generated-perimeter test in `src/styles.test.ts`:

```ts
it("renders transparent directional joints instead of square crest caps", () => {
  const corner = styles.match(/\.frame-corner\s*\{([^}]*)\}/s)?.[1] ?? "";

  expect(corner).toMatch(/width:\s*32px/);
  expect(corner).toMatch(/height:\s*32px/);
  expect(corner).toMatch(/background-color:\s*transparent/);
  expect(corner).toMatch(
    /background-image:\s*var\(--frame-rank-corner-piece,\s*none\)/,
  );
  expect(corner).toMatch(/background-size:\s*48px 48px/);
  expect(corner).toMatch(
    /filter:\s*drop-shadow\(0 0 4px var\(--cap-glow\)\)/,
  );
  expect(corner).not.toMatch(/\bbox-shadow\s*:/);
  expect(styles).not.toContain("--frame-corner-emblem");
  expect(styles).not.toContain('data-corner-emblem="true"');

  for (const [cornerName, position] of [
    ["tl", "left top"],
    ["tr", "right top"],
    ["bl", "left bottom"],
    ["br", "right bottom"],
  ] as const) {
    expect(styles).toMatch(
      new RegExp(
        `\\.frame-corner--${cornerName}\\s*\\{[^}]*` +
          `--frame-rank-corner-piece:\\s*var\\(--frame-rank-corner-${cornerName}\\)` +
          `[^}]*--frame-prestige-corner-piece:\\s*var\\(--frame-corner-surface-${cornerName}\\)` +
          `[^}]*background-position:\\s*${position}`,
        "s",
      ),
    );
  }

  expect(styles).toMatch(
    /#perimeter\[data-corner-surface="true"\] \.frame-corner\s*\{[^}]*background-image:\s*var\(--frame-prestige-corner-piece,\s*none\)/s,
  );
});
```

- [ ] **Step 3: Run focused tests and verify RED**

Run:

```bash
npm test -- src/frame-renderer.test.ts src/styles.test.ts
```

Expected: FAIL because the renderer still publishes and applies the repeated
crest emblem, while CSS still paints opaque cap surfaces and rectangular
shadows.

- [ ] **Step 4: Remove the repeated corner-emblem contract**

In `src/frame-renderer.ts`:

- Delete `hasCornerEmblem` from `FrameRenderPlan`.
- Delete the `cornerEmblem` local.
- Delete the `--frame-corner-emblem` CSS variable.
- Delete `hasCornerEmblem` from the returned plan.
- Delete `perimeter.dataset.cornerEmblem`.
- Keep `hasCornerSurface`, all directional corner URL variables, prestige
  crest propagation, ornament lanes, and revision ordering unchanged.

- [ ] **Step 5: Replace cap composition with directional alpha artwork**

In `src/styles.css`, reduce the base corner rule to:

```css
.frame-corner {
  z-index: 3;
  width: 32px;
  height: 32px;
  overflow: hidden;
  border: 0;
  background-color: transparent;
  background-image: var(--frame-rank-corner-piece, none);
  background-repeat: no-repeat;
  background-size: 48px 48px;
  filter: drop-shadow(0 0 4px var(--cap-glow));
  image-rendering: pixelated;
}

#perimeter[data-corner-surface="true"] .frame-corner {
  background-image: var(--frame-prestige-corner-piece, none);
}
```

For each directional corner, keep its outer anchor and radius but replace cap,
rail-collar, and border-width variables with:

```css
.frame-corner--tl {
  --frame-rank-corner-piece: var(--frame-rank-corner-tl);
  --frame-prestige-corner-piece: var(--frame-corner-surface-tl);
  top: 0;
  left: 0;
  border-radius: var(--hud-radius) 0 0;
  background-position: left top;
}
```

Repeat with matching `tr`, `bl`, and `br` variables, anchors, radii, and outer
positions. Remove:

- the old mask-only prestige corner rule,
- cap rail-collar background layers,
- opaque cap surface layers,
- base and Prestige X corner `box-shadow`,
- `.frame-corner::before`,
- the `data-corner-emblem` selector, and
- the now-obsolete prestige rule that clears `--cap-rank-h` and
  `--cap-rank-v`.

Keep rails below corners, ornament lanes, Prestige VII-X `::after` animation,
and Reduced Motion coverage unchanged.

- [ ] **Step 6: Run focused tests and verify GREEN**

Run:

```bash
npm test -- src/frame-renderer.test.ts src/styles.test.ts
```

Expected: PASS.

- [ ] **Step 7: Run the full frontend suite**

Run:

```bash
npm test
```

Expected: every Vitest file passes with zero failures.

- [ ] **Step 8: Commit structural corner rendering**

```bash
git add src/frame-renderer.test.ts src/styles.test.ts src/frame-renderer.ts src/styles.css
git commit -m "fix: render structural frame corners"
```

### Task 3: Production And Visual Verification

**Files:**
- Verify only; no planned source changes.

**Interfaces:**
- Consumes: `preview.html` query parameters and the existing native bundle
  path.
- Produces: a verified, signed local
  `src-tauri/target/release/bundle/macos/Mana.app`.

- [ ] **Step 1: Run complete automated verification**

Run:

```bash
npm test
CARGO_PROFILE_TEST_DEBUG=0 /Users/brianlee/.cargo/bin/cargo test --manifest-path src-tauri/Cargo.toml
npm run build
git diff --check
```

Expected: all frontend and Rust tests pass, the production frontend builds,
and whitespace validation is clean.

- [ ] **Step 2: Start the deterministic preview**

Run:

```bash
npm run dev -- --host 127.0.0.1 --port 1421
```

Inspect these exact routes:

```text
preview.html?rank=platinum&prestige=0&providers=both
preview.html?rank=godlike&prestige=0&providers=both
preview.html?rank=godlike&prestige=1&providers=both
preview.html?rank=godlike&prestige=10&providers=both
preview.html?rank=platinum&prestige=0&providers=claude&motion=reduced
preview.html?rank=platinum&prestige=0&providers=codex&motion=reduced
```

- [ ] **Step 3: Verify computed geometry and pixels**

For each representative route, confirm:

- root width is 488px,
- card left and right padding are 32px,
- provider-content width remains 424px,
- reset times and sprite/aura overflow remain inside the frame safe area,
- footer padding is `10px 36px 18px`,
- rank corners use four distinct directional pieces,
- prestige joints fully replace rank corners,
- corner backgrounds are transparent,
- no square shadows or repeated rank crests remain,
- rails meet beneath every corner, and
- Reduced Motion freezes existing animation without changing geometry.

- [ ] **Step 4: Build and sign the native bundle**

Run:

```bash
PATH="/Users/brianlee/.cargo/bin:${PATH}" npm run tauri build
codesign --force --deep --sign - --timestamp=none src-tauri/target/release/bundle/macos/Mana.app
codesign --verify --deep --strict --verbose=2 src-tauri/target/release/bundle/macos/Mana.app
```

Expected: release build succeeds and `Mana.app` is valid on disk.

- [ ] **Step 5: Relaunch the exact release bundle**

Stop only the exact old executable path if it is running, then open:

```text
/Users/brianlee/mana/src-tauri/target/release/bundle/macos/Mana.app
```

Verify the new process path and inspect the live window.

- [ ] **Step 6: Confirm repository state**

Run:

```bash
git status --short --branch
git log -3 --oneline --decorate
```

Expected: no uncommitted source changes and the safe-area and structural-corner
commits are present after the design and plan commits.
