# Rank Armor and Prestige Art Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship rank-aware illustrated mana frames, preserve the original app
border, add a corner-mounted Rank Up control, align familiars, and redesign ten
prestige medallions.

**Architecture:** `#root[data-rank]` remains the single cosmetic selector. Each
tier supplies one transparent 2× meter foreground while the existing shared
frame is only the default for missing/unknown rank. The app perimeter uses the
exact pre-iteration baseline styling. Prestige remains file-based at the existing badge URLs;
tests validate all bitmap contracts before CSS integration can pass.

**Tech Stack:** TypeScript, Vitest, CSS, HTML, Pillow, built-in image generation, Tauri/Vite.

## Global Constraints

- Do not change XP, ranks, prestige math, provider data, sprite selection, or meter fill geometry.
- Mana overlays are `public/hud/mana-bar-frame-<tier>.png`, exactly 288×40 RGBA.
- Prestige medallions remain `public/badges/prestige-1.png` through `prestige-10.png`, exactly 96×96 RGBA.
- No floating star/diamond indicators, generic five-point stars, loose sparkles, text, or numerals in generated art.
- Rank Up / Prestige is positioned at `top: 4px; right: 4px`.
- Provider fills remain cyan/blue for Claude and magenta/pink for Codex.
- The current `public/hud/mana-bar-frame.png` remains the default when no
  known rank selects art; known ranks render exactly one foreground frame.
- Preserve reduced-motion behavior.
- Do not create commits.

---

### Task 1: Lock the bitmap and markup contracts

**Files:**
- Create: `src/rank-decoration.test.ts`
- Modify: `src/styles.test.ts`
- Modify: `src/view.test.ts`

**Interfaces:**
- Consumes: existing PNG decoder pattern from `src/sprites.test.ts`.
- Produces: failing acceptance tests for 14 meter overlays, 10 medallions, rank CSS mapping, removed indicators, armored shell, and corner action placement.

- [ ] **Step 1: Write failing bitmap tests**

Create a PNG decoder in `src/rank-decoration.test.ts` and assert:

```ts
const tiers = ["naked", "plastic", "wood", "iron", "bronze", "silver", "gold",
  "platinum", "emerald", "diamond", "master", "legend", "champion", "godlike"];

for (const tier of tiers) {
  const image = decodeRgba(new URL(`../public/hud/mana-bar-frame-${tier}.png`, import.meta.url));
  expect([image.width, image.height], tier).toEqual([288, 40]);
  expect(edgeAlpha(image), tier).toBe(0);
  expect(visiblePixels(image), tier).toBeGreaterThan(1_200);
}

for (let prestige = 1; prestige <= 10; prestige += 1) {
  const image = decodeRgba(new URL(`../public/badges/prestige-${prestige}.png`, import.meta.url));
  expect([image.width, image.height], String(prestige)).toEqual([96, 96]);
  expect(edgeAlpha(image), String(prestige)).toBe(0);
  expect(visiblePixels(image), String(prestige)).toBeGreaterThan(1_000);
}
```

- [ ] **Step 2: Write failing integration tests**

Update stylesheet and view tests to require the new behavior:

```ts
expect(styles).toContain('--meter-frame-art: url("/hud/mana-bar-frame-silver.png")');
expect(styles).toContain('--meter-frame-art: url("/hud/mana-bar-frame.png")');
expect(styles).toMatch(/\.track::after\s*\{[^}]*background-image:\s*var\(--meter-frame-art\)/s);
expect(styles).toMatch(/#action\s*\{[^}]*top:\s*4px[^}]*right:\s*4px[^}]*clip-path:/s);
expect(styles).toMatch(/#frame::before\s*\{/);
expect(styles).toMatch(/#frame::after\s*\{/);
expect(styles).not.toContain("activity-signal");
expect(styles).not.toContain("★");
expect(cardHtml(snapshot, "claude")).not.toContain("activity-signal");
```

- [ ] **Step 3: Run the focused tests and verify RED**

Run: `npx vitest run src/rank-decoration.test.ts src/styles.test.ts src/view.test.ts`

