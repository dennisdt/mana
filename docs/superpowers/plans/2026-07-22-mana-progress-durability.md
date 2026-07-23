# Mana Progress Durability Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Preserve every valid existing Mana progression field across application updates, interrupted writes, and recoverable file corruption.

**Architecture:** A new `progress_store.rs` module owns versioned serialization, legacy migration, backup recovery, and atomic filesystem replacement. `progress.rs` continues to own progression math and scanning, but commands and the watcher commit cloned candidate states through `ProgressStore` before publishing them.

**Tech Stack:** Rust 2021, Tauri v2, serde/serde_json, standard-library filesystem APIs, cargo test. No new crate dependency.

## Global Constraints

- Keep the Tauri identifier exactly `com.vantasoft.mana`.
- Keep the primary path at `app_data_dir()/progress.json`.
- Preserve `rank`, `prestige`, `prestige_token_floor`, `initialized`, `tally.total_tokens`, `claude_offsets`, `codex_offsets`, and `codex_totals`.
- Any successfully parsed legacy file is treated as initialized; only a genuinely new install starts uninitialized.
- Never replace existing unreadable progress files with a default state.
- `progress.pre-migration-v1.json` is immutable once created.
- Rank-up, prestige, and scanner mutations become visible only after persistence succeeds.
- Do not change XP math, rank gates, prestige eligibility, or token parsing.
- Use TDD and commit after each independently passing task.

---

### Task 1: Capture and validate the currently running app's progress

**Files:**
- Read: `$HOME/Library/Application Support/com.vantasoft.mana/progress.json`
- Create outside git: `$HOME/Library/Application Support/com.vantasoft.mana/progress.manual-before-v2-YYYYMMDD-HHMMSS.json`

**Interfaces:**
- Produces: one validated byte-for-byte manual recovery copy created before any persistence code runs.

- [ ] **Step 1: Locate and validate the current primary**

Run:

```bash
APP_DIR="$HOME/Library/Application Support/com.vantasoft.mana"
test -f "$APP_DIR/progress.json"
python3 -m json.tool "$APP_DIR/progress.json" >/dev/null
```

Expected: both commands exit 0. If the file does not exist, record that this is a genuinely new install and skip only the copy step.

- [ ] **Step 2: Create a uniquely named copy without overwriting anything**

Run:

```bash
APP_DIR="$HOME/Library/Application Support/com.vantasoft.mana"
STAMP="$(date +%Y%m%d-%H%M%S)"
BACKUP="$APP_DIR/progress.manual-before-v2-$STAMP.json"
cp -p "$APP_DIR/progress.json" "$BACKUP"
python3 -m json.tool "$BACKUP" >/dev/null
shasum -a 256 "$APP_DIR/progress.json" "$BACKUP"
```

Expected: validation exits 0 and both SHA-256 values match. Do not add the file to git or print its contents.

- [ ] **Step 3: Record only non-sensitive invariants for later comparison**

Run:

```bash
APP_DIR="$HOME/Library/Application Support/com.vantasoft.mana"
BACKUP="$(ls -t "$APP_DIR"/progress.manual-before-v2-*.json | head -1)"
python3 - "$BACKUP" <<'PY'
import json, sys
state = json.load(open(sys.argv[1]))
tally = state.get("tally", {})
print({
    "rank": state.get("rank"),
    "prestige": state.get("prestige"),
    "prestige_token_floor": state.get("prestige_token_floor"),
    "initialized": state.get("initialized"),
    "total_tokens": tally.get("total_tokens"),
    "claude_cursor_count": len(tally.get("claude_offsets", {})),
    "codex_cursor_count": len(tally.get("codex_offsets", {})),
    "codex_total_count": len(tally.get("codex_totals", {})),
})
PY
```

Expected: a compact invariant summary suitable for comparing after migration.

---

### Task 2: Add versioned state parsing and lossless legacy migration

**Files:**
- Create: `src-tauri/src/progress_store.rs`
- Create: `src-tauri/tests/fixtures/progress_v1.json`
- Create: `src-tauri/tests/fixtures/progress_v1_early.json`
- Modify: `src-tauri/src/progress.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/progress_store.rs`

**Interfaces:**
- Consumes: `crate::progress::ProgressState`.
- Produces:

```rust
pub const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgressEnvelope {
    pub schema_version: u32,
    pub state: ProgressState,
}

pub fn encode_state(state: &ProgressState) -> std::io::Result<Vec<u8>>;
pub fn decode_state(bytes: &[u8]) -> std::io::Result<(ProgressState, bool)>;
```

