# Mana Prestige Productivity And Frame Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild Mana progression from output tokens only, apply the approved tiered prestige curve with exact carryover, show one generated prestige crest in the footer, and replace detached prestige corners with connected generated pixel-art joints.

**Architecture:** Rust remains the sole owner of token scanning, progression math, migration, and atomic persistence. Schema v3 loads a valid v2 document into an in-memory rebuild state, preserves the original bytes, rescans retained logs from zero, and publishes v3 only after derivation succeeds. The frontend receives an exact decimal lifetime-output string, swaps one stable footer text track on hover, and renders the resolved prestige crest separately from a perimeter assembled from tiled rails and reflected canonical corners.

**Tech Stack:** Rust 2021, serde/serde_json, Tauri v2, TypeScript 5.6, vanilla HTML/CSS, Vitest 3, Vite 6, Python 3, Pillow, built-in GPT Image 2 image generation, Playwright browser QA.

## Global Constraints

- Claude XP counts only `message.usage.output_tokens`.
- Codex XP counts only `total_token_usage.output_tokens + total_token_usage.reasoning_output_tokens`.
- Input, cached-input, and cache-creation tokens never contribute.
- Preserve the exact valid v2 bytes at `progress.pre-migration-v2.json` before publishing schema v3.
- Do not publish or mutate the v2 primary until the full retained-history rebuild and v3 staging both succeed.
- Existing progress may decrease after the requested output-only recalculation.
- Prestige multipliers are cumulative: I-III each `x1.5`, IV-VI each `x1.75`, and VII+ each `x2`.
- Prestige spends the exact Level 120 cycle cost; every surplus output token carries forward.
- The Rust lifetime count is `u64`, serialized to TypeScript as an unsigned decimal string.
- Show one crest for the current recalculated prestige, no crest at Prestige 0, and Prestige X art above ten.
- Remove the black Roman-numeral plaque and the prestige top-center frame crest.
- Prestige corners are generated `96x96` RGBA pixel-art L joints rendered at `48x48`; never stretch them.
- Rails tile only along their long axis and underlap corners by eight CSS pixels.
- Reduced Motion freezes cosmetic frame animation without changing geometry.
- Keep the 456px glass width, 24px perimeter bleed, menu-bar-only activation policy, and no Dock icon.
- Never run or install the newly built app against live progress during implementation. Browser previews and a build-only `.app` are allowed.
- Use TDD and commit after every independently passing task.

## File Structure

- `src-tauri/src/progress.rs`: output scanners, exact curve math, recalculation, carryover, and public progress payload.
- `src-tauri/src/progress_store.rs`: schema-specific wire types, immutable recovery copies, load outcomes, and durable v3 publication.
- `src-tauri/tests/fixtures/progress_v2.json`: fixed real-shape v2 migration fixture.
- `src/progress-view.ts`: exact footer copy and lifetime-output formatting.
- `src/frame-assets.ts`: rank/prestige asset resolution and the one-crest URL.
- `src/frame-renderer.ts`: perimeter-only DOM and CSS variable application.
- `src/main.ts`: live progress rendering and hover state.
- `src/preview.ts`: deterministic browser-only progress states.
- `index.html`, `src/styles.css`: stable footer track, crest placement, corner/rail geometry, and Reduced Motion.
- `scripts/normalize_prestige_corners.py`: chroma-clean canonical-corner normalization, validation, deterministic reflection, and atomic kit publication.
- `scripts/test_normalize_prestige_corners.py`: normalizer unit and rollback tests.
- `public/frames/prestige/{1..10}/corner-*.png`: regenerated connected corners.
- `src/generated-assets.test.ts`: production bitmap contact/reflection gates.
- `README.md`, `docs/images/mana-widget.png`: output-only behavior and final verified screenshot.

---

### Task 1: Implement Output-Only Scanning And Tiered Progression

**Files:**
- Modify: `src-tauri/src/progress.rs`

**Interfaces:**
- Produces:

```rust
pub fn xp_for_level(level: u32, prestige: u32) -> u64;
pub fn level_for_xp(xp: u64, prestige: u32) -> u32;
pub fn prestige_cycle_token_cost(prestige: u32) -> u64;
pub fn recalculate_from_output_history(state: &mut ProgressState);
pub fn try_prestige(state: &mut ProgressState) -> Result<(), String>;
```

- `Progress.lifetime_output_tokens` remains a Rust `u64` and uses a custom serializer that emits a decimal JSON string.
- During this task, `TallyState.total_tokens` and `codex_totals` retain their old internal names so the existing v2 store remains compilable. Task 2 performs the schema-safe persistent rename before any application build.

- [ ] **Step 1: Replace combined-token scanner expectations with failing output-only tests**

Replace the scanner fixtures and assertions in `progress.rs` tests with:

```rust
const CLAUDE_LINE: &str = r#"{"message":{"usage":{"input_tokens":10,"cache_creation_input_tokens":5,"cache_read_input_tokens":85,"output_tokens":100}}}"#;
const CODEX_EVENT: &str = r#"{"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":900,"output_tokens":3,"reasoning_output_tokens":4,"total_tokens":20000}}}}"#;

fn tally_test_dir(label: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "mana-output-{label}-{}",
        std::process::id(),
    ));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

fn append_line(path: &std::path::Path, line: &str) {
    use std::io::Write as _;
    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    writeln!(file, "{line}").unwrap();
}

#[test]
fn claude_scan_counts_output_only_and_is_idempotent() {
    let dir = tally_test_dir("claude");
    let file = dir.join("proj/session.jsonl");
    write(&file, &format!("{CLAUDE_LINE}\nnot json\n{CLAUDE_LINE}\n"));
    let mut state = TallyState::default();
    scan_claude_dir(&dir, &mut state);
    assert_eq!(state.total_tokens, 200);
    scan_claude_dir(&dir, &mut state);
    assert_eq!(state.total_tokens, 200);
}

#[test]
fn codex_scan_counts_output_and_reasoning_deltas_only() {
    let dir = tally_test_dir("codex");
    let file = dir.join("session.jsonl");
    write(&file, &format!("{CODEX_EVENT}\n"));
    let mut state = TallyState::default();
    scan_codex_dir(&dir, &mut state);
    assert_eq!(state.total_tokens, 7);
    let appended = CODEX_EVENT
        .replace("\"output_tokens\":3", "\"output_tokens\":30")
        .replace("\"reasoning_output_tokens\":4", "\"reasoning_output_tokens\":20");
    append_line(&file, &appended);
    scan_codex_dir(&dir, &mut state);
    assert_eq!(state.total_tokens, 50);
}
```

