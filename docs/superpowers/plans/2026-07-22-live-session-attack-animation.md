# Live Session Attack Animation Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Claude and Codex attack only while their local session logs are actively changing, keep Mana quiet on launch, and remove the detached left-edge debris from Claude's Master working frames.

**Architecture:** Replace process-presence checks with a pure per-provider fingerprint tracker that observes JSONL metadata, seeds a quiet startup baseline, and holds activity for 2.5 seconds after a write. A Tauri-managed store exposes the current pair to the frontend and emits the existing event on state changes. Clean the single affected atlas behind a failing alpha-strip regression test.

**Tech Stack:** Rust 2021, Tauri 2, Tokio time, TypeScript, Vitest, RGBA PNG sprite atlases.

## Global Constraints

- Poll local session metadata every 1 second.
- Keep a provider active for exactly 2.5 seconds after its latest detected write; at 2.5 seconds it is idle.
- The first scan is always a quiet baseline, even when sessions already exist.
- Claude and Codex activity must remain independent.
- Add no Rust or npm dependency.
- Preserve the frontend state priority `hover/moving > working > idle`.
- Preserve the Claude Master atlas at 448x336 RGBA with a 4x3 grid and transparent cell margins.
- Do not change rank progression, token totals, usage polling, credentials, window sizing, sprite timing, hover behavior, or non-Master artwork.

---

## File Map

- Modify `src-tauri/src/activity.rs`: pure JSONL fingerprint scanning, provider activity tracker, managed activity state, watcher, and Rust tests.
- Modify `src-tauri/src/lib.rs`: register the managed activity store before starting the watcher.
- Modify `README.md`: describe session-write activity detection rather than process presence.
- Modify `src/sprites.test.ts`: assert the Claude Master working-row left strip is transparent.
- Modify `public/sprites/claude-rank-master.png`: remove only detached working-row fragments.

### Task 1: Pure Session-Write Activity Tracker

**Files:**
- Modify: `src-tauri/src/activity.rs`

**Interfaces:**
- Produces: `const ACTIVITY_GRACE: Duration = Duration::from_millis(2500)`.
- Produces: `fn jsonl_fingerprints(root: &Path) -> HashMap<PathBuf, FileFingerprint>`.
- Produces: `ProviderActivity::update(&mut self, current: HashMap<PathBuf, FileFingerprint>, now: Instant) -> bool`.
- Consumes: only `std::fs`, `std::path`, `std::time`, and `std::collections`.

- [ ] **Step 1: Replace the old matcher tests with failing tracker tests**

Add tests in `src-tauri/src/activity.rs` that construct fingerprint maps directly:

```rust
fn fingerprints(entries: &[(&str, u64, u64)]) -> HashMap<PathBuf, FileFingerprint> {
    entries
        .iter()
        .map(|(path, len, modified_ms)| {
            (
                PathBuf::from(path),
                FileFingerprint {
                    len: *len,
                    modified: UNIX_EPOCH + Duration::from_millis(*modified_ms),
                },
            )
        })
        .collect()
}

#[test]
fn initial_scan_is_quiet_and_a_later_write_attacks() {
    let start = Instant::now();
    let mut tracker = ProviderActivity::default();
    assert!(!tracker.update(fingerprints(&[("session.jsonl", 10, 1)]), start));
    assert!(tracker.update(
        fingerprints(&[("session.jsonl", 20, 2)]),
        start + Duration::from_secs(1),
    ));
}

#[test]
fn new_file_after_baseline_attacks_but_deleted_files_do_not() {
    let start = Instant::now();
    let mut tracker = ProviderActivity::default();
    assert!(!tracker.update(fingerprints(&[("old.jsonl", 10, 1)]), start));
    assert!(!tracker.update(HashMap::new(), start + Duration::from_secs(1)));
    assert!(tracker.update(
        fingerprints(&[("new.jsonl", 1, 2)]),
        start + Duration::from_secs(2),
    ));
}

#[test]
fn activity_expires_at_the_grace_boundary() {
    let start = Instant::now();
    let mut tracker = ProviderActivity::default();
    tracker.update(fingerprints(&[("session.jsonl", 10, 1)]), start);
    tracker.update(
        fingerprints(&[("session.jsonl", 20, 2)]),
        start + Duration::from_secs(1),
    );
    assert!(tracker.update(
        fingerprints(&[("session.jsonl", 20, 2)]),
        start + Duration::from_millis(3499),
    ));
    assert!(!tracker.update(
        fingerprints(&[("session.jsonl", 20, 2)]),
        start + Duration::from_millis(3500),
    ));
}
```

Add a filesystem test using a unique path under `std::env::temp_dir()`:

```rust
#[test]
fn missing_directory_scans_as_quiet() {
    let root = std::env::temp_dir().join(format!(
        "mana-missing-activity-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    assert!(jsonl_fingerprints(&root).is_empty());
}
```