The boolean returned by `decode_state` is `true` only when unversioned legacy bytes were migrated.

- [ ] **Step 1: Add an exact current-schema fixture**

Create `src-tauri/tests/fixtures/progress_v1.json`:

```json
{
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
```

Create `src-tauri/tests/fixtures/progress_v1_early.json` without the later
`initialized`, `codex_offsets`, and `codex_totals` fields:

```json
{
  "rank": 5,
  "prestige": 2,
  "prestige_token_floor": 7,
  "tally": {
    "total_tokens": 42,
    "claude_offsets": {"/fixture/early-claude.jsonl": 11}
  }
}
```

- [ ] **Step 2: Write failing migration and envelope tests**

Add tests in `progress_store.rs`:

```rust
fn fixture_state() -> ProgressState {
    serde_json::from_slice(include_bytes!("../tests/fixtures/progress_v1.json")).unwrap()
}

#[test]
fn migrates_current_unversioned_state_without_changing_progress() {
    let bytes = include_bytes!("../tests/fixtures/progress_v1.json");
    let (state, migrated) = decode_state(bytes).unwrap();
    assert!(migrated);
    assert_eq!((state.rank, state.prestige), (8, 7));
    assert_eq!(state.prestige_token_floor, 123456);
    assert!(state.initialized);
    assert_eq!(state.tally.total_tokens, 987654321);
    assert_eq!(state.tally.claude_offsets["/fixture/claude.jsonl"], 44);
    assert_eq!(state.tally.codex_offsets["/fixture/codex.jsonl"], 55);
    assert_eq!(state.tally.codex_totals["/fixture/codex.jsonl"], 66);
}

#[test]
fn legacy_state_without_initialized_is_treated_as_initialized() {
    let bytes = include_bytes!("../tests/fixtures/progress_v1_early.json");
    let (state, migrated) = decode_state(bytes).unwrap();
    assert!(migrated);
    assert!(state.initialized);
    assert_eq!((state.rank, state.prestige), (5, 2));
    assert_eq!(state.tally.claude_offsets["/fixture/early-claude.jsonl"], 11);
    assert!(state.tally.codex_offsets.is_empty());
    assert!(state.tally.codex_totals.is_empty());
}

#[test]
fn version_two_roundtrips_every_field() {
    let original = fixture_state();
    let bytes = encode_state(&original).unwrap();
    let (decoded, migrated) = decode_state(&bytes).unwrap();
    assert!(!migrated);
    assert_eq!(decoded, original);
}

#[test]
fn rejects_unknown_future_schema() {
    let error = decode_state(br#"{"schema_version":99,"state":{}}"#).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn rejects_rank_outside_the_known_tier_table() {
    let mut invalid = fixture_state();
    invalid.rank = crate::progress::TIERS.len();
    let bytes = serde_json::to_vec(&ProgressEnvelope {
        schema_version: SCHEMA_VERSION,
        state: invalid,
    }).unwrap();
    let error = decode_state(&bytes).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}
```

- [ ] **Step 3: Run the focused test and verify RED**

Run: `cd src-tauri && cargo test progress_store`

Expected: FAIL because the module and functions do not exist.

- [ ] **Step 4: Implement versioned and legacy decoding**

Add `pub mod progress_store;` to `lib.rs`. In `progress_store.rs`, deserialize through an untagged wire enum and a dedicated legacy type whose missing `initialized` field defaults to `true`:

```rust
#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ProgressDocument {
    Versioned(ProgressEnvelope),
    Legacy(LegacyProgressState),
}

fn legacy_initialized() -> bool { true }

#[derive(serde::Deserialize)]
struct LegacyProgressState {
    rank: usize,
    prestige: u32,
    prestige_token_floor: u64,
    #[serde(default = "legacy_initialized")]
    initialized: bool,
    tally: crate::progress::TallyState,
}
```

Add `#[serde(default)]` to `TallyState` in `progress.rs` so cursor maps added
after the earliest unversioned files deserialize as empty maps instead of
invalidating an otherwise recoverable state.

`decode_state` must reject a versioned document whose `schema_version` is not
`2` and reject `state.rank >= TIERS.len()`. Do not reject historical token
floors or cursor relationships merely because they look unusual; preserving a
successfully written state is safer than inventing a new invariant.
`encode_state` always emits
`ProgressEnvelope { schema_version: 2, state: state.clone() }`.

- [ ] **Step 5: Run the focused test and verify GREEN**

Run: `cd src-tauri && cargo test progress_store`