Add malformed, negative, non-integer, partial-line, idempotence, and shrunken-cumulative assertions. Negative JSON numbers must contribute zero because `as_u64()` returns `None`.

- [ ] **Step 2: Add failing exact curve, recalculation, and carryover tests**

```rust
#[test]
fn tiered_curve_matches_exact_thresholds_through_prestige_ten() {
    let expected = [
        800, 1_200, 1_800, 2_700, 4_725, 8_268, 14_470,
        28_940, 57_881, 115_762, 231_525,
    ];
    for (prestige, xp) in expected.into_iter().enumerate() {
        assert_eq!(xp_for_level(10, prestige as u32), xp);
    }
}

#[test]
fn recalculation_spends_complete_cycles_and_keeps_remainder() {
    let first = prestige_cycle_token_cost(0);
    let second = prestige_cycle_token_cost(1);
    let remainder = xp_for_level(10, 2) * TOKENS_PER_XP;
    let mut state = state_with(first + second + remainder, 13, 8, 99);
    recalculate_from_output_history(&mut state);
    assert_eq!((state.prestige, state.prestige_token_floor), (2, first + second));
    assert_eq!(progress_view(&state).level, 10);
    assert_eq!(state.rank, 2);
}

#[test]
fn prestige_spends_exact_cycle_cost_and_preserves_surplus() {
    let cost = prestige_cycle_token_cost(0);
    let surplus = xp_for_level(10, 1) * TOKENS_PER_XP;
    let mut state = state_with(cost + surplus, TIERS.len() - 1, 0, 0);
    try_prestige(&mut state).unwrap();
    assert_eq!((state.prestige, state.rank, state.prestige_token_floor), (1, 0, cost));
    assert_eq!(progress_view(&state).level, 10);
}

#[test]
fn prestige_rejects_final_rank_without_the_complete_cycle_cost() {
    let cost = prestige_cycle_token_cost(0);
    let mut state = state_with(cost - 1, TIERS.len() - 1, 0, 0);
    assert!(try_prestige(&mut state).is_err());
}
```

Also cover zero output, exact Level 120 output, multiple complete cycles, `u64::MAX`, and failed candidate persistence.

- [ ] **Step 3: Run the focused Rust tests to verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml progress::tests -- --nocapture
```

Expected: FAIL on output-only scanner totals, the old `1.5^p` curve, missing recalculation, and reset-to-now prestige behavior.

- [ ] **Step 4: Implement output-only parsers and exact rational curve math**

Replace the token parsers and curve function with:

```rust
fn claude_line_output_tokens(line: &str) -> u64 {
    serde_json::from_str::<serde_json::Value>(line)
        .ok()
        .and_then(|value| value.get("message")?.get("usage")?.get("output_tokens")?.as_u64())
        .unwrap_or(0)
}

fn codex_line_output_total(line: &str) -> Option<u64> {
    let value = serde_json::from_str::<serde_json::Value>(line).ok()?;
    let info = value
        .get("payload")
        .and_then(|payload| payload.get("info"))
        .or_else(|| value.get("info"))?;
    let usage = info.get("total_token_usage")?;
    let output = usage.get("output_tokens").and_then(|value| value.as_u64()).unwrap_or(0);
    let reasoning = usage
        .get("reasoning_output_tokens")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    Some(output.saturating_add(reasoning))
}

pub fn xp_for_level(level: u32, prestige: u32) -> u64 {
    if level <= 1 {
        return 0;
    }
    let first = prestige.min(3);
    let second = prestige.saturating_sub(3).min(3);
    let third = prestige.saturating_sub(6);
    let multiplier_numerator = 3u128
        .saturating_pow(first)
        .saturating_mul(7u128.saturating_pow(second))
        .saturating_mul(2u128.saturating_pow(third));
    let multiplier_denominator = 2u128
        .saturating_pow(first)
        .saturating_mul(4u128.saturating_pow(second));
    let level = u128::from(level);
    let numerator = 4u128
        .saturating_mul(level.saturating_pow(3))
        .saturating_mul(multiplier_numerator);
    let denominator = 5u128.saturating_mul(multiplier_denominator);
    u64::try_from(numerator / denominator).unwrap_or(u64::MAX)
}

pub fn prestige_cycle_token_cost(prestige: u32) -> u64 {
    xp_for_level(GATES[TIERS.len() - 1], prestige).saturating_mul(TOKENS_PER_XP)
}
```

Use `claude_line_output_tokens` in the Claude sum and `codex_line_output_total` for the latest Codex cumulative delta. Keep the existing complete-line and offset behavior unchanged.

- [ ] **Step 5: Implement deterministic recalculation, carryover, and exact serialization**

Add:

```rust
pub fn recalculate_from_output_history(state: &mut ProgressState) {
    let mut prestige = 0u32;
    let mut floor = 0u64;
    loop {
        let cost = prestige_cycle_token_cost(prestige);
        let remaining = state.tally.total_tokens.saturating_sub(floor);
        if cost == 0 || remaining < cost {
            break;
        }
        floor = floor.checked_add(cost).expect("affordability guarantees a u64 sum");
        prestige = prestige.saturating_add(1);
        if floor == u64::MAX {
            break;
        }
    }
    state.prestige = prestige;
    state.prestige_token_floor = floor;
    let level = level_for_xp(effective_xp(state), prestige);
    state.rank = GATES.iter().rposition(|gate| level >= *gate).unwrap_or(0);
    state.initialized = true;
}

fn serialize_u64_decimal<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}
```

Annotate the new public field:

```rust
#[serde(serialize_with = "serialize_u64_decimal")]
pub lifetime_output_tokens: u64,
```

Set it from the tally in `progress_view`. Change `try_prestige` to validate both final rank and `effective_output >= prestige_cycle_token_cost(current_prestige)`, increment the floor by that exact cost with checked arithmetic, then increment prestige and reset rank.

- [ ] **Step 6: Run focused and full Rust tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml progress::tests -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all Rust tests PASS.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/progress.rs
git commit -m "feat: count output-only prestige progression"
```

---