- [ ] **Step 2: Run the focused Rust tests and verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml activity::tests -- --nocapture
```

Expected: compilation fails because `FileFingerprint`, `ProviderActivity`, and `jsonl_fingerprints` do not exist after the old process-only implementation.

- [ ] **Step 3: Implement the minimal tracker**

Replace the process-command imports and matcher with:

```rust
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime};
use tauri::Emitter;

const ACTIVITY_GRACE: Duration = Duration::from_millis(2500);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileFingerprint {
    len: u64,
    modified: SystemTime,
}

fn jsonl_fingerprints(root: &Path) -> HashMap<PathBuf, FileFingerprint> {
    let mut result = HashMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else { continue };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(metadata) = entry.metadata() else { continue };
            if metadata.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "jsonl") {
                let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                result.insert(path, FileFingerprint { len: metadata.len(), modified });
            }
        }
    }
    result
}

#[derive(Default)]
struct ProviderActivity {
    previous: Option<HashMap<PathBuf, FileFingerprint>>,
    last_write_at: Option<Instant>,
}

impl ProviderActivity {
    fn update(&mut self, current: HashMap<PathBuf, FileFingerprint>, now: Instant) -> bool {
        let wrote = self.previous.as_ref().is_some_and(|previous| {
            current.iter().any(|(path, fingerprint)| previous.get(path) != Some(fingerprint))
        });
        self.previous = Some(current);
        if wrote {
            self.last_write_at = Some(now);
        }
        self.last_write_at
            .is_some_and(|last_write| now.saturating_duration_since(last_write) < ACTIVITY_GRACE)
    }
}
```

- [ ] **Step 4: Run the focused Rust tests and verify GREEN**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml activity::tests -- --nocapture
```

Expected: all activity tracker tests pass.

- [ ] **Step 5: Commit the pure tracker**

```bash
git add src-tauri/src/activity.rs
git commit -m "fix: track live session writes for attacks"
```

### Task 2: Tauri Activity Store and Watcher

