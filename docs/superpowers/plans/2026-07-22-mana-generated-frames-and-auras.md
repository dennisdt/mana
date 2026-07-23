# Mana Generated Frames and Auras Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace Mana's CSS application border with scalable generated rank/prestige frame pieces and add grounded, provider-specific animated elemental auras.

**Architecture:** The webview reserves a fixed transparent bleed around the existing 456px glass content area. A frame registry resolves generated rail, corner, ornament, and crest assets into one perimeter renderer; a separate aura registry and deadline scheduler animate effect atlases behind existing character sprites.

**Tech Stack:** TypeScript 5.6, vanilla HTML/CSS, Vitest, Vite 6, Pillow, built-in image generation, Tauri v2.

## Global Constraints

- Complete `2026-07-22-mana-progress-durability.md` before running the updated app against real user data.
- Keep the glass content width at 456 CSS pixels.
- Reserve `FRAME_BLEED = 24` CSS pixels on every side; the fixed window width becomes 504 CSS pixels.
- Generated frames surround the glass container; they are not nested content panels.
- Never stretch a complete frame or a corner, crest, ornament, or starburst.
- Rail art stays stationary. Only a masked light pass may move over it.
- No dotted/dashed fallback borders, side medallions, bottom crest, duplicated ornament lane, or stacked prestige decorations.
- Show exactly one highest-earned prestige crest and render I-X as interface text; Prestige 12, for example, displays `P12` while reusing Prestige X art.
- Claude is fire/poison; Codex is ice/lightning. Their high-tier loops total exactly 3200ms and 3650ms respectively.
- Aura and character element bounds remain fixed while frames advance.
- Explicitly unauthenticated providers create neither sprite nor aura DOM.
- Preserve existing mana-bar geometry, progress math, rank sprite selection, and reduced-motion behavior.
- All production assets must live under `public/`; nothing may load from `.superpowers/` or `$CODEX_HOME`.
- Use TDD and commit after each independently passing task.

## Production Asset Geometry

All bitmap sizes below are source pixels and display at exactly 50%:

| Piece | Source | CSS display |
|---|---:|---:|
| horizontal rail tile | 128x32 | 64x16 |
| vertical rail tile | 32x128 | 16x64 |
| directional corner/starburst | 96x96 | 48x48 |
| horizontal ornament | 64x32 | 32x16 |
| vertical ornament | 32x64 | 16x32 |
| rank top crest | 192x96 | 96x48 |
| prestige top crest | 192x96 | 96x48 |
| aura frame cell | 192x192 | 96x96 |

---

### Task 1: Add deterministic frame-kit normalization and bitmap gates

**Files:**
- Create: `scripts/normalize_frame_art.py`
- Create: `scripts/test_normalize_frame_art.py`

**Interfaces:**
- Produces:

```python
PIECE_SPECS = {
    "corner-tl": (96, 96), "rail-h": (128, 32), "corner-tr": (96, 96),
    "rail-v": (32, 128), "crest-top": (192, 96), "ornament-h": (64, 32),
    "corner-bl": (96, 96), "ornament-v": (32, 64), "corner-br": (96, 96),
}

def split_frame_kit(source: Image.Image) -> dict[str, Image.Image];
def normalize_piece(source: Image.Image, size: tuple[int, int]) -> Image.Image;
```

- [ ] **Step 1: Write failing normalizer tests**

Create a synthetic 3x3 RGBA kit with differently colored shapes in the fixed cell order and assert:

```python
def test_split_frame_kit_uses_fixed_nine_cell_contract():
    pieces = split_frame_kit(make_fixture_kit())
    assert tuple(pieces) == (
        "corner-tl", "rail-h", "corner-tr",
        "rail-v", "crest-top", "ornament-h",
        "corner-bl", "ornament-v", "corner-br",
    )

def test_normalized_pieces_match_source_contracts_and_clear_edges():
    pieces = split_frame_kit(make_fixture_kit())
    for name, size in PIECE_SPECS.items():
        result = normalize_piece(pieces[name], size)
        assert result.size == size
        assert max(edge_alpha(result)) == 0
```

- [ ] **Step 2: Run and verify RED**

Run: `python3 -m unittest scripts/test_normalize_frame_art.py -v`

Expected: FAIL because `normalize_frame_art` does not exist.

- [ ] **Step 3: Implement the frame-kit CLI**

The CLI accepts:

```text
--family rank|prestige --name NAME --input SOURCE --output-root public/frames
```

Require the input width and height to be divisible by 3. Split row-major, remove alpha below 16, trim each cell, preserve aspect ratio with nearest-neighbor resampling, center in its target canvas, and keep a 4-source-pixel transparent edge. Write all non-empty rank pieces to the requested directory under `public/frames/ranks/` and all prestige pieces to the requested numeric directory under `public/frames/prestige/`.

- [ ] **Step 4: Run and verify GREEN**

Run: `python3 -m unittest scripts/test_normalize_frame_art.py -v`

Expected: all normalizer tests PASS.

- [ ] **Step 5: Commit the tooling**

```bash
git add scripts/normalize_frame_art.py scripts/test_normalize_frame_art.py
git commit -m "feat: normalize generated frame kits"
```

---

### Task 2: Generate and normalize all rank and prestige frame kits

**Files:**
- Create: `src/generated-assets.test.ts`
- Create: rank piece PNGs under `public/frames/ranks/` for every tier from `plastic` through `godlike`
- Create: prestige piece PNGs under `public/frames/prestige/1/` through `public/frames/prestige/10/`

**Interfaces:**
- Consumes: `scripts/normalize_frame_art.py` and the approved tier hierarchy.
- Produces: production bitmap pieces satisfying `src/generated-assets.test.ts`.

Rank optional assets:

```ts
const RANK_EXTRAS = {
  gold: ["crest-top"], platinum: ["crest-top"],
  emerald: ["crest-top", "ornament-h", "ornament-v"],
  diamond: ["crest-top", "ornament-h", "ornament-v"],
  master: ["crest-top", "ornament-h", "ornament-v"],
  legend: ["crest-top", "ornament-h", "ornament-v"],
  champion: ["crest-top", "ornament-h", "ornament-v"],
  godlike: ["crest-top", "ornament-h", "ornament-v"],
} as const;
```

- [ ] **Step 1: Write the failing production asset gate**

In `src/generated-assets.test.ts`, reuse the PNG decoder pattern from
`rank-decoration.test.ts`. Require the six base pieces for every non-naked
rank, the optional assets declared by `RANK_EXTRAS`, and seven prestige pieces
for every level 1-10. Assert exact dimensions, RGBA color type, zero edge alpha,
and non-empty visible coverage.

- [ ] **Step 2: Run and verify RED**

Run: `npx vitest run src/generated-assets.test.ts`

Expected: FAIL because production frame assets do not exist.

- [ ] **Step 3: Generate one 3x3 kit per rank**

Use the built-in image generator once per non-naked rank with this exact layout and the material table from the approved spec:

```text
Use case: stylized-concept
Asset type: modular 2x fantasy MMORPG application-frame sprite kit
Primary request: create one coherent custom pixel-art frame kit in the named rank material.
Composition: exact 3-column by 3-row grid of equal square cells. Row 1 is top-left corner, quiet seamless horizontal rail tile, top-right corner. Row 2 is quiet seamless vertical rail tile, top-center crest, horizontal ornament. Row 3 is bottom-left corner, vertical ornament, bottom-right corner. Each piece is isolated, centered, fully visible, and uses consistent lighting.
Style: polished early-2000s Korean fantasy MMORPG UI pixel art, crisp chunky clusters, dark navy outline, hand-painted highlights, front orthographic UI view.
Constraints: no complete frame, character, text, numbers, scenery, cast shadow, checkerboard, dotted border, repeated ornament inside rail cells, or content crossing cell boundaries. Low ranks keep crest/ornament cells empty; elite ranks use them. Flat solid #ff00ff background with no #ff00ff in the art.
```

Use these materials in order: plastic molded pale gray, carved wood, riveted iron, warm bronze, polished silver, gilded gold, pale platinum, dark metal with emerald gems, prismatic diamond, crimson master runes, royal-purple legend runes, gold-blue champion regalia, white-gold celestial godlike armor.

- [ ] **Step 4: Generate one 3x3 kit per prestige level**

Use the same cell contract with this prompt:

```text
Use case: stylized-concept
Asset type: modular 2x prestige overlay sprite kit for a fantasy MMORPG frame
Primary request: create the named Prestige level overlay as one coherent pixel-art kit.
Composition: exact 3-column by 3-row grid. Row 1 is top-left starburst, horizontal rail inlay, top-right starburst. Row 2 is vertical rail inlay, one centered prestige crest, one small horizontal inlay ornament. Row 3 is bottom-left starburst, one small vertical inlay ornament, bottom-right starburst.
Style: polished early-2000s Korean fantasy MMORPG pixel art, mounted diamonds, white-gold metal, dark navy outline, consistent front lighting.
Constraints: no complete frame, side medallions, bottom crest, wings detached from the crest, text, numeral, scenery, cast shadow, duplicate rail fragments, or content crossing cells. Flat #ff00ff background.
```

Progress I-III from compact gem to faint inlay, IV-VI to broader shoulders and two-tone inlay, VII to purple rail channels, VIII to cyan facets, IX to alternating gemstone inlay, and X to the celestial twin-channel Ascendant crest.

- [ ] **Step 5: Remove chroma and normalize each kit**

For every generated source, run the installed chroma helper, then:

```bash
EMERALD_SOURCE="tmp/generated/emerald-frame-kit-source.png"
EMERALD_ALPHA="tmp/generated/emerald-frame-kit-alpha.png"
PRESTIGE_X_ALPHA="tmp/generated/prestige-10-frame-kit-alpha.png"
python "${CODEX_HOME:-$HOME/.codex}/skills/.system/imagegen/scripts/remove_chroma_key.py" \
  --input "$EMERALD_SOURCE" --out "$EMERALD_ALPHA" --auto-key border \
  --soft-matte --transparent-threshold 12 --opaque-threshold 220 --despill
python3 scripts/normalize_frame_art.py --family rank --name emerald --input "$EMERALD_ALPHA" --output-root public/frames
python3 scripts/normalize_frame_art.py --family prestige --name 10 --input "$PRESTIGE_X_ALPHA" --output-root public/frames
```

Repeat for all 13 illustrated ranks and all ten prestige levels. Empty optional cells must not create files.

- [ ] **Step 6: Inspect the asset contact sheet**

Generate a local contact sheet from the normalized outputs:

```bash
python3 - <<'PY'
from pathlib import Path
from PIL import Image, ImageDraw

files = sorted(Path("public/frames").glob("**/*.png"))
thumbs = []
for path in files:
    image = Image.open(path).convert("RGBA")
    image.thumbnail((160, 96), Image.Resampling.NEAREST)
    tile = Image.new("RGBA", (192, 128), (10, 12, 22, 255))
    tile.alpha_composite(image, ((192 - image.width) // 2, 8))
    ImageDraw.Draw(tile).text((6, 108), str(path.relative_to("public/frames")), fill="white")
    thumbs.append(tile)

columns = 5
rows = (len(thumbs) + columns - 1) // columns
sheet = Image.new("RGBA", (columns * 192, rows * 128), (5, 6, 12, 255))
for index, tile in enumerate(thumbs):
    sheet.alpha_composite(tile, ((index % columns) * 192, (index // columns) * 128))
Path("tmp").mkdir(exist_ok=True)
sheet.save("tmp/mana-frame-contact-sheet.png")
PY
```

Inspect `tmp/mana-frame-contact-sheet.png` with the image viewer. Reject any kit with inconsistent lighting, cropped corners, rail ornament baked into a repeat tile, unreadable low-tier material, or non-escalating prestige. Regenerate only the rejected kit.

- [ ] **Step 7: Run the production asset gate**

Run: `npx vitest run src/generated-assets.test.ts`

Expected: all frame files PASS exact size, transparency, edge, and coverage checks.

- [ ] **Step 8: Commit**

```bash
git add src/generated-assets.test.ts public/frames
git commit -m "feat: add generated rank and prestige frame kits"
```

---

### Task 3: Add frame registries, fallback resolution, and a single-crest model

**Files:**
- Create: `src/frame-assets.ts`
- Create: `src/frame-assets.test.ts`
- Modify: `src/progress-view.ts`
- Modify: `src/progress-view.test.ts`

**Interfaces:**
- Produces:

```ts
export const RANK_TIERS = [
  "naked", "plastic", "wood", "iron", "bronze", "silver", "gold",
  "platinum", "emerald", "diamond", "master", "legend", "champion", "godlike",
] as const;

export type RankTier = typeof RANK_TIERS[number];
export type FrameSide = "top" | "right" | "bottom" | "left";
export type FrameCorner = "tl" | "tr" | "bl" | "br";

export type FramePieceSet = {
  key: string;
  rails: Record<FrameSide, string>;
  corners: Record<FrameCorner, string>;
  ornaments: Partial<Record<FrameSide, string>>;
  crestTop?: string;
};

export type ResolvedFrameDecoration = {
  requestedTier: RankTier;
  resolvedTier: RankTier;
  rank: FramePieceSet | null;
  prestige: FramePieceSet | null;
  prestigeText: string;
  diagnostics: string[];
};

export function prestigeLabel(prestige: number): string;
export async function resolveFrameDecoration(
  tier: string,
  prestige: number,
  probe?: (url: string) => Promise<boolean>,
): Promise<ResolvedFrameDecoration>;
```

- [ ] **Step 1: Write failing pure registry tests**

```ts
it("uses one prestige crest and clamps prestige art at ten", async () => {
  const model = await resolveFrameDecoration("godlike", 12, async () => true);
  expect(model.prestige?.crestTop).toBe("/frames/prestige/10/crest-top.png");
  expect(model.prestigeText).toBe("P12");
});

it("falls back to the nearest complete lower rank", async () => {
  const model = await resolveFrameDecoration("emerald", 0, async (url) =>
    !url.includes("/emerald/") && !url.includes("/platinum/"),
  );
  expect(model.resolvedTier).toBe("gold");
  expect(model.diagnostics).toHaveLength(2);
});

it("renders roman prestige labels from one through ten", () => {
  expect(Array.from({ length: 10 }, (_, i) => prestigeLabel(i + 1))).toEqual(
    ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"],
  );
});
```

- [ ] **Step 2: Run and verify RED**

Run: `npx vitest run src/frame-assets.test.ts src/progress-view.test.ts`

Expected: FAIL because the registry is missing and the current footer still models stacked badges.

- [ ] **Step 3: Implement convention-based paths and complete-kit probing**

Build rank/prestige paths from directory conventions, probe every required piece for one candidate, and fall back only when the candidate kit is incomplete. Use the existing `probeImage` default. A rank crest and prestige crest never coexist in the resolved model: when prestige is positive, the prestige crest owns the sole top anchor.

- [ ] **Step 4: Retire stacked badge helpers**

Remove `badgeSlots` from `progress-view.ts` and its tests. Prestige identity now comes from `ResolvedFrameDecoration.prestigeText` at the frame crest.

- [ ] **Step 5: Run and verify GREEN**

Run: `npx vitest run src/frame-assets.test.ts src/progress-view.test.ts`

Expected: all registry, fallback, and prestige-label tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src/frame-assets.ts src/frame-assets.test.ts src/progress-view.ts src/progress-view.test.ts
git commit -m "feat: resolve rank and prestige frame assets"
```

---

### Task 4: Replace the CSS border with the generated perimeter renderer

**Files:**
- Create: `src/frame-renderer.ts`
- Create: `src/frame-renderer.test.ts`
- Modify: `index.html`
- Modify: `src/main.ts`
- Modify: `src/styles.css`
- Modify: `src/styles.test.ts`
- Modify: `src/window-layout.ts`
- Modify: `src/window-layout.test.ts`
- Modify: `src/rank-decoration.test.ts`
- Modify: `src-tauri/tauri.conf.json`
- Delete: `public/badges/prestige-1.png` through `public/badges/prestige-10.png`

**Interfaces:**
- Consumes: `ResolvedFrameDecoration`.
- Produces:

```ts
export const FRAME_BLEED = 24;
export function frameLayerHtml(): string;
export function frameRenderPlan(model: ResolvedFrameDecoration): {
  cssVariables: Record<string, string>;
  ornamentCounts: Record<FrameSide, number>;
  prestigeText: string;
};
export function applyFrameDecoration(
  perimeter: HTMLElement,
  model: ResolvedFrameDecoration,
): void;
```

- [ ] **Step 1: Write failing structure and geometry tests**

Require exactly four rails, four corners, four ornament lanes, one crest anchor, and one prestige text element:

```ts
it("renders one normalized perimeter structure", async () => {
  const html = frameLayerHtml();
  expect((html.match(/data-frame-rail=/g) ?? [])).toHaveLength(4);
  expect((html.match(/data-frame-corner=/g) ?? [])).toHaveLength(4);
  expect((html.match(/data-frame-ornaments=/g) ?? [])).toHaveLength(4);
  expect((html.match(/data-frame-crest/g) ?? [])).toHaveLength(1);
  expect((html.match(/data-prestige-text/g) ?? [])).toHaveLength(1);

  const model = await resolveFrameDecoration("godlike", 10, async () => true);
  const plan = frameRenderPlan(model);
  expect(plan.ornamentCounts).toEqual({ top: 2, right: 1, bottom: 2, left: 1 });
  expect(plan.prestigeText).toBe("X");
});
```

Update layout tests:

```ts
expect(FRAME_BLEED).toBe(24);
expect(scaledRosterSize(207.2, 1)).toEqual({ width: 504, height: 258 });
expect(tauriConfig.app.windows[0]).toMatchObject({ width: 504, height: 223 });
```

- [ ] **Step 2: Run and verify RED**

Run: `npx vitest run src/frame-renderer.test.ts src/window-layout.test.ts src/styles.test.ts`

Expected: FAIL because the perimeter renderer and bleed geometry do not exist.

- [ ] **Step 3: Create the transparent shell and glass inset**

Replace the old empty `#frame` with one empty perimeter mount in `index.html`:

```html
<div id="root" data-tauri-drag-region="deep">
  <div id="glass">
    <div id="content">
      <div id="card">
        <section id="card-claude" class="provider-card claude"></section>
        <section id="card-codex" class="provider-card codex"></section>
      </div>
      <div id="progress">
        <span class="level"></span>
        <div class="xpbar"><div class="xpfill"></div></div>
      </div>
    </div>
    <button id="action" hidden></button>
    <div id="ceremony" hidden>
      <div class="ceremony-panel">
        <h1></h1>
        <p></p>
        <button class="confirm"></button>
        <button class="later">Later</button>
      </div>
    </div>
  </div>
  <div id="perimeter" aria-hidden="true"></div>
</div>
```

At frontend startup, populate the single source of markup truth:

```ts
const perimeter = document.getElementById("perimeter")!;
perimeter.innerHTML = frameLayerHtml();
```

`#root` is transparent and clips only at the webview edge. `#glass` uses `position:absolute; inset:24px`, owns the current glass background/radius, and remains 456px wide. Corners and the 96x48 crest center on the glass edge and fit exactly within the 24px bleed.

- [ ] **Step 4: Update fixed window geometry without shrinking content**

In `window-layout.ts`:

```ts
export const ROSTER_WIDTH = 456;
export const FRAME_BLEED = 24;
export const WINDOW_WIDTH = ROSTER_WIDTH + FRAME_BLEED * 2;

export function scaledRosterSize(contentHeight: number, scale: number): Size {
  return {
    width: Math.round(WINDOW_WIDTH * scale),
    height: Math.ceil((rosterHeight(contentHeight) + FRAME_BLEED * 2) * scale),
  };
}
```

Set Tauri initial geometry to `width: 504`, `height: 223`. Keep non-resizable behavior and saved-position clamping.

- [ ] **Step 5: Implement fixed pieces and evenly spaced ornament lanes**

`frameRenderPlan` converts the resolved model into deterministic CSS variables,
ornament counts, and crest text without touching the DOM. `applyFrameDecoration`
applies that plan, updates `data-rank` and `data-prestige`, and creates ornament
instances only in the four existing lanes. Horizontal lanes use `display:grid;
grid-auto-flow:column; justify-content:space-evenly`; vertical lanes use the row
equivalent. Never put ornament pixels in rail backgrounds.

Use two horizontal ornament instances on top/bottom and one vertical instance on each side for decorated ranks. Prestige ornaments replace rank ornaments in the same lanes rather than adding a second lane.

- [ ] **Step 6: Remove the old visible CSS perimeter**

Delete or neutralize `#root` border/ring/corner-tick art, `#frame`, `champion-radiance`, and `godlike-halo`. Keep one continuous fallback border on `#glass`. Generated rails are stationary backgrounds; Prestige VII-X adds only an animated masked highlight in `.frame-rail::after`.

Update reduced-motion rules to stop the light pass and corner flashes.

- [ ] **Step 7: Wire asynchronous resolution without races**

In `main.ts`:

```ts
let decorationRevision = 0;

async function updateFrameDecoration(tier: string, prestige: number): Promise<void> {
  const revision = ++decorationRevision;
  const model = await resolveFrameDecoration(tier, prestige);
  if (revision !== decorationRevision) return;
  applyFrameDecoration(document.getElementById("perimeter")!, model);
  for (const diagnostic of model.diagnostics) console.warn(`[mana] ${diagnostic}`);
}
```