### Task 2: Add Schema V3 Decoding And Immutable V2 Recovery

**Files:**
- Modify: `src-tauri/src/progress.rs`
- Modify: `src-tauri/src/progress_store.rs`
- Create: `src-tauri/tests/fixtures/progress_v2.json`

**Interfaces:**
- Produces:

```rust
pub const SCHEMA_VERSION: u32 = 3;

pub struct TallyState {
    pub output_tokens: u64,
    pub claude_offsets: HashMap<String, u64>,
    pub codex_offsets: HashMap<String, u64>,
    pub codex_output_totals: HashMap<String, u64>,
}

pub struct LoadOutcome {
    pub state: ProgressState,
    pub source: RecoverySource,
    pub needs_output_rebuild: bool,
}
```

- `ProgressPaths` exposes distinct `pre_migration_v1` and `pre_migration_v2` paths.
- A decoded v3 document is immediately usable. A decoded v2 or unversioned document is validated, snapshotted in its original bytes, and returns a fresh v3 candidate with `needs_output_rebuild = true`.

- [ ] **Step 1: Freeze a complete v2 fixture**

Create `src-tauri/tests/fixtures/progress_v2.json` from the current schema-2 envelope, retaining all fields:

```json
{
  "schema_version": 2,
  "state": {
    "rank": 8,
    "prestige": 7,
    "prestige_token_floor": 123456,
    "initialized": true,
    "tally": {
      "total_tokens": 987654321,
      "claude_offsets": {"/fixture/claude.jsonl": 44},
      "codex_offsets": {"/fixture/codex.jsonl": 55},
      "codex_totals": {"/fixture/codex.jsonl": 66}
    }
  }
}
```

- [ ] **Step 2: Write failing v3 wire and migration tests**

Add tests that assert:

```rust
fn output_fixture_state() -> ProgressState {
    ProgressState {
        rank: 8,
        prestige: 7,
        prestige_token_floor: 123_456,
        initialized: true,
        tally: TallyState {
            output_tokens: 987_654_321,
            claude_offsets: std::collections::HashMap::from([
                ("/fixture/claude.jsonl".into(), 44),
            ]),
            codex_offsets: std::collections::HashMap::from([
                ("/fixture/codex.jsonl".into(), 55),
            ]),
            codex_output_totals: std::collections::HashMap::from([
                ("/fixture/codex.jsonl".into(), 66),
            ]),
        },
    }
}

#[test]
fn version_three_roundtrips_output_fields() {
    let original = output_fixture_state();
    let bytes = encode_state(&original).unwrap();
    let decoded = decode_state(&bytes).unwrap();
    assert_eq!(decoded, DecodedState::Current(original));
    assert_eq!(serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["schema_version"], 3);
}

#[test]
fn valid_v2_primary_returns_fresh_rebuild_without_replacing_primary() {
    let (_dir, paths) = test_paths("v2-rebuild");
    let v2 = include_bytes!("../tests/fixtures/progress_v2.json");
    std::fs::write(&paths.primary, v2).unwrap();
    let loaded = load_state(&paths).unwrap();
    assert!(loaded.needs_output_rebuild);
    assert_eq!(loaded.state, ProgressState::default());
    assert_eq!(std::fs::read(&paths.primary).unwrap(), v2);
    assert_eq!(std::fs::read(&paths.pre_migration_v2).unwrap(), v2);
}

#[test]
fn existing_valid_v2_recovery_is_never_overwritten() {
    let (_dir, paths) = test_paths("v2-immutable");
    let first = include_bytes!("../tests/fixtures/progress_v2.json");
    let mut second = serde_json::from_slice::<serde_json::Value>(first).unwrap();
    second["state"]["rank"] = 3.into();
    std::fs::write(&paths.pre_migration_v2, first).unwrap();
    std::fs::write(&paths.primary, serde_json::to_vec(&second).unwrap()).unwrap();
    load_state(&paths).unwrap();
    assert_eq!(std::fs::read(&paths.pre_migration_v2).unwrap(), first);
}
```

Also test invalid/unreadable/symlink recovery paths fail closed, v3 never rebuilds, incomplete v2 uses a valid backup, and future schemas remain rejected.

- [ ] **Step 3: Run the store tests to verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml progress_store::tests -- --nocapture
```

Expected: FAIL because schema 2 is still current, v2 output wire names do not exist, and only the v1 recovery path is modeled.

- [ ] **Step 4: Rename persistent tally fields and define schema-specific wire types**

Rename every internal reference to `output_tokens` and `codex_output_totals`. Keep separate strict wire structs:

```rust
#[derive(serde::Deserialize)]
struct ProgressEnvelopeV3Wire {
    schema_version: u32,
    state: ProgressStateV3Wire,
}

#[derive(serde::Deserialize)]
struct ProgressEnvelopeV2Wire {
    schema_version: u32,
    state: ProgressStateV2Wire,
}

#[derive(Debug, PartialEq)]
enum DecodedState {
    Current(ProgressState),
    NeedsOutputRebuild { source_schema: u32 },
}
```

`ProgressStateV3Wire` requires `output_tokens`, `claude_offsets`, `codex_offsets`, and `codex_output_totals`. `ProgressStateV2Wire` requires the original `total_tokens` and `codex_totals` fields but never maps those combined totals into output progression. Validate v2 rank and complete field presence, then return `NeedsOutputRebuild`.

- [ ] **Step 5: Publish schema-specific immutable snapshots**

Generalize the existing snapshot writer to:

```rust
fn write_immutable_snapshot(
    path: &Path,
    bytes: &[u8],
    expected_schema: SnapshotSchema,
) -> io::Result<()>;
```

`SnapshotSchema::V1` accepts only an unversioned valid document and writes `progress.pre-migration-v1.json`. `SnapshotSchema::V2` accepts only a complete schema-2 envelope and writes `progress.pre-migration-v2.json`. Both use create-new staging, `sync_all`, byte-for-byte staged validation, atomic hard-link publication, and parent-directory sync. Existing valid snapshots remain unchanged; invalid existing paths fail closed.

Update `load_state` so v2 and unversioned candidates return `ProgressState::default()` with `needs_output_rebuild = true` and never call `save_state`. Preserve the current recovery order for usable v3 primary, backup, and temporary candidates.

- [ ] **Step 6: Run Rust tests and inspect encoded keys**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml progress_store::tests -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all tests PASS. A v3 round trip contains `output_tokens` and `codex_output_totals`, and contains neither `total_tokens` nor `codex_totals`.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/progress.rs src-tauri/src/progress_store.rs src-tauri/tests/fixtures/progress_v2.json
git commit -m "feat: preserve v2 before output rebuild"
```

