# Generated Rank Frames, Prestige Effects, Character Auras, and Durable Progress

Date: 2026-07-22
Status: approved design; implementation pending

## Goal

Replace Mana's CSS-drawn application border with generated pixel-art fantasy
frames that become more impressive with rank and prestige. Upgrade Claude and
Codex with provider-specific animated elemental auras, while guaranteeing that
installing an update does not silently reset existing level or prestige data.

The result must retain Mana's compact, readable widget layout. The visual
progression should feel like an early-2000s fantasy MMORPG HUD without turning
the application perimeter or character area into constant spectacle.

## Superseded Decisions

This specification supersedes two older decisions:

1. Rank borders are no longer handcrafted CSS themes. Generated bitmap pieces
   become the visible application perimeter.
2. Missing or corrupt progression files no longer silently return a fresh
   default state when recoverable or unreadable progress files exist.

The XP curve, rank gates, prestige math, provider usage calculation, mana bar
geometry, and authenticated-provider visibility remain unchanged.

## Approved Visual Direction

### Application perimeter

- Generated pixel art replaces the visible CSS border around the entire
  application container. It does not sit inside the content area.
- The frame scales to the container by composing native-proportion parts. No
  complete frame image is stretched to fit.
- Each rank has its own material and ornament language from `naked` through
  `godlike`.
- Rank progression remains restrained at low tiers and grows more elaborate at
  high tiers.
- The frame cannot obscure provider labels, meters, percentages, reset times,
  the divider, or the progression footer.
- No dotted or dashed outline is visible behind or outside the generated art.
- The glass container remains visible inside the generated perimeter.

### Frame composition

Every rank frame uses the same rendering contract:

- four fixed-proportion corner pieces;
- four quiet, seamless rail textures that cover the full perimeter between
  corners;
- optional native-size rail ornaments distributed evenly between corners;
- one optional top-center rank crest at elite ranks;
- transparent overflow for ornaments that extend outside the container;
- a continuous 1px rank-colored fallback outline used only while art is
  unavailable.

Rail textures may repeat, but decorative motifs do not live at a repeat seam.
The renderer places rail ornaments as independent native-size images with
equal spacing. Corners, crests, starbursts, and rail nodes never scale with the
container. Only the quiet rail span grows or repeats.

This separation prevents the duplicated, compressed, and uneven motifs seen in
the earlier full-frame experiments.

### Rank escalation

The 14 existing ranks keep their current names and material identity:

| Band | Ranks | Perimeter treatment |
|---|---|---|
| Undressed | naked | No illustrated perimeter; glass edge only |
| Found | plastic, wood | Simple material rails and small square corners |
| Forged | iron, bronze, silver | Metal rails, rivets, restrained corner leaves |
| Royal | gold, platinum | Layered bright metal, compact center crest |
| Gem | emerald, diamond | Mounted gems, dark metal contrast, clean facets |
| Mythic | master, legend | Strong color channels, runes, larger corners |
| Apex | champion, godlike | Celestial metal, richer crest, luminous inlay |

Every rank receives custom generated art. Recoloring one generic CSS frame is
not sufficient.

### Prestige composition

Prestige augments the active rank frame instead of replacing it with an
unrelated second frame. Only one prestige system is rendered:

- one top-center prestige emblem aligned to the top rail;
- four corner starbursts at the frame corners;
- prestige rail inlay layered into the rank rails;
- the Roman numeral rendered as interface text, not baked into the bitmap;
- no side medallions, bottom crest, detached rail fragments, or oversized
  wings.

The highest earned prestige is shown. Previously earned prestige decorations
are not stacked.

### Prestige escalation

| Prestige | Treatment |
|---|---|
| I-III | Compact gem crest, faint rail inlay, very small corner glint |
| IV-VI | Broader crest shoulders, two-tone inlay, clearer corner starbursts |
| VII | Purple prestige channel introduced into all four rails |
| VIII | Cyan facets and a slightly stronger corner halo |
| IX | Alternating gemstone inlay and mature four-corner starbursts |
| X | Celestial twin-channel rail, Ascendant crest, brightest controlled light pass |

Prestige counts above X reuse the Prestige X visual treatment and keep the
actual count in text.