Call it from `renderProgress` after updating rank data.

- [ ] **Step 8: Remove footer badge DOM and CSS**

Remove `.badges` from `index.html`, `badgeHtml` and image probing from `main.ts`, and `.badge` rules from `styles.css`. Delete the retired `public/badges/prestige-*.png` files and replace their old assertions in `rank-decoration.test.ts` with generated prestige-kit assertions. The single frame crest carries prestige identity.

- [ ] **Step 9: Run focused tests and verify GREEN**

Run:

```bash
npx vitest run src/frame-assets.test.ts src/frame-renderer.test.ts src/window-layout.test.ts src/styles.test.ts src/progress-view.test.ts
npm run build
```

Expected: all focused tests and the frontend build PASS.

- [ ] **Step 10: Commit**

```bash
git add index.html src/main.ts src/styles.css src/styles.test.ts src/frame-renderer.ts src/frame-renderer.test.ts src/window-layout.ts src/window-layout.test.ts src/rank-decoration.test.ts src-tauri/tauri.conf.json public/badges
git commit -m "feat: render generated application frames"
```

---

### Task 5: Add deterministic aura normalization and generate six atlases

**Files:**
- Create: `scripts/normalize_aura_art.py`
- Create: `scripts/test_normalize_aura_art.py`
- Create: `public/effects/claude-aura-low.png`
- Create: `public/effects/claude-aura-mid.png`
- Create: `public/effects/claude-aura-high.png`
- Create: `public/effects/codex-aura-low.png`
- Create: `public/effects/codex-aura-mid.png`
- Create: `public/effects/codex-aura-high.png`
- Modify: `src/generated-assets.test.ts`

**Interfaces:**
- Produces:

```python
def normalize_aura_frames(
    source: Image.Image,
    columns: int,
    rows: int,
    frame_count: int,
) -> Image.Image:
    """Return a horizontal RGBA strip of frame_count 192x192 anchored cells."""
```

- [ ] **Step 1: Write failing normalization tests**

Use synthetic frames with different subject sizes. Assert output width is `192 * frame_count`, height is 192, every frame has transparent edges, and each visible subject shares the same bottom baseline and center anchor.

- [ ] **Step 2: Run and verify RED**

Run: `python3 -m unittest scripts/test_normalize_aura_art.py -v`

Expected: FAIL because the aura normalizer is missing.

- [ ] **Step 3: Implement baseline-preserving normalization**

Split the source grid using rounded proportional boundaries, find each cell alpha box above 16, calculate one shared scale from the largest width/height, resize with nearest-neighbor, center horizontally, and bottom-align every subject to source y=184. Expose:

```text
--input SOURCE --output DEST --columns N --rows N --frames N
```

- [ ] **Step 4: Generate provider and band atlases**

Use built-in image generation on a flat #ff00ff background:

- low: 2 columns x 1 row, two quiet frames;
- mid: 4 columns x 1 row, four traveling-particle frames;
- high: 4 columns x 2 rows, eight authored buildup/travel/accent/settle frames.

Claude prompt invariants: orange-gold flame tongues, emerald poison motes, rising vapor curls, ember accents, large empty character opening, no full ring/wreath/wings/crest.

Codex prompt invariants: cyan/royal-blue lightning forks, pale ice sparks, traveling snowflakes, crystalline accent, large empty character opening, no full ring/wreath/wings/crest.

Use the approved high-tier preview as a visual reference. Each frame visibly progresses while keeping a compact footprint.

- [ ] **Step 5: Remove chroma, normalize, and verify**

Run the installed chroma helper, then `normalize_aura_art.py` with the matching grid dimensions. For the Claude high atlas:

```bash
CLAUDE_HIGH_SOURCE="tmp/generated/claude-aura-high-source.png"
CLAUDE_HIGH_ALPHA="tmp/generated/claude-aura-high-alpha.png"
python "${CODEX_HOME:-$HOME/.codex}/skills/.system/imagegen/scripts/remove_chroma_key.py" \
  --input "$CLAUDE_HIGH_SOURCE" --out "$CLAUDE_HIGH_ALPHA" --auto-key border \
  --soft-matte --transparent-threshold 12 --opaque-threshold 220 --despill
python3 scripts/normalize_aura_art.py \
  --input "$CLAUDE_HIGH_ALPHA" --output public/effects/claude-aura-high.png \
  --columns 4 --rows 2 --frames 8
```