Expected: FAIL because rank meter files and armor integration do not exist, the action is still at 8px/10px, and `activity-signal` is still rendered.

---

### Task 2: Add deterministic post-processing for generated HUD art

**Files:**
- Create: `scripts/normalize_hud_art.py`
- Create: `scripts/test_normalize_hud_art.py`

**Interfaces:**
- Produces: `normalize_meter(source: Image.Image) -> Image.Image` and `normalize_badge(source: Image.Image) -> Image.Image`.
- Meter output: 288×40 with a 2px transparent edge and exact 284×36 visible
  bounds, so every generated tier shares one runtime geometry.
- Badge output: 96×96 with a 4px transparent edge and centered visible bounds.

- [ ] **Step 1: Write normalization tests**

```py
def test_meter_normalization_preserves_wide_art_inside_clear_edges():
    source = Image.new("RGBA", (900, 300), (0, 0, 0, 0))
    ImageDraw.Draw(source).rounded_rectangle((40, 80, 860, 220), 30, fill=(220, 180, 80, 255))
    result = normalize_meter(source)
    assert result.size == (288, 40)
    assert max(edge_alpha(result)) == 0

def test_badge_normalization_centers_crest_inside_clear_edges():
    source = Image.new("RGBA", (700, 700), (0, 0, 0, 0))
    ImageDraw.Draw(source).ellipse((80, 40, 620, 650), fill=(220, 180, 80, 255))
    result = normalize_badge(source)
    assert result.size == (96, 96)
    assert max(edge_alpha(result)) == 0
```

- [ ] **Step 2: Run tests and verify RED**

Run: `python3 scripts/test_normalize_hud_art.py`

Expected: FAIL with `ModuleNotFoundError: normalize_hud_art`.

- [ ] **Step 3: Implement the normalizer**

Trim alpha above 16, resize meters with Lanczos to the exact `(284, 36)`
template, resize badges with Lanczos while preserving aspect ratio inside
`(88, 88)`, center on transparent canvases, and expose CLI flags:

```text
--kind meter|badge --input SOURCE --output DESTINATION
```

- [ ] **Step 4: Run tests and verify GREEN**

Run: `python3 scripts/test_normalize_hud_art.py`

Expected: both tests PASS.

---

### Task 3: Generate the 14 rank meter overlays

**Files:**
- Create: `public/hud/mana-bar-frame-naked.png` through `public/hud/mana-bar-frame-godlike.png`

**Interfaces:**
- Consumes: `scripts/normalize_hud_art.py --kind meter` and the visual tier table in the approved spec.
- Produces: 14 transparent ornamental overlays used by CSS variables.

- [ ] **Step 1: Generate each meter on flat chroma**

Use built-in image generation with `public/hud/mana-bar-frame.png`, the matching Claude/Codex rank sheets, and this invariant prompt:

```text
Use case: precise-object-edit
Asset type: 2x fantasy game HUD mana-frame overlay
Primary request: redesign the existing silver mana frame in the named rank material.
Composition: one perfectly horizontal, centered 7:1 bar; fixed empty inner channel; symmetrical end caps.
Style: richly shaded painterly pixel art with dark navy outline and crisp 2x UI detail.
Constraints: no fill, text, numbers, stars, loose sparkles, scenery, shadows, cropping, or extra objects; keep ornament outside the channel; flat #00FF00 background.
```

Generate one image per tier so silhouettes stay controlled. Low tiers remain restrained; top tiers gain mounted gems, plate layering, winged corners, and halo geometry without becoming taller than the 40px output.

- [ ] **Step 2: Remove chroma and normalize**

For each output, run the installed chroma helper with border auto-key, soft matte, despill, then:

```bash
python3 scripts/normalize_hud_art.py --kind meter --input <alpha.png> --output public/hud/mana-bar-frame-<tier>.png
```

- [ ] **Step 3: Run the bitmap gate**

Run: `npx vitest run src/rank-decoration.test.ts`

Expected: meter assertions PASS; prestige assertions remain RED until Task 4.

---

### Task 4: Generate the 10 prestige medallions