**Files:**
- Modify: `src-tauri/src/activity.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `README.md`

**Interfaces:**
- Consumes: `ProviderActivity::update` and `jsonl_fingerprints` from Task 1.
- Produces: `pub struct ActivityStore(pub Mutex<Activity>)`.
- Produces: `pub fn get_activity(store: tauri::State<'_, ActivityStore>) -> Activity`.
- Preserves: frontend event name `activity` and JSON keys `claude`, `codex`.

- [ ] **Step 1: Add failing source-contract tests**

Add Rust tests in `src-tauri/src/activity.rs`:

```rust
#[test]
fn providers_track_independently() {
    let start = Instant::now();
    let mut claude = ProviderActivity::default();
    let mut codex = ProviderActivity::default();
    claude.update(fingerprints(&[("claude.jsonl", 10, 1)]), start);
    codex.update(fingerprints(&[("codex.jsonl", 10, 1)]), start);

    assert!(claude.update(
        fingerprints(&[("claude.jsonl", 20, 2)]),
        start + Duration::from_secs(1),
    ));
    assert!(!codex.update(
        fingerprints(&[("codex.jsonl", 10, 1)]),
        start + Duration::from_secs(1),
    ));
}

#[test]
fn activity_store_starts_quiet() {
    assert_eq!(
        *ActivityStore::default().0.lock().unwrap(),
        Activity { claude: false, codex: false }
    );
}
```

Update `src-tauri/src/lib.rs` tests to assert the source registers the activity store:

```rust
#[test]
fn builder_registers_activity_store() {
    let source = include_str!("lib.rs");
    assert!(source.contains(".manage(activity::ActivityStore::default())"));
}
```

- [ ] **Step 2: Run focused tests and verify RED**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml activity_store -- --nocapture
```

Expected: FAIL because `Activity` and `ActivityStore` are not defined.

- [ ] **Step 3: Implement the shared state, watcher, and command**

Add:

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Activity {
    claude: bool,
    codex: bool,
}

#[derive(Default)]
pub struct ActivityStore(pub Mutex<Activity>);
```

Replace the watcher and command with:

```rust
pub fn spawn_activity_watcher(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        use tauri::Manager as _;
        let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else { return };
        let claude_root = home.join(".claude/projects");
        let codex_root = home.join(".codex/sessions");
        let mut claude = ProviderActivity::default();
        let mut codex = ProviderActivity::default();
        let mut tick = tokio::time::interval(Duration::from_secs(1));

        loop {
            tick.tick().await;
            let now = Instant::now();
            let next = Activity {
                claude: claude.update(jsonl_fingerprints(&claude_root), now),
                codex: codex.update(jsonl_fingerprints(&codex_root), now),
            };
            let changed = {
                let store = app.state::<ActivityStore>();
                let current = &mut *store.0.lock().unwrap();
                if *current == next { false } else { *current = next; true }
            };
            if changed {
                let _ = app.emit("activity", next);
            }
        }
    });
}

#[tauri::command]
pub fn get_activity(store: tauri::State<'_, ActivityStore>) -> Activity {
    *store.0.lock().unwrap()
}
```

In `src-tauri/src/lib.rs`, add the store beside the snapshot store:

```rust
.manage(poll::Snapshots::default())
.manage(activity::ActivityStore::default())
```

Replace the README activity sentence with:

```markdown
- Attack activity is detected locally from Claude and Codex session-log writes once per second; Mana never uploads session content or activity telemetry.
```

- [ ] **Step 4: Run Rust tests and frontend tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
npm test
```

Expected: all Rust tests and all Vitest files pass.

- [ ] **Step 5: Commit the integration**

```bash
git add src-tauri/src/activity.rs src-tauri/src/lib.rs README.md
git commit -m "fix: animate attacks only during live work"
```

### Task 3: Remove Claude Master Left-Edge Debris

**Files:**
- Modify: `src/sprites.test.ts`
- Modify: `public/sprites/claude-rank-master.png`

**Interfaces:**
- Consumes: existing `decodeRgba` PNG helper in `src/sprites.test.ts`.
- Produces: a 448x336 RGBA atlas whose second-row first 12 source pixels in every cell are transparent.

- [ ] **Step 1: Add the failing alpha-strip regression test**

Add:

```typescript
it("keeps Claude Master working frames free of left-edge debris", () => {
  const image = decodeRgba(
    new URL("../public/sprites/claude-rank-master.png", import.meta.url),
  );
  expect([image.width, image.height]).toEqual([448, 336]);
  const cell = 112;
  const workingTop = cell;
  const workingBottom = cell * 2;
  for (let column = 0; column < 4; column += 1) {
    for (let y = workingTop; y < workingBottom; y += 1) {
      for (let x = column * cell; x < column * cell + 12; x += 1) {
        expect(image.pixels[(y * image.width + x) * 4 + 3]).toBeLessThanOrEqual(16);
      }
    }
  }
});
```

- [ ] **Step 2: Run the focused test and verify RED**

Run:

```bash
npm test -- src/sprites.test.ts
```

Expected: FAIL on visible alpha in the far-left strip of Claude Master working cells.

- [ ] **Step 3: Edit only the affected sprite fragments**

Use the image-editing tool on `public/sprites/claude-rank-master.png` with this exact instruction:

```text
Preserve this transparent 448x336 RGBA 4-column by 3-row pixel-art sprite atlas exactly. Remove only the detached tiny red, orange, and dark flecks in the far-left transparent margin of the second-row attack frames, especially the isolated pieces around x=4..9 inside cells 2, 3, and 4. Do not alter the Claude character, skull wand, portals, flames, particles connected to the spells, colors, scale, baselines, transparency, dimensions, grid, or any pixel outside those detached far-left fragments.
```

Reject the output if dimensions change or if visual inspection shows any change beyond the detached fragments. Replace the tracked PNG only after it satisfies those constraints.

- [ ] **Step 4: Verify the atlas test and inspect the result**

Run:

```bash
npm test -- src/sprites.test.ts
sips -g pixelWidth -g pixelHeight -g format public/sprites/claude-rank-master.png
```

Expected: all sprite tests pass; `pixelWidth: 448`, `pixelHeight: 336`, and `format: png`. Visually confirm the working row retains the skull and portal effects with no isolated fragment on the far left.

- [ ] **Step 5: Commit the asset cleanup**

```bash
git add src/sprites.test.ts public/sprites/claude-rank-master.png
git commit -m "fix: remove Claude Master sprite debris"
```

### Task 4: Full Verification, Release Build, and Local Relaunch

**Files:**
- Verify all files changed by Tasks 1-3.
- Build output: `src-tauri/target/release/bundle/macos/Mana.app`

**Interfaces:**
- Consumes: completed activity tracker/store and cleaned Master atlas.
- Produces: verified release app and synchronized GitHub `main`.

- [ ] **Step 1: Run all automated verification**

Run:

```bash
npm test
cargo test --manifest-path src-tauri/Cargo.toml
npm run build
```

Expected: all Vitest files pass, all Rust tests pass, and the production frontend build exits 0.

- [ ] **Step 2: Build and verify the macOS release app**

Run:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
npm run tauri build
codesign --force --deep --sign - --timestamp=none src-tauri/target/release/bundle/macos/Mana.app
codesign --verify --deep --strict --verbose=2 src-tauri/target/release/bundle/macos/Mana.app
```

Expected: Tauri build exits 0 and codesign reports that Mana is valid on disk and satisfies its designated requirement.

- [ ] **Step 3: Relaunch and manually exercise the original symptoms**

Open `src-tauri/target/release/bundle/macos/Mana.app`. Confirm:

1. Both sprites begin in idle on a quiet launch.
2. Writing to the active Claude session log starts Claude's attack within one second.
3. Writing to the active Codex desktop session log starts Codex's attack within one second.
4. Each provider returns to idle about 2.5 seconds after its writes stop.
5. Relaunching with quiet sessions does not attack.
6. Claude Master shows no detached fragment on the left.

- [ ] **Step 4: Verify repository state and push**

Run:

```bash
git diff --check
git status --short --branch
git push origin main
git fetch origin main --quiet
git rev-list --left-right --count origin/main...main
```

Expected: no uncommitted changes, push succeeds, and divergence is `0 0`.