---

### Task 3: Publish The First Output Rebuild Atomically

**Files:**
- Modify: `src-tauri/src/progress.rs`
- Modify: `src-tauri/src/progress_store.rs`

**Interfaces:**
- Produces:

```rust
impl ProgressStore {
    pub(crate) fn output_rebuild_pending(&self) -> bool;
    pub(crate) fn finish_output_rebuild(&self);
}

pub fn publish_rebuilt_state(
    paths: &ProgressPaths,
    state: &ProgressState,
) -> std::io::Result<()>;
```

- `ProgressStore` keeps rebuild status in `AtomicBool`; it is never serialized.
- Rank-up and prestige commands reject with `"output history rebuild is still in progress"` until the flag clears.

- [ ] **Step 1: Write failing publication-boundary tests**

Add a migration-only checkpoint enum and tests for:

```rust
fn v2_paths(label: &str) -> (TestDirectory, ProgressPaths) {
    let (directory, paths) = test_paths(label);
    std::fs::write(
        &paths.primary,
        include_bytes!("../tests/fixtures/progress_v2.json"),
    )
    .unwrap();
    let loaded = load_state(&paths).unwrap();
    assert!(loaded.needs_output_rebuild);
    (directory, paths)
}

fn load_exact_v3(path: &std::path::Path) -> std::io::Result<ProgressState> {
    match decode_state(&std::fs::read(path)?)? {
        DecodedState::Current(state) => Ok(state),
        DecodedState::NeedsOutputRebuild { .. } => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "expected schema v3",
        )),
    }
}

#[test]
fn interrupted_rebuild_staging_keeps_v2_primary_authoritative() {
    let (_dir, paths) = v2_paths("rebuild-stage-failure");
    let v2 = std::fs::read(&paths.primary).unwrap();
    let rebuilt = output_fixture_state();
    publish_rebuilt_state_with_hook(&paths, &rebuilt, |checkpoint| {
        if checkpoint == RebuildCheckpoint::TemporarySynced {
            return Err(std::io::Error::other("simulated interruption"));
        }
        Ok(())
    }).unwrap_err();
    assert_eq!(std::fs::read(&paths.primary).unwrap(), v2);
    assert_eq!(std::fs::read(&paths.pre_migration_v2).unwrap(), v2);
}

#[test]
fn successful_rebuild_replaces_v2_with_complete_v3() {
    let (_dir, paths) = v2_paths("rebuild-success");
    let rebuilt = output_fixture_state();
    publish_rebuilt_state(&paths, &rebuilt).unwrap();
    assert_eq!(load_exact_v3(&paths.primary).unwrap(), rebuilt);
    assert!(load_state(&paths).unwrap().needs_output_rebuild == false);
}
```

Add a progress watcher helper test in `progress.rs` that begins with empty v3 cursors, scans retained Claude/Codex fixtures, calls `recalculate_from_output_history`, persists, and only then changes the current state and rebuild flag. A persistence error must preserve current state and leave the flag set.

- [ ] **Step 2: Run focused tests to verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml rebuild -- --nocapture
```

Expected: FAIL because the migration publisher and process-local flag do not exist.

- [ ] **Step 3: Implement migration-only atomic publication**

Implement `publish_rebuilt_state_with_hook` by:

1. Requiring a valid immutable snapshot matching the source being replaced:
   `progress.pre-migration-v2.json` for schema v2 or
   `progress.pre-migration-v1.json` for an unversioned legacy source.
2. Encoding and validating a complete v3 document.
3. Writing and syncing `progress.json.tmp`.
4. Atomically renaming the staged v3 file over `progress.json` without first rotating the v2 primary into the ordinary backup path.
5. Syncing the parent directory.
6. Treating an already-visible, byte-valid v3 primary as committed if the final directory-sync call reports an error after rename.

The immutable v2 recovery copy is the rollback source. Ordinary v3 saves continue using the existing primary-to-backup rotation.

- [ ] **Step 4: Integrate rebuild processing with the immediate watcher tick**

Extract a testable helper:

```rust
fn scan_and_commit_progress(
    current: &mut ProgressState,
    rebuild_pending: bool,
    claude_dir: &Path,
    codex_dir: &Path,
    persist_normal: impl FnOnce(&ProgressState) -> Result<(), String>,
    persist_rebuild: impl FnOnce(&ProgressState) -> Result<(), String>,
) -> Result<(Option<Progress>, bool), String>;
```

When pending, start from `ProgressState::default()`, scan both directories from zero, run `recalculate_from_output_history`, call `persist_rebuild`, then replace current state and return `rebuild_completed = true`. When not pending, preserve the existing incremental scan, new-install baseline, silent sub-XP save, and visible-event behavior.

In `spawn_progress_watcher`, clear the `AtomicBool` only after the helper returns success. Reject rank/prestige commands while pending so no ordinary save can replace the v2 source before the rebuild.

- [ ] **Step 5: Run all Rust tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected: all tests PASS, including retry after failed rebuild, idempotent v3 restart, exact retained-output recalculation, and unchanged live in-memory state on persistence failure.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/progress.rs src-tauri/src/progress_store.rs
git commit -m "feat: atomically publish output history rebuild"
```

---

### Task 4: Render One Exact Prestige Footer

**Files:**
- Modify: `index.html`
- Modify: `src/progress-view.ts`
- Modify: `src/progress-view.test.ts`
- Modify: `src/frame-assets.ts`
- Modify: `src/frame-assets.test.ts`
- Modify: `src/frame-renderer.ts`
- Modify: `src/frame-renderer.test.ts`
- Modify: `src/main.ts`
- Modify: `src/styles.css`
- Modify: `src/styles.test.ts`

**Interfaces:**
- Produces:

```ts
export type Progress = {
  xp: number;
  level: number;
  rank: number;
  tier: string;
  prestige: number;
  lifetime_output_tokens: string;
  rank_up_eligible: boolean;
  prestige_eligible: boolean;
  level_progress: { current: number; needed: number };
};

export function levelLabel(progress: Progress): string;
export function lifetimeOutputLabel(decimal: string): string;
```

