# Mana Prestige Productivity And Frame Polish Design

**Date:** 2026-07-23

## Goal

Make prestige progression measure productive output only, become materially
harder at higher prestige levels, preserve surplus XP across prestige
transactions, and present one clear generated prestige badge inside a frame
whose corners visibly connect to its rails.

## Non-Negotiable Requirements

- Claude progression counts only `message.usage.output_tokens`.
- Codex progression counts
  `total_token_usage.output_tokens + total_token_usage.reasoning_output_tokens`.
- Input tokens, cached input tokens, and cache-creation tokens never contribute
  XP or the lifetime productivity counter.
- Existing installations are recalculated immediately from output history still
  available in local Claude and Codex session files.
- Recalculation may decrease level, rank, and prestige.
- The pre-upgrade progress document is preserved in an immutable recovery copy
  before the new schema is published.
- Prestige spends only the exact cost of the completed cycle. Output earned
  beyond that cost carries into the next prestige.
- The UI shows exactly one prestige badge: the highest prestige produced by the
  recalculated current state.
- Hover replaces the level label with an exact, comma-separated lifetime output
  count without resizing the widget.
- Prestige corners are generated pixel-art L joints that connect to both rails.
- Prestige VII-X become increasingly ornate, with Prestige X as the apex.
- Reduced Motion freezes cosmetic animation without changing geometry.
- Mana remains menu-bar-only and must not gain a Dock icon.

## Output-Only Tally

### Claude

For every complete Claude transcript line, the scanner reads only:

```text
message.usage.output_tokens
```

Missing, malformed, negative, or non-integer values contribute zero. Existing
incremental byte-offset behavior remains unchanged so partial lines and
repeated scans cannot double-count output.

### Codex

For every Codex session, the scanner uses the latest cumulative:

```text
total_token_usage.output_tokens
total_token_usage.reasoning_output_tokens
```

The two fields are added with saturating arithmetic. The stored per-file
cumulative output total is compared with the latest value so only its positive
delta is added. Truncation or rotation resets the per-file baseline without
creating output.

### Public Progress View

The backend progress payload adds:

```text
lifetime_output_tokens: u64
```

The Rust field remains a `u64` but serializes across the Tauri boundary as an
unsigned decimal string, avoiding JavaScript's unsafe-integer range. This is
the only lifetime token value exposed in the progress footer. The UI validates
and formats that string with ASCII thousands separators:

```text
12,345,678 lifetime output
```

No abbreviated `K`, `M`, or `B` formatting is used.

## Schema V3 Migration

The progress document advances from schema version 2 to version 3. Version 3
stores output-only tally state:

```text
output_tokens
claude_offsets
codex_offsets
codex_output_totals
```

Loading a valid version 2 document performs these steps:

1. Validate the complete version 2 document before using any field.
2. Publish an immutable `progress.pre-migration-v2.json` recovery copy using
   staged write, file sync, atomic rename, and directory sync.
3. Preserve the original version 2 bytes exactly. Never overwrite an existing
   valid v2 recovery copy.
4. Return an in-memory `V2NeedsOutputRebuild` load result containing the
   validated v2 source and a fresh v3 candidate. This is process state, not a
   serialized v3 field.
5. Reset output-only counters and their cursors so retained session files are
   scanned from byte zero.
6. After the first complete output scan, derive level, rank, prestige, XP floor,
   and lifetime output from the retained output history.
7. Atomically publish the complete version 3 document only after the rebuild
   succeeds.

If the recovery copy, scan, derivation, or version 3 save fails, the version 2
primary remains authoritative and no partial version 3 state is committed.
Existing version 3 documents never repeat the rebuild.

An existing recovery path is accepted only when it contains a valid schema v2
document. The migration never replaces it, even when its bytes came from an
earlier v2 revision. An invalid or unreadable recovery path is a hard migration
failure so the process cannot silently destroy the only known backup.

Historical files that no longer exist cannot be reconstructed. This is an
accepted consequence of the requested full recalculation.

## Tiered Prestige Curve

The current prestige number selects the difficulty of the active cycle. The
cumulative multiplier is:

```text
M(p) =
  1.5 ^ min(p, 3)
  * 1.75 ^ min(max(p - 3, 0), 3)
  * 2 ^ max(p - 6, 0)
```

Examples:

| Prestige | Multiplier |
| --- | ---: |
| 0 | 1.000000 |
| I | 1.500000 |
| II | 2.250000 |
| III | 3.375000 |
| IV | 5.906250 |
| V | 10.335938 |
| VI | 18.087891 |
| VII | 36.175781 |
| VIII | 72.351563 |
| IX | 144.703125 |
| X | 289.406250 |

The level threshold remains cubic:

```text
floor(0.8 * level^3 * M(prestige))
```

The implementation uses exact integer rational arithmetic with `u128`
intermediates and saturates to `u64::MAX`; it does not use floating-point
progression math.

## Full Recalculation

After the migration output scan, progression is derived from total retained
output by spending complete prestige-cycle costs in order:

1. Start at Prestige 0 with all rebuilt output unspent.
2. A complete cycle costs the XP threshold for Level 120 at that prestige,
   multiplied by `TOKENS_PER_XP`.
3. While the unspent output covers the complete cycle, subtract that exact cost
   and increment prestige.
4. Use the remaining output to derive the active prestige's level.
5. Set rank to the highest rank gate reached by that level.
6. Store the sum of completed-cycle costs as the prestige output floor.