Expected: all migration and envelope tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/progress.rs src-tauri/src/progress_store.rs src-tauri/tests/fixtures/progress_v1.json src-tauri/tests/fixtures/progress_v1_early.json
git commit -m "feat: add lossless progress schema migration"
```

---

### Task 3: Add recovery paths and immutable legacy snapshots

**Files:**
- Modify: `src-tauri/src/progress_store.rs`
- Test: `src-tauri/src/progress_store.rs`

**Interfaces:**
- Produces:

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressPaths {
    pub primary: std::path::PathBuf,
    pub backup: std::path::PathBuf,
    pub pre_migration: std::path::PathBuf,
    pub temporary: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverySource { Primary, Backup, PreMigration, New }

#[derive(Debug)]
pub struct LoadOutcome {
    pub state: ProgressState,
    pub source: RecoverySource,
}

impl ProgressPaths {
    pub fn from_primary(primary: std::path::PathBuf) -> Self;
}

pub fn load_state(paths: &ProgressPaths) -> std::io::Result<LoadOutcome>;
```

- [ ] **Step 1: Write failing recovery-order tests**

Add an isolated no-dependency test directory helper:

```rust
static TEST_DIRECTORY_ID: std::sync::atomic::AtomicU64 =
    std::sync::atomic::AtomicU64::new(0);

struct TestDirectory(std::path::PathBuf);

impl Drop for TestDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn test_paths(label: &str) -> (TestDirectory, ProgressPaths) {
    use std::sync::atomic::Ordering;
    let id = TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "mana-progress-{label}-{}-{id}",
        std::process::id(),
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let paths = ProgressPaths::from_primary(dir.join("progress.json"));
    (TestDirectory(dir), paths)
}
```

Then add:

```rust
#[test]
fn corrupt_primary_recovers_from_backup() {
    let (dir, paths) = test_paths("backup-recovery");
    std::fs::write(&paths.primary, b"not json").unwrap();
    std::fs::write(&paths.backup, encode_state(&fixture_state()).unwrap()).unwrap();
    let loaded = load_state(&paths).unwrap();
    assert_eq!(loaded.source, RecoverySource::Backup);
    assert_eq!(loaded.state, fixture_state());
    drop(dir);
}

#[test]
fn existing_invalid_files_never_become_default() {
    let (_dir, paths) = test_paths("invalid-existing");
    std::fs::write(&paths.primary, b"broken").unwrap();
    let error = load_state(&paths).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn no_files_is_the_only_new_install_path() {
    let (_dir, paths) = test_paths("new-install");
    let loaded = load_state(&paths).unwrap();
    assert_eq!(loaded.source, RecoverySource::New);
    assert!(!loaded.state.initialized);
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cd src-tauri && cargo test progress_store::tests`

Expected: FAIL because recovery types and `load_state` are missing.

- [ ] **Step 3: Implement strict recovery order**

Read `primary`, `backup`, and `pre_migration` in order. A missing file advances to the next candidate; invalid bytes are retained and also advance. Return `ProgressState::default()` only when none of the three paths exists.

When the primary decodes as legacy, create `pre_migration` with `OpenOptions::new().write(true).create_new(true)` and write the original bytes. Treat `AlreadyExists` as success and never truncate the existing snapshot.

- [ ] **Step 4: Add snapshot immutability test**

```rust
#[test]
fn legacy_snapshot_is_written_once_and_never_overwritten() {
    let (_dir, paths) = test_paths("immutable-legacy");
    let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
    std::fs::write(&paths.primary, legacy).unwrap();
    load_state(&paths).unwrap();
    let original = std::fs::read(&paths.pre_migration).unwrap();
    std::fs::write(&paths.primary, br#"{"rank":0}"#).unwrap();
    let _ = load_state(&paths);
    assert_eq!(std::fs::read(&paths.pre_migration).unwrap(), original);
}
```

- [ ] **Step 5: Run and verify GREEN**

Run: `cd src-tauri && cargo test progress_store::tests`

Expected: all load, recovery, and snapshot tests PASS.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/progress_store.rs
git commit -m "feat: recover progress from durable backups"
```

---

### Task 4: Replace direct writes with validated atomic saves

**Files:**
- Modify: `src-tauri/src/progress_store.rs`
- Test: `src-tauri/src/progress_store.rs`

**Interfaces:**
- Produces:

```rust
pub fn save_state(paths: &ProgressPaths, state: &ProgressState) -> std::io::Result<()>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveCheckpoint { TemporarySynced, BackupReplaced, PrimaryReplaced }
```

- [ ] **Step 1: Write failing atomic-save tests**

Add these focused helpers beside the storage tests:

```rust
fn state_with_rank(rank: usize) -> ProgressState {
    let mut state = fixture_state();
    state.rank = rank;
    state
}