- `frameRenderPlan` sets `--progress-prestige-crest` from the resolved prestige kit and uses only the rank crest for `--frame-crest`.

- [ ] **Step 1: Write failing copy and exact-format tests**

```ts
const base: Progress = {
  xp: 850,
  level: 10,
  rank: 5,
  tier: "silver",
  prestige: 7,
  lifetime_output_tokens: "18446744073709551615",
  rank_up_eligible: false,
  prestige_eligible: false,
  level_progress: { current: 50, needed: 125 },
};

it("adds current prestige to the rest label", () => {
  expect(levelLabel(base)).toBe("Lv 10 · Silver · Prestige VII");
});

it("formats the full u64 decimal string exactly", () => {
  expect(lifetimeOutputLabel(base.lifetime_output_tokens)).toBe(
    "18,446,744,073,709,551,615 lifetime output",
  );
  expect(lifetimeOutputLabel("001")).toBe("0 lifetime output");
});
```

Change prestige ceremony copy to avoid promising Level 1:

```ts
expect(dialogCopy("prestige", base).body).toBe(
  "The curve steepens. Surplus output carries forward into Prestige 8.",
);
```

- [ ] **Step 2: Write failing one-crest perimeter tests**

Assert `frameLayerHtml()` contains four rails, four corners, four ornament lanes, at most one rank frame crest, and no `data-prestige-text` or `.frame-prestige-text`. Assert:

```ts
const plan = frameRenderPlan(decoration());
expect(plan.cssVariables["--frame-crest"]).toBe('url("/rank/crest-top.png")');
expect(plan.cssVariables["--progress-prestige-crest"]).toBe(
  'url("/prestige/crest-top.png")',
);
```

Applying Prestige 0 must set the host `data-prestige-crest="false"`; applying Prestige X must set it to `"true"`. Reapplying must not accumulate nodes.

- [ ] **Step 3: Run focused frontend tests to verify RED**

Run:

```bash
npx vitest run src/progress-view.test.ts src/frame-assets.test.ts src/frame-renderer.test.ts src/styles.test.ts
```

Expected: FAIL on the missing lifetime field/formatter, top plaque markup, and missing footer crest variables.

- [ ] **Step 4: Implement exact labels and the single crest data flow**

Use string-only validation and formatting:

```ts
const UNSIGNED_DECIMAL = /^(0|[1-9]\d*)$/;

export function lifetimeOutputLabel(decimal: string): string {
  if (!UNSIGNED_DECIMAL.test(decimal)) return "0 lifetime output";
  return `${decimal.replace(/\B(?=(\d{3})+(?!\d))/g, ",")} lifetime output`;
}

export function levelLabel(progress: Progress): string {
  const base = `Lv ${progress.level} · ${tierDisplayName(progress.tier)}`;
  return progress.prestige > 0
    ? `${base} · Prestige ${romanNumeral(progress.prestige)}`
    : base;
}
```

Remove `prestigeText` from `ResolvedFrameDecoration` and delete the generic numeral-plaque path. Keep the generated `crest-top.png` on the resolved prestige model. In `frameRenderPlan`, map the rank crest to `--frame-crest` and prestige crest to `--progress-prestige-crest`. In `applyFrameDecoration`, set `perimeter.parentElement.dataset.prestigeCrest` from `model.prestige !== null`.

- [ ] **Step 5: Add stable footer markup and hover rendering**

Replace the footer markup with:

```html
<div id="progress">
  <span class="prestige-crest" aria-hidden="true"></span>
  <span class="progress-copy">
    <span class="level"></span>
    <span class="lifetime-output"></span>
  </span>
  <div class="xpbar"><div class="xpfill"></div></div>
</div>
```

In `renderProgress`, populate both labels once. In body enter/leave handlers, set `root.dataset.hovering = String(hovering)` in addition to updating sprites. CSS swaps the two text spans:

```css
#progress .prestige-crest {
  display: none;
  flex: 0 0 40px;
  width: 40px;
  height: 20px;
  background: var(--progress-prestige-crest, none) center / 40px 20px no-repeat;
  image-rendering: pixelated;
}

#root[data-prestige-crest="true"] #progress .prestige-crest {
  display: block;
}

#progress .progress-copy {
  flex: 0 1 228px;
  width: 228px;
  min-width: 0;
  overflow: hidden;
  white-space: nowrap;
}

#progress .lifetime-output,
#root[data-hovering="true"] #progress .level {
  display: none;
}

#root[data-hovering="true"] #progress .lifetime-output {
  display: inline;
}
```

Keep tabular numerals and text overflow ellipsis. Do not resize the footer, content, or window on hover.

- [ ] **Step 6: Remove the prestige top plaque and preserve Reduced Motion**

Delete `.frame-prestige-text` CSS. Keep `.frame-crest` only for rank art. Ensure the current rail/corner masked animations remain covered by `prefers-reduced-motion: reduce`; the footer crest itself does not introduce a new animation.

- [ ] **Step 7: Run frontend tests and build**

Run:

```bash
npx vitest run src/progress-view.test.ts src/frame-assets.test.ts src/frame-renderer.test.ts src/styles.test.ts
npm run build
```

Expected: all focused tests PASS and TypeScript/Vite build succeeds.

- [ ] **Step 8: Commit**

```bash
git add index.html src/progress-view.ts src/progress-view.test.ts src/frame-assets.ts src/frame-assets.test.ts src/frame-renderer.ts src/frame-renderer.test.ts src/main.ts src/styles.css src/styles.test.ts
git commit -m "feat: show one exact prestige footer"
```

---

### Task 5: Add Deterministic Connected-Corner Tooling

**Files:**
- Create: `scripts/normalize_prestige_corners.py`
- Create: `scripts/test_normalize_prestige_corners.py`
- Modify: `src/generated-assets.test.ts`

**Interfaces:**
- Produces:

```python
def normalize_top_left_corner(source: Image.Image) -> Image.Image;
def validate_top_left_corner(corner: Image.Image) -> None;
def reflected_corners(top_left: Image.Image) -> dict[str, Image.Image];
def publish_prestige_corners(
    source: Image.Image,
    prestige: int,
    output_root: Path,
) -> Path;
```

- CLI:

```text
python3 scripts/normalize_prestige_corners.py \
  --prestige 1 --input tmp/generated/prestige-corners/1-alpha.png \
  --output-root public/frames
```

- [ ] **Step 1: Write failing normalizer tests**

Use a synthetic asymmetric L joint and assert:

```python
def make_connected_fixture() -> Image.Image:
    image = Image.new("RGBA", (64, 64), (0, 0, 0, 0))
    draw = ImageDraw.Draw(image)
    draw.rectangle((12, 18, 63, 34), fill=(245, 205, 95, 255))
    draw.rectangle((18, 12, 34, 63), fill=(245, 205, 95, 255))
    draw.polygon(((18, 18), (38, 18), (46, 30), (30, 46), (18, 38)), fill=(150, 210, 255, 255))
    draw.point((13, 19), fill=(255, 255, 255, 15))
    return image

def test_normalized_top_left_keeps_exterior_clear_and_sockets_connected(self):
    corner = normalize_top_left_corner(make_connected_fixture())
    self.assertEqual(corner.size, (96, 96))
    alpha = corner.getchannel("A")
    self.assertIsNone(alpha.crop((0, 0, 4, 96)).getbbox())
    self.assertIsNone(alpha.crop((0, 0, 96, 4)).getbbox())
    self.assertIsNotNone(alpha.crop((88, 28, 96, 68)).getbbox())
    self.assertIsNotNone(alpha.crop((28, 88, 68, 96)).getbbox())

def test_other_corners_are_exact_reflections(self):
    corners = reflected_corners(normalize_top_left_corner(make_connected_fixture()))
    self.assertEqual(
        corners["corner-tr"].tobytes(),
        corners["corner-tl"].transpose(Image.Transpose.FLIP_LEFT_RIGHT).tobytes(),
    )
    self.assertEqual(
        corners["corner-bl"].tobytes(),
        corners["corner-tl"].transpose(Image.Transpose.FLIP_TOP_BOTTOM).tobytes(),
    )
```

Also assert the publisher preserves existing `rail-h.png`, `rail-v.png`, and `crest-top.png` bytes, removes no kit files, rejects disconnected source art, rejects prestige outside 1-10, and restores the complete destination after a simulated late-save failure.

- [ ] **Step 2: Run Python tests to verify RED**

Run:

```bash
python3 -m unittest scripts/test_normalize_prestige_corners.py -v
```

Expected: FAIL because the focused corner normalizer does not exist.

- [ ] **Step 3: Implement normalization, reflection, and atomic publication**

Normalize the visible source with nearest-neighbor resampling into a `96x96` canvas. Preserve four transparent pixels on the top and left exterior edges, anchor the visible bounding box toward the bottom-right connection edges, and validate visible alpha in both socket windows. Derive the other three corners only with Pillow `FLIP_LEFT_RIGHT` and `FLIP_TOP_BOTTOM`.

Stage by copying the existing complete prestige directory, replace only its four corner PNGs, validate all staged files, then atomically swap the staged directory using the rollback pattern already implemented in `normalize_frame_art.py`.

- [ ] **Step 4: Strengthen production bitmap gates**

In `generated-assets.test.ts`, stop requiring all four edges of prestige corners to be transparent. For each Prestige I-X assert:

```ts
function exteriorBandAlpha(
  image: DecodedPng,
  side: "top" | "left",
  width: number,
): number {
  let maximum = 0;
  const xLimit = side === "left" ? width : image.width;
  const yLimit = side === "top" ? width : image.height;
  for (let y = 0; y < yLimit; y += 1) {
    for (let x = 0; x < xLimit; x += 1) {
      maximum = Math.max(maximum, image.pixels[(y * image.width + x) * 4 + 3]);
    }
  }
  return maximum;
}

function socketVisible(
  image: DecodedPng,
  box: { x: number; y: number; width: number; height: number },
): boolean {
  for (let y = box.y; y < box.y + box.height; y += 1) {
    for (let x = box.x; x < box.x + box.width; x += 1) {
      if (image.pixels[(y * image.width + x) * 4 + 3] > 16) return true;
    }
  }
  return false;
}

function reflectPixels(
  image: DecodedPng,
  horizontal: boolean,
  vertical: boolean,
): Uint8Array {
  const reflected = new Uint8Array(image.pixels.length);
  for (let y = 0; y < image.height; y += 1) {
    for (let x = 0; x < image.width; x += 1) {
      const sourceX = horizontal ? image.width - 1 - x : x;
      const sourceY = vertical ? image.height - 1 - y : y;
      const source = (sourceY * image.width + sourceX) * 4;
      reflected.set(image.pixels.subarray(source, source + 4), (y * image.width + x) * 4);
    }
  }
  return reflected;
}

expect(exteriorBandAlpha(topLeft, "top", 4)).toBe(0);
expect(exteriorBandAlpha(topLeft, "left", 4)).toBe(0);
expect(socketVisible(topLeft, { x: 88, y: 28, width: 8, height: 40 })).toBe(true);
expect(socketVisible(topLeft, { x: 28, y: 88, width: 40, height: 8 })).toBe(true);
expect(topRight.pixels).toEqual(reflectPixels(topLeft, true, false));
expect(bottomLeft.pixels).toEqual(reflectPixels(topLeft, false, true));
expect(bottomRight.pixels).toEqual(reflectPixels(topLeft, true, true));
```

Rank-corner gates remain unchanged.

- [ ] **Step 5: Run tooling tests**

Run:

```bash
python3 -m unittest scripts/test_normalize_prestige_corners.py -v
npx vitest run src/generated-assets.test.ts
```

Expected: Python tests PASS; the production asset test remains RED only because old prestige corners fail the new socket/reflection contract.

- [ ] **Step 6: Commit tooling and failing production gate**

```bash
git add scripts/normalize_prestige_corners.py scripts/test_normalize_prestige_corners.py src/generated-assets.test.ts
git commit -m "test: require connected prestige corners"
```

---

### Task 6: Generate Prestige I-X Connected Corners

**Files:**
- Replace: `public/frames/prestige/1/corner-*.png`
- Replace: `public/frames/prestige/2/corner-*.png`
- Replace: `public/frames/prestige/3/corner-*.png`
- Replace: `public/frames/prestige/4/corner-*.png`
- Replace: `public/frames/prestige/5/corner-*.png`
- Replace: `public/frames/prestige/6/corner-*.png`
- Replace: `public/frames/prestige/7/corner-*.png`
- Replace: `public/frames/prestige/8/corner-*.png`
- Replace: `public/frames/prestige/9/corner-*.png`
- Replace: `public/frames/prestige/10/corner-*.png`