The calculation may increase or decrease prestige, rank, and level relative to
schema version 2. It is deterministic and idempotent for the same retained
history. The loop stops at the first unaffordable cycle. Once a cycle cost
saturates to `u64::MAX`, at most one such cycle can be completed by a `u64`
lifetime total, so extreme prestige values cannot create a zero-cost or
non-terminating loop.

## Prestige Carryover

Manual prestige remains available only at the final rank. The transaction:

1. Computes the exact Level 120 cycle cost for the current prestige.
2. Verifies that effective output covers that cost.
3. Increments prestige and resets rank to Unranked.
4. Advances the persisted output floor by exactly the completed cycle cost.
5. Leaves every output token above that floor available in the new cycle.
6. Saves the candidate state atomically before changing in-memory state or
   emitting `progress-update`.

The resulting new-cycle level may be greater than Level 1. Rank advancement
remains a deliberate user action after normal runtime prestige; carryover never
automatically invokes multiple rank-up or prestige ceremonies.

## Prestige Badge

Each prestige level continues to resolve its unique generated
`crest-top.png`. The renderer changes its placement:

- Remove the generic black Roman-numeral plaque.
- Remove the prestige crest from the top-center frame position.
- Render one scaled generated crest immediately before the footer level label.
- Show no crest at Prestige 0.
- Resolve at most one crest. "Highest" means the current prestige reached by
  the recalculated state, not a separate historical maximum. Use the highest
  authored Prestige X art for current prestige values above ten.
- Keep the text `Prestige <roman>` in the footer level label when not hovered.

The hover state keeps the crest visible and replaces only the text label with
the lifetime output counter. It does not add a tooltip, second row, or height
transition.

## Generated Corner Contract

All Prestige I-X corner sets are replaced. Each set begins with one generated
top-left canonical corner and derives the other three through deterministic
pixel-perfect reflections.

Every normalized corner is a `96x96` transparent PNG rendered at `48x48` CSS
pixels. Its authored shape must:

- form a clear 90-degree L joint;
- include horizontal and vertical rail sockets;
- reach the inner horizontal and vertical connection edges;
- keep the prestige ornament centered on the elbow instead of floating outside
  it;
- preserve at least four transparent source pixels at the two exterior edges;
- avoid text, loose fragments, detached sparkles, and duplicated rail ends.

The horizontal and vertical rails underlap the corner by eight CSS pixels.
Corners paint above rails. Generated art is never stretched; rails continue to
tile only along their long axis.

Prestige I-VI use restrained material progression. Prestige VII-IX add
increasing crystalline and celestial detail. Prestige X uses the brightest
prismatic-gold joint, but the rail sockets remain visually dominant enough to
read as one connected perimeter.

## Hover Interaction

The body hover state already drives familiar animation. It also sets a
presentation-only footer state:

```text
rest:  [highest crest] Lv 84 · Diamond · Prestige VII
hover: [highest crest] 12,345,678 lifetime output
```

The footer reserves one stable text track. Both strings are single-line,
truncated only if the fixed content width is genuinely exceeded, and use
tabular numerals. Entering or leaving hover does not resize or reposition the
window. After v3 migration, "lifetime" means the exact output total rebuilt
from retained local Claude and Codex history; output from deleted history is
not represented.

## Preview And Documentation

The browser-only preview accepts an additional deterministic
`outputTokens=<u64>` query value. It can render rest and hover footer states
without Tauri, credentials, or live progress access.

The visual matrix covers:

- Prestige I, IV, VII, IX, and X corners;
- the single footer crest at Prestige I, VII, and X;
- rest and hover footer copy;
- Claude-only, Codex-only, and both-provider layouts;
- animated and Reduced Motion states;
- fixed rail, corner, crest, footer, and root geometry across samples.

The README screenshot is replaced with the final Prestige X rest state after
browser and packaged-panel verification.

## Testing And Release Gates

### Rust

- Output-only Claude and Codex scanner fixtures.
- Idempotent incremental scans and rotation behavior.
- Exact tiered multiplier thresholds through Prestige X.
- Saturation behavior for extreme prestige values.
- Full recalculation across zero, partial, exact, and multi-cycle totals.
- Carryover at exact threshold, threshold plus remainder, and overflow edges.
- Failed persistence leaves in-memory and on-disk progression unchanged.
- Version 2 recovery copy publication, interruption boundaries, retry, and
  version 3 idempotence.

### Frontend

- Lifetime output field typing and exact comma formatting.
- One resolved prestige crest and no legacy numeral plaque.
- Hover swaps only footer text.
- Fixed footer geometry and no window resize.
- Missing crest art leaves readable text.
- Preview query validation and deterministic hover rendering.

### Generated Art

- Every Prestige I-X corner exists at `96x96`.
- All four corners are exact directional reflections of the canonical source.
- Required connection-edge alpha is present.
- Exterior edges remain clear.
- No corner is empty, clipped, or identical to another prestige level.

### Final Verification

- Full Python, frontend, Rust, build, and diff checks pass.
- Browser matrix has no detached corners, clipped crest, duplicate badge,
  layout shift, or animation-induced geometry change.
- A build-only `Mana.app` preserves bundle identifier, version, arm64
  architecture, and menu-bar-only activation policy.
- The native app is not installed or launched until explicitly approved.
- The pre-upgrade live progress file is never used as a test fixture.