### Perimeter motion

- Rank artwork itself stays registered and static.
- Prestige VII-X may animate a masked light pass along a stationary rail.
- Prestige corner starbursts use short opacity or highlight flashes with
  staggered delays. They do not scale, orbit, or bounce.
- Prestige X is the richest state, but still uses one crest, four corners, and
  one rail system.
- Background positions for generated rail art remain fixed during animation.
- `prefers-reduced-motion` freezes light passes and flashes without hiding the
  earned frame.

## Character Aura System

### Layering

Provider character rendering becomes three independent layers:

1. a generated elemental aura behind the character;
2. the existing rank-specific character sprite animation;
3. optional tiny foreground particles for the highest aura band.

The aura is not baked into the character atlas. Character and aura animations
can therefore run at different speeds and phases without changing rank sprite
assets.

### Provider identity

- Claude uses orange-gold fire, emerald poison motes, rising vapor curls, and
  ember accents.
- Codex uses cyan and royal-blue lightning, pale ice sparks, drifting
  snowflakes, and crystalline accents.
- Both providers have comparable visible energy and footprint.
- Neither provider uses a generic radial glow, full wreath, crown, wing set,
  or permanent circle.

### Aura escalation

| Rank band | Aura behavior |
|---|---|
| naked-iron | No aura |
| bronze-gold | Two-frame quiet elemental flicker |
| platinum-diamond | Four-frame traveling particles and restrained buildup |
| master-godlike | Eight authored frames with buildup, travel, accent, and settle |
| Prestige VII-X | Same footprint with one extra glint and a slightly brighter rare accent |

The approved high-tier animation uses eight distinct generated effect frames.
Particles travel between frames and elemental silhouettes change; the effect
is not an opacity-only glow.

### Timing and registration

- High-tier Claude loops in 3.2 seconds.
- High-tier Codex loops in 3.65 seconds with a negative phase
  offset.
- Frame holds are intentionally uneven so motion feels hand animated.
- Every aura frame shares one center and baseline registration box.
- The aura footprint stays fixed while its contents animate.
- The character sprite's visible opaque bounds are optically centered and
  lowered into the aura baseline using provider-specific CSS variables.
- Aura animation never changes the character element's bounding box.
- Existing character idle, working, and hover animations continue unchanged.
- Reduced-motion mode freezes the aura on its quiet first frame.

Production aura atlases use transparent 2x source art. High-tier atlases use
eight horizontal square cells. The display target is 96x96 CSS pixels around
the current 68x68 character art. At that scale, Claude uses a -4px horizontal
and +10px vertical sprite-art offset; Codex uses a -3px horizontal and +8px
vertical sprite-art offset. The aura itself stays centered on the familiar
slot.

## Asset Contracts

### Rank assets

Production files live under:

`public/frames/ranks/<tier>/`

Each illustrated rank provides:

- `rail-h.png`: seamless horizontal rail texture;
- `rail-v.png`: seamless vertical rail texture;
- `corner-tl.png`, `corner-tr.png`, `corner-bl.png`, `corner-br.png`;
- `ornament-h.png` and `ornament-v.png` when the rank has rail motifs;
- `crest-top.png` when the rank has a center crest.

Naked intentionally has no bitmap frame.

### Prestige assets

Production files live under:

`public/frames/prestige/<level>/`

Each level from 1 through 10 provides a transparent crest. Levels that change
the rail or corner treatment also provide their own rail inlay, corner
starburst, or ornament files. Direction-specific corner art is preferred over
rotating one bitmap when rotation would invert highlights.

### Aura assets

Production files live under:

`public/effects/`

File names use:

`<provider>-aura-<band>.png`

where provider is `claude` or `codex`, and band is `low`, `mid`, or `high`.
All frames in one atlas have identical dimensions, transparent exteriors, and
a common anchor. No non-transparent pixel may touch the atlas edge.

### Generated-art processing

- Image generation creates the visual source; deterministic local processing
  removes chroma backgrounds, separates cells, normalizes registration, and
  writes final PNGs.
- Production assets are copied into `public/`; no runtime asset may depend on
  `.superpowers/`, temporary files, or `$CODEX_HOME/generated_images`.