Repeat with `--columns 2 --rows 1 --frames 2` for low and `--columns 4 --rows 1 --frames 4` for mid. Extend `generated-assets.test.ts` to assert `(384,192)`, `(768,192)`, and `(1536,192)` for low, mid, and high atlases.

Run:

```bash
python3 -m unittest scripts/test_normalize_aura_art.py -v
npx vitest run src/generated-assets.test.ts
```

Expected: all normalizer and production asset tests PASS.

- [ ] **Step 6: Commit**

```bash
git add scripts/normalize_aura_art.py scripts/test_normalize_aura_art.py public/effects src/generated-assets.test.ts
git commit -m "feat: add generated elemental aura atlases"
```

---

### Task 6: Add aura bands, irregular scheduling, and grounded rendering

**Files:**
- Create: `src/aura-assets.ts`
- Create: `src/aura-assets.test.ts`
- Create: `src/aura-animation.ts`
- Create: `src/aura-animation.test.ts`
- Modify: `src/view.ts`
- Modify: `src/view.test.ts`
- Modify: `src/main.ts`
- Modify: `src/styles.css`
- Modify: `src/styles.test.ts`

**Interfaces:**
- Produces:

```ts
export type Provider = "claude" | "codex";
export type AuraBand = "low" | "mid" | "high";

export type AuraDescriptor = {
  provider: Provider;
  band: AuraBand;
  atlasUrl: string;
  frameCount: 2 | 4 | 8;
  cellSizeCss: 96;
  frameHoldsMs: readonly number[];
  phaseOffsetMs: number;
  spriteXOffsetPx: number;
  spriteYOffsetPx: number;
};

export function auraBandForTier(tier: string): AuraBand | null;
export function resolveAura(provider: Provider, tier: string, prestige: number): AuraDescriptor | null;
export function auraFrameAt(elapsedMs: number, descriptor: AuraDescriptor, reducedMotion: boolean): number;
export function auraFrameDelayAt(elapsedMs: number, descriptor: AuraDescriptor, reducedMotion: boolean): number | undefined;
```

- [ ] **Step 1: Write failing band and timing tests**

```ts
expect(auraBandForTier("iron")).toBeNull();
expect(auraBandForTier("bronze")).toBe("low");
expect(auraBandForTier("platinum")).toBe("mid");
expect(auraBandForTier("master")).toBe("high");

const claude = resolveAura("claude", "godlike", 0)!;
const codex = resolveAura("codex", "godlike", 0)!;
expect(claude.frameHoldsMs.reduce((a, b) => a + b, 0)).toBe(3200);
expect(codex.frameHoldsMs.reduce((a, b) => a + b, 0)).toBe(3650);
expect(codex.phaseOffsetMs).not.toBe(claude.phaseOffsetMs);
expect(auraFrameAt(9999, claude, true)).toBe(0);
```

- [ ] **Step 2: Write failing provider-visibility markup tests**

Update `renderProvider` behavior behind a pure helper so an explicitly unauthenticated snapshot produces empty card HTML. For visible providers require:

```ts
expect(cardHtml(snapshot, "claude")).toContain('class="aura"');
expect(cardHtml({ ...snapshot, authenticated: false }, "claude")).toBe("");
```

- [ ] **Step 3: Run and verify RED**

Run: `npx vitest run src/aura-assets.test.ts src/aura-animation.test.ts src/view.test.ts`

Expected: FAIL because aura modules and markup are missing.

- [ ] **Step 4: Implement descriptors and uneven frame holds**

Use exact high-tier holds:

```ts
const HIGH_HOLDS = {
  claude: [352, 384, 384, 384, 384, 384, 384, 544],
  codex: [511, 401, 474, 401, 474, 401, 438, 550],
} as const;
```

These total 3200ms and 3650ms. Claude phase is 0; Codex phase is 1380ms. Low and mid bands use provider-specific two/four-frame subsets with slower holds.

- [ ] **Step 5: Add a single deadline-based aura scheduler**

Follow `sprite-animation.ts`: compute phase-adjusted elapsed time, walk cumulative frame holds, and return the delay until the next boundary. `main.ts` owns one `auraFrameTimer`, updates all `.aura` elements, and schedules the minimum next delay. Do not create CSS keyframe loops or one interval per element.