**Files:**
- Replace: `public/badges/prestige-1.png` through `public/badges/prestige-10.png`

**Interfaces:**
- Consumes: `scripts/normalize_hud_art.py --kind badge`.
- Produces: ten 96×96 transparent medallions used by the existing badge DOM.

- [ ] **Step 1: Generate the medallions**

Use built-in image generation, grounded on the Godlike sprite finish and this invariant prompt:

```text
Use case: stylized-concept
Asset type: 2x prestige medallion for a fantasy game HUD
Composition: one centered circular crest, front view, strong readable silhouette at 24px.
Style: richly shaded painterly pixel art, dark navy outline, physical metal and mounted gems.
Constraints: no text, numbers, generic five-point stars, loose sparkles, scenery, cast shadow, cropping, or extra icons; flat #00FF00 background.
```

Follow the approved sequence from forged silver seal through white-gold cosmic crown. Each icon must be distinct while sharing the same circular crest family.

- [ ] **Step 2: Remove chroma and normalize**

Run the chroma helper, then:

```bash
python3 scripts/normalize_hud_art.py --kind badge --input <alpha.png> --output public/badges/prestige-<n>.png
```

- [ ] **Step 3: Run the bitmap gate and verify GREEN**

Run: `npx vitest run src/rank-decoration.test.ts`

Expected: all 24 bitmap assets PASS size, edge, and coverage checks.

---

### Task 5: Wire rank frames, preserve the old shell, action, and XP sparkle

**Files:**
- Modify: `src/styles.css`
- Modify: `src/view.ts`
- Modify: `src/styles.test.ts`
- Modify: `src/view.test.ts`

**Interfaces:**
- Consumes: `#root[data-rank]` and the 14 meter overlay paths.
- Produces: `--meter-frame-art`, the unchanged original app perimeter, no activity
  indicator DOM, a true-corner action tab, aligned content spacing, and XP
  gem-glints.

- [ ] **Step 1: Map every rank to meter and armor tokens**

Each rank rule defines its file and material tokens:

```css
#root[data-rank="silver"] {
  --meter-frame-art: url("/hud/mana-bar-frame-silver.png");
  --armor-outer: #eef5ff;
  --armor-mid: #9ba9ba;
  --armor-dark: #303845;
  --armor-accent: #aee8ff;
  --armor-w: 2px;
  --armor-corner: 16px;
}
```

Repeat explicitly for all 14 tiers with escalating width, corner size, glow, and tier material colors.

- [ ] **Step 2: Render one selected frame above the live fill**

```css
:root {
  --meter-frame-art: url("/hud/mana-bar-frame.png");
}

.track {
  background: none;
}

.track::after {
  position: absolute;
  z-index: 2;
  inset: 0;
  background-image: var(--meter-frame-art);
  background-position: center;
  background-size: 100% 100%;
  background-repeat: no-repeat;
  content: "";
  pointer-events: none;
}
```

- [ ] **Step 3: Restore the pre-iteration application perimeter**

Use `git show HEAD:src/styles.css` as the source of truth for the shell-only
rules: `#root`, `#root::before`, `#root::after`, rank frame tokens, `#frame`,
Champion/Godlike shell animations, and their reduced-motion handling. Remove
all new armor tokens and experimental corner-guard layers. Keep the new meter,
action, XP, familiar, and prestige integration around those restored rules.

- [ ] **Step 4: Remove activity diamonds and preserve working feedback**

Remove `<span class="activity-signal">` from `cardHtml`, remove its CSS, simplify `.head` to three columns, and add the working glow to the familiar:

```css
.provider-card[data-working] .sprite {
  filter: drop-shadow(0 2px 1px rgba(0, 0, 0, 0.42)) drop-shadow(0 0 9px var(--glow));
}
```

- [ ] **Step 5: Move and reshape the action control**

Set `top: 4px; right: 4px`, inherit rank material variables, remove the pill radius, and use an angular `clip-path` with an inset bevel.

- [ ] **Step 6: Replace the star fallback**

Keep missing-badge resilience, but render a circular metal crest with gradients and borders. Remove the `★` glyph entirely.