- Generated sources are never scaled non-uniformly.
- Asset tests enforce dimensions, alpha, edge clearance, frame count, and
  non-empty coverage.

## Frontend Architecture

### Asset registries

`src/frame-assets.ts` owns the mapping from rank and prestige to frame pieces.
It returns a complete resolved decoration model and falls back to the nearest
valid lower visual tier when optional art is unavailable.

`src/aura-assets.ts` maps provider plus rank/prestige to an aura atlas, frame
count, frame timing, and optical registration variables.

Progression math never imports either registry.

### Frame renderer

The application shell gains one pointer-inert perimeter layer containing:

- four rail spans;
- four corner anchors;
- four ornament lanes;
- one top-center crest anchor;
- one prestige overlay layer.

The renderer updates CSS variables and data attributes from the current rank
and prestige. Content layout remains independent from perimeter overflow.
Changing frame internals must not require changes to provider cards or meters.

Ornament lanes use native-size items and equal distribution. A side never
contains two overlapping renderers, and a motif cannot be supplied both by a
rail background and an ornament element.

### Aura renderer

The provider familiar slot gains a dedicated aura element behind `.sprite`.
The aura scheduler follows the existing deadline-based sprite scheduling
pattern rather than starting one interval per element. Provider duration and
phase are deterministic, so redraws do not resynchronize the two characters.

The existing rule remains: a provider section is hidden when its snapshot is
explicitly unauthenticated. Aura elements are created only for visible
provider sections.

## Durable Progression Storage

### Non-negotiable invariant

An application update must not intentionally reset or re-baseline valid
existing level, rank, prestige, token floor, tally total, or scan cursors.

The Tauri identifier remains `com.vantasoft.mana`. The app data directory and
the `progress.json` location remain stable. Renaming the product, bundle, or
repository must not change that identifier without a separate migration.

### Versioned envelope

New saves use a versioned document:

```json
{
  "schema_version": 2,
  "state": {
    "rank": 8,
    "prestige": 7,
    "prestige_token_floor": 123456,
    "initialized": true,
    "tally": {}
  }
}
```

The current unversioned `ProgressState` document is schema version 1. Loading a
version 1 file migrates every existing field into version 2 without applying
`initialize_baseline` and without resetting rank or prestige. Any successfully
parsed legacy progress file is treated as already initialized, including older
documents that predate the `initialized` field. Only a genuinely new install
with no progress files begins uninitialized.

Future migrations are explicit, ordered functions from one schema version to
the next. Unknown future schema versions are rejected rather than partially
deserialized.

### Files

- `progress.json`: current primary state;
- `progress.json.bak`: last known-good state;
- `progress.pre-migration-v1.json`: immutable byte-for-byte snapshot of the
  first legacy file migrated to version 2;
- `progress.json.tmp`: same-directory temporary file used only during an
  atomic save.

The pre-migration snapshot uses create-new semantics and is never overwritten.

### Startup recovery order

1. Load and validate `progress.json`.
2. If invalid, load and validate `progress.json.bak`.
3. If needed, load and migrate `progress.pre-migration-v1.json`.
4. If no progress files exist, create a genuinely new default state.
5. If progress files exist but none are valid, report a recovery error and do
   not overwrite them with a default state.

The loader reports whether state came from the primary, backup, legacy
snapshot, or a new install. Recovery from a backup rewrites the primary only
after the recovered state validates.

### Atomic save

Saving uses this sequence in the app data directory:

1. Serialize the complete versioned state in memory.
2. Write all bytes to `progress.json.tmp`.
3. Flush and `sync_all` the temporary file.
4. Re-read and validate the temporary file.
5. Move the previous valid primary to `progress.json.bak`.
6. Rename the temporary file to `progress.json`.
7. Sync the parent directory.

A crash between steps leaves either a valid primary or a valid backup. Startup
removes an abandoned invalid temporary file only after another valid state has
loaded.

### Transactional updates

Rank-up and prestige commands apply changes to a cloned candidate state. They
persist the candidate before replacing the in-memory state and emitting an
update. A persistence failure leaves the prior state active and returns an
error.