- [ ] **Step 6: Render and ground the aura behind the sprite**

Visible provider markup becomes:

```html
<div class="familiar-slot">
  <div class="aura" data-provider="claude" data-frame="0" aria-hidden="true"></div>
  <div class="sprite claude-mage" data-provider="claude" data-state="idle" data-frame="0" aria-hidden="true"></div>
</div>
```

The aura is an absolute 96x96 layer with a 96px background cell. `data-frame="0"` through `7` select `background-position-x` in 96px steps. Set `background-size` from the descriptor frame count.

Apply exact sprite-art offsets at the 68px art scale:

```css
.sprite.claude-mage { --sprite-x-offset: -4px; --sprite-y-offset: 10px; }
.sprite.codex-mage { --sprite-x-offset: -3px; --sprite-y-offset: 8px; }
.sprite::before {
  left: calc(50% + var(--sprite-x-offset, 0px));
  top: calc(50% + var(--sprite-y-offset, 0px));
}
```

Remove the generic `.familiar-slot::before` radial aura. A missing aura image leaves the sprite visible.

- [ ] **Step 7: Preserve reduced motion and asynchronous redraws**

The scheduler sets every aura to frame 0 and stops its timer when reduced motion is active. Rebuilding a provider or changing rank calls one shared `syncAuraFrames(performance.now())`; it does not reset provider phase.

- [ ] **Step 8: Run focused tests and verify GREEN**

Run:

```bash
npx vitest run src/aura-assets.test.ts src/aura-animation.test.ts src/view.test.ts src/styles.test.ts
npm run build
```

Expected: all aura, markup, style, and build checks PASS.

- [ ] **Step 9: Commit**

```bash
git add src/aura-assets.ts src/aura-assets.test.ts src/aura-animation.ts src/aura-animation.test.ts src/view.ts src/view.test.ts src/main.ts src/styles.css src/styles.test.ts
git commit -m "feat: animate provider elemental auras"
```

---

### Task 7: Build a visual matrix, package, and update public documentation

**Files:**
- Create: `preview.html`
- Create: `src/preview.ts`
- Modify: `README.md`
- Replace: `docs/images/mana-widget.png`

**Interfaces:**
- Consumes: frame renderer, aura renderer, current card markup, and query parameters `rank`, `prestige`, and `providers`.
- Produces: a Vite-served deterministic visual harness and the final release screenshot.

- [ ] **Step 1: Add the preview harness**

`preview.html` imports `src/styles.css` and `src/preview.ts`. `preview.ts` renders fixed Claude/Codex usage rows, applies the requested rank/prestige, and supports `providers=claude`, `providers=codex`, or `providers=both`. It never calls Tauri APIs or reads user credentials.

- [ ] **Step 2: Run the complete automated suite**

Run:

```bash
python3 -m unittest discover -s scripts -p 'test_*.py' -v
npm test
npm run build
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: every command exits 0.

- [ ] **Step 3: Capture the visual matrix with browser automation**

Start Vite and inspect at desktop scale:

```bash
npm run dev -- --host 127.0.0.1
```

Capture low, bronze, emerald, legend, champion, godlike, Prestige I, VII, IX, and X; repeat the apex states for Claude-only, Codex-only, both providers, and reduced motion. Compare two samples per animated state. Rail/corner positions and familiar element bounds must be pixel-identical; only masked light and aura pixels may change.

- [ ] **Step 4: Inspect the real Tauri panel**

Run `npm run tauri dev`, confirm the 456px glass content remains readable inside the 24px bleed, the menu-bar-only behavior remains intact, and frame ornaments are not clipped by the native webview.

- [ ] **Step 5: Build and inspect the packaged app**

Run: `npm run tauri build -- --bundles app`

Expected: `src-tauri/target/release/bundle/macos/Mana.app` exists. Launch it only after the progress-durability plan's disposable migration check has passed.

- [ ] **Step 6: Update public documentation**

Replace `docs/images/mana-widget.png` with the final Godlike plus Prestige X screenshot and update README's feature list to mention generated rank frames, highest-earned prestige crest, provider auras, and update-safe local progression.

- [ ] **Step 7: Commit**

```bash
git add preview.html src/preview.ts README.md docs/images/mana-widget.png
git commit -m "docs: show generated Mana progression visuals"
```