**Interfaces:**
- Consumes: the existing same-level `rail-h.png`, `rail-v.png`, and `crest-top.png` as visual references.
- Produces: ten canonical generated top-left joints plus thirty deterministic reflections satisfying Task 5.

- [ ] **Step 1: Inspect the current material references**

For each prestige directory, make a temporary reference strip containing its horizontal rail, vertical rail, and crest. Inspect Prestige I, IV, VII, IX, and X strips with `view_image` before generation so prompts preserve their material and escalation.

```bash
python3 - <<'PY'
from pathlib import Path
from PIL import Image, ImageDraw

output = Path("tmp/generated/prestige-corners/references")
output.mkdir(parents=True, exist_ok=True)
for level in range(1, 11):
    root = Path(f"public/frames/prestige/{level}")
    canvas = Image.new("RGBA", (448, 160), (10, 12, 22, 255))
    rail_h = Image.open(root / "rail-h.png").convert("RGBA")
    rail_v = Image.open(root / "rail-v.png").convert("RGBA")
    crest = Image.open(root / "crest-top.png").convert("RGBA")
    canvas.alpha_composite(rail_h, (16, 24))
    canvas.alpha_composite(rail_v, (176, 16))
    canvas.alpha_composite(crest, (224, 16))
    ImageDraw.Draw(canvas).text((16, 136), f"Prestige {level}: rail-h, rail-v, crest", fill="white")
    canvas.save(output / f"{level}.png")
PY
```

- [ ] **Step 2: Generate one canonical top-left corner per prestige with built-in GPT Image 2**

Issue one built-in `image_gen` call per prestige using the same-level strip as a reference and this exact base prompt:

```text
Use case: stylized-concept
Asset type: canonical top-left application-frame corner sprite
Primary request: create one connected 90-degree fantasy MMORPG frame joint matching the supplied prestige rail and crest material.
Subject: one top-left L joint with a horizontal rail socket exiting exactly to the right and a vertical rail socket exiting exactly downward; the elbow ornament is physically mounted over the joint.
Style/medium: polished early-2000s Korean fantasy MMORPG pixel art, crisp chunky pixel clusters, dark navy outline, orthographic UI view, no antialias blur.
Composition/framing: one isolated corner centered with generous padding; both rail sockets are fully visible and reach their respective inner connection edges.
Scene/backdrop: perfectly flat solid #00ff00 chroma-key background with no shadow, gradient, texture, or lighting variation.
Constraints: no complete frame, no detached starburst, no floating gem, no loose fragments, no text, no numerals, no duplicate rail ends, no cast shadow, no perspective, no content cropped by the canvas, and no #00ff00 in the art.
```

Tier additions:

- I-III: compact silver-gold joint, one restrained diamond, minimal rays.
- IV-VI: broader gold shoulders, clearer mounted crystal, moderate celestial engraving.
- VII: prismatic violet channel and one compact wing pair.
- VIII: brighter cyan-violet crystal facets with a stronger socket collar.
- IX: alternating celestial gemstones and refined gold rays without detached pieces.
- X: apex white-gold/prismatic joint, radiant central diamond and compact symmetric wings, brightest material while sockets remain the dominant structural exits.

Copy each selected generated source into `tmp/generated/prestige-corners/<n>-source.png`.

- [ ] **Step 3: Remove chroma and normalize each canonical corner**

For every level:

```bash
python "${CODEX_HOME:-$HOME/.codex}/skills/.system/imagegen/scripts/remove_chroma_key.py" \
  --input tmp/generated/prestige-corners/1-source.png \
  --out tmp/generated/prestige-corners/1-alpha.png \
  --auto-key border --soft-matte --transparent-threshold 12 \
  --opaque-threshold 220 --despill

python3 scripts/normalize_prestige_corners.py \
  --prestige 1 \
  --input tmp/generated/prestige-corners/1-alpha.png \
  --output-root public/frames
```

Repeat through Prestige X. If chroma fringe remains, retry that source once with `--edge-contract 1`. If either socket validator fails, regenerate that prestige rather than drawing a code-native substitute.

- [ ] **Step 4: Run bitmap gates and inspect a corner contact sheet**

Run:

```bash
npx vitest run src/generated-assets.test.ts
```

Expected: PASS for all ten connected/reflected corner sets and every unchanged rail/crest.

Create `tmp/generated/prestige-corners/contact-sheet.png` showing TL/TR/BL/BR for I-X at both `96x96` and `48x48`, on the actual dark glass color. Inspect it with `view_image`. Reject cropped ornaments, detached starbursts, uneven reflections, green fringe, weak sockets, or non-escalating VII-X art.

```bash
python3 - <<'PY'
from pathlib import Path
from PIL import Image, ImageDraw

sheet = Image.new("RGBA", (640, 10 * 128), (29, 34, 47, 255))
draw = ImageDraw.Draw(sheet)
names = ("corner-tl", "corner-tr", "corner-bl", "corner-br")
for row, level in enumerate(range(1, 11)):
    draw.text((8, row * 128 + 8), f"P{level}", fill="white")
    for column, name in enumerate(names):
        image = Image.open(f"public/frames/prestige/{level}/{name}.png").convert("RGBA")
        sheet.alpha_composite(image, (48 + column * 112, row * 128 + 8))
        small = image.resize((48, 48), Image.Resampling.NEAREST)
        sheet.alpha_composite(small, (496 + (column % 2) * 56, row * 128 + 8 + (column // 2) * 56))
Path("tmp/generated/prestige-corners").mkdir(parents=True, exist_ok=True)
sheet.save("tmp/generated/prestige-corners/contact-sheet.png")
PY
```

- [ ] **Step 5: Commit production corners**

```bash
git add public/frames/prestige
git commit -m "feat: add connected generated prestige corners"
```

---

### Task 7: Add Deterministic Preview States And Correct Rail Underlap

**Files:**
- Modify: `src/preview.ts`
- Modify: `src/preview.test.ts`
- Modify: `src/styles.css`
- Modify: `src/styles.test.ts`

**Interfaces:**
- `PreviewOptions` adds:

```ts
outputTokens: string;
hovering: boolean;
```

- Query values:

```text
outputTokens=<unsigned-u64-decimal>
hover=true|false
```

- [ ] **Step 1: Write failing preview and geometry tests**

```ts
expect(
  parsePreviewOptions(
    new URLSearchParams(
      "rank=godlike&prestige=10&outputTokens=18446744073709551615&hover=true",
    ),
  ),
).toMatchObject({
  outputTokens: "18446744073709551615",
  hovering: true,
});

expect(
  parsePreviewOptions(new URLSearchParams("outputTokens=18446744073709551616")),
).toMatchObject({ outputTokens: "12345678" });
```

In styles tests, require horizontal rails to use `left: 40px; right: 40px`, vertical rails to use `top: 40px; bottom: 40px`, and corners to remain `48x48` above the rails.

- [ ] **Step 2: Run focused tests to verify RED**

Run:

```bash
npx vitest run src/preview.test.ts src/styles.test.ts
```

Expected: FAIL on missing output/hover options and current four-pixel underlap geometry.

- [ ] **Step 3: Implement safe preview parsing**

Validate the output query as a decimal string using `BigInt` only for the `0..u64::MAX` range check; never convert it to `number`. Default to `"12345678"`. Set `root.dataset.hovering` from the preview option and populate both footer labels using the same production helpers.

- [ ] **Step 4: Apply the eight-pixel rail underlap**

Update:

```css
.frame-rail--top,
.frame-rail--bottom {
  right: 40px;
  left: 40px;
}

.frame-rail--right,
.frame-rail--left {
  top: 40px;
  bottom: 40px;
}
```

Do not alter rail thickness, tile size, corner size, perimeter bleed, or glass inset.

- [ ] **Step 5: Run frontend tests and build**

Run:

```bash
npx vitest run src/preview.test.ts src/styles.test.ts src/generated-assets.test.ts
npm run build
```

Expected: all tests PASS and the browser preview build succeeds.

- [ ] **Step 6: Commit**

```bash
git add src/preview.ts src/preview.test.ts src/styles.css src/styles.test.ts
git commit -m "feat: preview exact prestige footer states"
```

---

### Task 8: Verify Visual Matrix, Documentation, And Build-Only App

**Files:**
- Modify: `README.md`
- Replace: `docs/images/mana-widget.png`

**Interfaces:**
- Produces a verified browser matrix and a build-only `src-tauri/target/release/bundle/macos/Mana.app`.
- Does not install, launch, or replace the currently running Mana application.

- [ ] **Step 1: Run the full automated suite**

Run:

```bash
python3 -m unittest scripts/test_normalize_frame_art.py scripts/test_normalize_prestige_corners.py -v
npm test
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
git diff --check
```

Expected: all Python, Vitest, TypeScript/Vite, and Rust checks PASS with no whitespace errors.

- [ ] **Step 2: Start one browser-only preview server**

Run:

```bash
npm run dev -- --host 127.0.0.1
```

Record the actual Vite URL. Do not open or launch the Tauri application.

- [ ] **Step 3: Inspect the required visual matrix with Playwright**

Capture desktop screenshots for:

```text
preview.html?rank=godlike&prestige=1&providers=both&outputTokens=12345678
preview.html?rank=godlike&prestige=4&providers=both&outputTokens=12345678
preview.html?rank=godlike&prestige=7&providers=both&outputTokens=12345678
preview.html?rank=godlike&prestige=9&providers=both&outputTokens=12345678
preview.html?rank=godlike&prestige=10&providers=both&outputTokens=18446744073709551615
preview.html?rank=godlike&prestige=10&providers=both&outputTokens=18446744073709551615&hover=true
preview.html?rank=godlike&prestige=10&providers=claude&motion=reduced
preview.html?rank=godlike&prestige=10&providers=codex&motion=reduced
```

For each screenshot verify:

- all four corner sockets overlap their rail by eight CSS pixels;
- no detached or duplicated corner fragment exists;
- the perimeter surrounds the glass and does not appear inside it;
- no top-center prestige plaque or numeral remains;
- exactly one generated current-prestige crest appears beside the footer;
- rest and hover copy stay on one line and the XP bar/root bounds do not move;
- Prestige VII, IX, and X escalate without making the content unreadable;
- Reduced Motion freezes rail/corner flashes while geometry remains identical;
- Claude-only and Codex-only layouts retain correct frame and footer alignment.

Use DOM bounding-box assertions for `#root`, `#glass`, `#progress`, `.progress-copy`, `.xpbar`, four `.frame-corner` nodes, and four `.frame-rail` nodes before and after hover. Compare canvas/screenshot pixels to ensure the frame and generated crest are nonblank.

- [ ] **Step 4: Update README behavior and screenshot**

Document:

- progression counts only Claude output and Codex output plus reasoning output;
- input and cached tokens do not count;
- upgrading to v3 recalculates from retained local logs and preserves an immutable v2 recovery file;
- each prestige tier becomes progressively harder;
- surplus output carries forward;
- hover shows exact retained lifetime output;
- preview accepts `outputTokens` and `hover`.

Replace `docs/images/mana-widget.png` with the verified Prestige X rest screenshot. Do not include browser chrome.

- [ ] **Step 5: Build the macOS app without launching it**

Stop the Vite server, then run:

```bash
npm run tauri build -- --bundles app
```

Expected: build succeeds and produces:

```text
src-tauri/target/release/bundle/macos/Mana.app
```

Confirm `Info.plist` still has `LSUIElement=true` and no Dock activation regression. Do not copy the bundle into `/Applications` and do not open it.

- [ ] **Step 6: Commit documentation and verified screenshot**

```bash
git add README.md docs/images/mana-widget.png
git commit -m "docs: explain output-only prestige progression"
```

- [ ] **Step 7: Record final evidence**

Run:

```bash
git status --short --branch
git log --oneline -8
shasum -a 256 \
  "$HOME/Library/Application Support/com.vantasoft.mana/progress.manual-before-v2-20260722-195941.json"
```

Expected: only intentional generated preview scratch files remain untracked or the tree is clean; task commits are visible; the existing manual live-progress backup still hashes to:

```text
49169ce0799c431fa44ca78571a62385429af389597f41d3428523e0b4689f19
```

Do not inspect, rewrite, or migrate the live primary progress file during this verification.