The background tally watcher also scans into a candidate clone. It commits the
candidate to memory only after persistence succeeds. Failed saves are retried
on the next watcher tick, so token offsets cannot advance without their tally
being durably stored.

## Error Handling and Fallbacks

- A missing rank or prestige bitmap falls back to the closest valid lower
  cosmetic tier and records a diagnostic.
- The continuous fallback outline appears only when the required perimeter
  art cannot load. It is never dotted or dashed.
- Aura failure removes the aura layer but keeps the character visible.
- Cosmetic loading cannot change progression state.
- Persistence errors do not silently disappear. Commands return them and the
  watcher logs them while retaining the last valid state.
- Invalid progression files remain on disk for recovery and are never replaced
  by a new default automatically.

## Testing and Release Validation

### Asset tests

- Validate expected file presence for every rank and prestige.
- Validate PNG dimensions, alpha channel, edge clearance, and visible-pixel
  coverage.
- Validate aura frame count and identical cell dimensions.
- Validate common aura baseline and center registration.
- Validate that rail ornament motifs are not duplicated inside rail textures.

### Frontend tests

- Resolve every rank and prestige to one decoration model.
- Confirm missing art falls back to the nearest lower tier.
- Confirm a side has one rail, one ornament lane, and no overlapping motif
  sources.
- Confirm provider sections and their aura layers are absent when explicitly
  unauthenticated.
- Confirm aura frame indices advance while sprite and aura element bounds stay
  fixed.
- Confirm Claude and Codex use different durations and phases.
- Confirm reduced-motion mode freezes aura and perimeter effects.

### Progression tests

- Round-trip the version 2 envelope.
- Migrate an exact fixture of the current unversioned schema without changing
  rank, prestige, token floor, initialized state, total tokens, or scan
  cursors.
- Preserve progress when a legacy fixture lacks fields introduced after its
  creation.
- Recover from a corrupt primary using the last-known-good backup.
- Recover from the immutable pre-migration snapshot when both current files
  are invalid.
- Refuse to default when invalid progress files exist.
- Simulate interrupted saves at each rename boundary.
- Verify failed rank-up, prestige, and watcher saves do not commit in-memory
  mutations.
- Assert `com.vantasoft.mana` remains the configured Tauri identifier.

### Visual QA

- Capture the real widget at low, forged, gem, mythic, apex, Prestige I,
  Prestige VII, Prestige IX, and Prestige X states.
- Test Claude-only, Codex-only, and both-provider layouts.
- Test normal and reduced-motion modes.
- Verify the generated perimeter covers the complete container at every
  supported widget size without short rails, gaps, stretching, or clipped
  crests.
- Compare two animation samples to prove rail art and character bounds stay
  fixed while highlights, particles, and aura frames advance.
- Install the new build over a copy of current app data and verify level, rank,
  prestige, token floor, tally total, and cursors are identical after launch.
- Build the Tauri application and inspect the packaged macOS widget before
  release.

## Acceptance Criteria

- Generated pixel-art frame pieces visibly replace the CSS application border
  for every illustrated rank.
- Frames surround the complete glass container and scale without stretching or
  compressed ornament.
- Rail motifs are evenly spaced with no duplicated seam artifacts.
- Prestige I-X visibly escalate, with Prestige X the richest controlled state.
- Prestige uses one crest, four corner starbursts, and one rail system.
- Claude and Codex use richer provider-specific frame animation rather than an
  opacity-only glow.
- Character sprites remain centered and grounded inside their auras.
- Provider animations remain asynchronous and stable across rerenders.
- Existing authenticated-provider visibility behavior remains intact.
- Updating over a valid current progress file preserves all progression and
  tally fields.
- Missing or corrupt files never cause a silent reset when a backup or legacy
  snapshot can recover the state.
- All automated tests, the frontend build, the Rust test suite, and the Tauri
  package build pass.

## Out of Scope

- Changes to XP pacing, rank gates, prestige eligibility, or provider usage
  calculations.
- Cloud sync or cross-device progression.
- Sound effects, rank-up cinematics, or new interaction controls.
- A settings editor for frame or aura selection.
- Showing unauthenticated providers as locked rows.