fn load_exact(path: &std::path::Path) -> std::io::Result<ProgressState> {
    let bytes = std::fs::read(path)?;
    decode_state(&bytes).map(|(state, _)| state)
}
```

Then add:

```rust
#[test]
fn save_keeps_previous_primary_as_backup() {
    let (_dir, paths) = test_paths("backup-rotation");
    let old = state_with_rank(3);
    let new = state_with_rank(4);
    save_state(&paths, &old).unwrap();
    save_state(&paths, &new).unwrap();
    assert_eq!(load_exact(&paths.primary).unwrap(), new);
    assert_eq!(load_exact(&paths.backup).unwrap(), old);
}

#[test]
fn every_interrupted_boundary_leaves_a_recoverable_state() {
    for checkpoint in [
        SaveCheckpoint::TemporarySynced,
        SaveCheckpoint::BackupReplaced,
        SaveCheckpoint::PrimaryReplaced,
    ] {
        let (_dir, paths) = test_paths(&format!("interrupt-{checkpoint:?}"));
        save_state(&paths, &state_with_rank(3)).unwrap();
        let _ = save_state_with_hook(&paths, &state_with_rank(4), |reached| {
            if reached == checkpoint {
                return Err(std::io::Error::other("simulated interruption"));
            }
            Ok(())
        });
        let loaded = load_state(&paths).unwrap();
        assert!([3, 4].contains(&loaded.state.rank));
    }
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cd src-tauri && cargo test progress_store::tests`

Expected: FAIL because atomic save and checkpoint injection are missing.

- [ ] **Step 3: Implement same-directory validated replacement**

`save_state_with_hook` performs these exact operations:

```rust
let bytes = encode_state(state)?;
std::fs::create_dir_all(paths.primary.parent().unwrap())?;
let mut temp = std::fs::OpenOptions::new()
    .create(true).truncate(true).write(true)
    .open(&paths.temporary)?;
use std::io::Write as _;
temp.write_all(&bytes)?;
temp.sync_all()?;
drop(temp);
decode_state(&std::fs::read(&paths.temporary)?)?;
hook(SaveCheckpoint::TemporarySynced)?;
```

If the primary exists, validate it before atomically replacing the prior backup with it. Then rename the validated temporary file to the primary, call the remaining hooks, and `sync_all` an open handle to the parent directory on macOS/Unix.

After `save_state` exists, update `load_state` so a state recovered from the
backup, pre-migration snapshot, or an unversioned primary is rewritten through
the same atomic writer. The recovered bytes are validated first; an invalid
primary is never rotated over a valid backup. Remove an abandoned temporary
file only after another valid state has loaded.

- [ ] **Step 4: Run and verify GREEN**

Run: `cd src-tauri && cargo test progress_store::tests`

Expected: all atomic-save and interruption tests PASS.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/progress_store.rs
git commit -m "feat: save progression atomically"
```

---

### Task 5: Make commands and scanning transactional

**Files:**
- Modify: `src-tauri/src/progress.rs`
- Modify: `src-tauri/src/progress_store.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/progress.rs`

**Interfaces:**
- Produces:

```rust
pub struct ProgressStore {
    pub(crate) state: std::sync::Mutex<ProgressState>,
    pub(crate) paths: ProgressPaths,
}

impl ProgressStore {
    pub fn load(app: &tauri::AppHandle) -> std::io::Result<Self>;
}
```

- [ ] **Step 1: Write failing transactional helper tests**

Extract a persistence-injected helper in `progress.rs`:

```rust
fn commit_candidate<F>(
    current: &mut ProgressState,
    candidate: ProgressState,
    persist: F,
) -> Result<Progress, String>
where
    F: FnOnce(&ProgressState) -> Result<(), String>;
```

Test it:

```rust
#[test]
fn failed_persistence_does_not_commit_candidate() {
    let mut current = state_with(800_000, 0, 0, 0);
    let mut candidate = current.clone();
    try_rank_up(&mut candidate).unwrap();
    let result = commit_candidate(&mut current, candidate, |_| Err("disk full".into()));
    assert!(result.is_err());
    assert_eq!(current.rank, 0);
}
```

- [ ] **Step 2: Run and verify RED**

Run: `cd src-tauri && cargo test progress::tests::failed_persistence`

Expected: FAIL because `commit_candidate` does not exist.

- [ ] **Step 3: Replace direct persistence paths**

Remove `save_progress`, `load_progress`, `store_path`, and the tuple-style `ProgressStore` from `progress.rs`. Re-export the new store with:

```rust
pub use crate::progress_store::ProgressStore;
```

`rank_up` and `prestige` lock the store, clone the current state, mutate the candidate, call `save_state(&store.paths, &candidate)`, and assign the candidate only after success. Emit `progress-update` after the lock is released.

The watcher scans a candidate clone. Persist whenever `candidate != *current`, even if the derived level view did not change, so token cursors and sub-XP totals remain durable. Emit only when the derived `Progress` changed.

- [ ] **Step 4: Fail startup instead of silently resetting unreadable state**

In `lib.rs`:

```rust
let progress_store = progress::ProgressStore::load(app.handle())
    .map_err(|error| std::io::Error::other(format!("progress recovery failed: {error}")))?;
app.manage(progress_store);
```

The app must not start its watcher against a fabricated default when files exist but are invalid.

- [ ] **Step 5: Run focused and full Rust tests**

Run:

```bash
cd src-tauri
cargo test progress
cargo test progress_store
cargo test
```

Expected: all tests PASS, including existing XP and scanner tests.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/lib.rs src-tauri/src/progress.rs src-tauri/src/progress_store.rs
git commit -m "feat: commit progression transactionally"
```

---

### Task 6: Lock the bundle identity and verify an upgrade over real-format data

**Files:**
- Modify: `src/branding.test.ts`
- Modify: `src-tauri/src/progress_store.rs`
- Test: `src/branding.test.ts`
- Test: `src-tauri/src/progress_store.rs`

**Interfaces:**
- Consumes: `com.vantasoft.mana`, the version 1 fixture, and the new migration path.
- Produces: a regression gate preventing accidental app-data directory changes.

- [ ] **Step 1: Extend the existing identity regression test**

Read `src-tauri/src/progress_store.rs` from `branding.test.ts` and add:

```ts
expect(tauriConfig.identifier).toBe("com.vantasoft.mana");
expect(progressStoreSource).toContain('dir.join("progress.json")');
expect(progressStoreSource).toContain('dir.join("progress.json.bak")');
```

- [ ] **Step 2: Run complete automated verification**

Run:

```bash
npm test
npm run build
cd src-tauri && cargo test
```

Expected: Vitest, TypeScript/Vite, and all Rust tests PASS.

- [ ] **Step 3: Add an ignored external-fixture migration test**

Add this ignored test to `progress_store.rs`:

```rust
#[test]
#[ignore = "requires MANA_PROGRESS_V1_FIXTURE"]
fn external_v1_fixture_preserves_all_progress_invariants() {
    let source = std::path::PathBuf::from(
        std::env::var_os("MANA_PROGRESS_V1_FIXTURE").expect("fixture path"),
    );
    let bytes = std::fs::read(source).unwrap();
    let (before, _) = decode_state(&bytes).unwrap();
    let (_dir, paths) = test_paths("external-migration");
    std::fs::write(&paths.primary, bytes).unwrap();
    let after = load_state(&paths).unwrap().state;
    assert_eq!(after, before);
    assert!(paths.pre_migration.exists());
}
```

Run it against the manual copy, never the live primary:

```bash
APP_DIR="$HOME/Library/Application Support/com.vantasoft.mana"
BACKUP="$(ls -t "$APP_DIR"/progress.manual-before-v2-*.json | head -1)"
MANA_PROGRESS_V1_FIXTURE="$BACKUP" \
  cargo test --manifest-path src-tauri/Cargo.toml \
  external_v1_fixture_preserves_all_progress_invariants -- --ignored
```

Expected: PASS; the complete `ProgressState`, including cursor maps, is byte-semantically identical after migration.

- [ ] **Step 4: Build the Tauri application**

Run: `npm run tauri build`

Expected: exit 0 and a packaged `Mana.app` under `src-tauri/target/release/bundle/macos/`.

- [ ] **Step 5: Install over a disposable copy, never the only live data**

Launch the packaged app once with the disposable migrated directory or a temporary HOME/app-data harness. Confirm the invariant summary remains identical. Only after that passes may the build be run against the user's real app-data directory.

- [ ] **Step 6: Commit**

```bash
git add src/branding.test.ts src-tauri/src/progress_store.rs
git commit -m "test: lock Mana progress identity"
```