- [ ] **Step 7: Add the XP sparkle without star icons**

Use `.xpfill::before` for a narrow traveling sheen and `.xpfill::after` for
two circular radial-gradient gem glints. Both remain clipped inside the XP
fill and `prefers-reduced-motion` disables their animations.

- [ ] **Step 8: Keep familiar ornament visible**

Set `overflow: visible` explicitly on `.familiar-slot`, `.sprite`, and provider
sections. This is defensive framing; the Codex Champion authored silhouette
is corrected separately in Task 6.

Use smooth `image-rendering: auto` for the 2x atlas at non-integer Retina scale.
Move colored illumination off the rectangular `.sprite` filter and onto a
larger blurred circular `.familiar-slot::before` radial layer; working state
intensifies that radial light. Keep only a subtle neutral grounding shadow on
the sprite itself.

- [ ] **Step 9: Run focused tests and verify GREEN**

Run: `npx vitest run src/styles.test.ts src/view.test.ts src/rank-decoration.test.ts`

Expected: all focused tests PASS.

---

### Task 6: Correct and optically align the Codex Champion familiar

**Files:**
- Replace: `public/sprites/codex-rank-champion.png`

**Interfaces:**
- Edit target: the existing Codex Champion 4×3 rank atlas.
- Reference: `public/sprites/codex-base.png` for face and body identity.
- Produces: the same 448×336 RGBA atlas with a complete pointed staff
  sunburst and comfortable transparent breathing room in every cell.

- [ ] **Step 1: Reproduce and isolate the authored-art cause**

Render the first idle cell at application scale and inspect its alpha bounds.
Confirm that the cell already has transparent margin but the staff-head
sunburst itself has a flat, cut-looking top edge.

- [ ] **Step 2: Edit the atlas while preserving its approved identity**

Use the built-in image generator to preserve the blue-and-gold Champion armor,
laurel, face, staff, pose consistency, row meanings, and flat key background.
Change only the framing and staff ornament so the complete pointed sunburst is
visible with breathing room in all 12 cells.

- [ ] **Step 3: Normalize and verify the corrected atlas**

Remove the flat key background, normalize with `scripts/normalize_rank_atlas.py`,
then run the sprite contract and rank-art quality checks. Re-render the first
idle frame at application scale and confirm the staff no longer reads as clipped.

- [ ] **Step 4: Match Champion optical scale and baseline**

Normalize the Codex Champion idle and hover rows so their median visible height
and bottom baseline match Claude Champion within four source pixels. Preserve
the 112px cell contract, at least 4px of transparent safety margin, and the
working row whose baseline already matches.

---

### Task 7: Visual QA and full verification

**Files:**
- No production changes unless a visual defect is reproduced and covered by a failing test.

**Interfaces:**
- Produces: contact sheets and real-scale application screenshots for review.

- [ ] **Step 1: Create QA contact sheets**

Render all 14 meter frames at both source size and actual 144×20 display size,
plus all 10 badges at 96×96 and actual 24×24 display size. Inspect for channel
intrusion, chroma residue, rank-order regression, indistinct low tiers,
unreadable prestige silhouettes, and the Codex Champion staff silhouette.

- [ ] **Step 2: Run the application and inspect representative ranks**

Capture Naked, Silver, Emerald, Master, Champion, and Godlike. Verify armor
follows the rounded window, the action control sits 4px from the top/right, no
diamonds remain, fills remain aligned, shell ornament does not cover content,
the Champion familiar is fully legible, and the XP fill shows gem-like sparkle.

- [ ] **Step 3: Run fresh automated verification**

Run:

```bash
python3 scripts/test_normalize_hud_art.py
npm test
npm run build
```

Expected: Python tests PASS, all Vitest files PASS with zero failures, and Vite production build exits 0.

- [ ] **Step 4: Inspect final repository state**

Run: `git status --short` and `git diff --stat`.

Expected: only the approved design/plan, generated HUD/badge assets, integration code/tests, the previously uncommitted Codex Godlike correction, and no generation intermediates. No commits are created.
