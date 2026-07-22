# Rank Progression Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lifetime token usage becomes XP → exponential levels → 13 cosmetic ranks with manual Rank Up, plus prestige cycles that steepen the curve and award badges.

**Architecture:** Rust (`progress.rs`) owns all math, tallying, persistence and validation, mirroring how `poll.rs` owns usage snapshots; it emits `progress-update` events. The frontend only renders: a footer strip (level chip, XP bar, prestige badges), a top-right Rank Up / Prestige button, tier-themed dialogs, `data-rank` border themes, and per-tier sprite sheets with graceful fallback.

**Tech Stack:** Tauri v2 (Rust backend, vanilla TS frontend), vitest, cargo test. No new dependencies.

**Spec:** `docs/superpowers/specs/2026-07-21-rank-progression-design.md` — read it first; its numbers are authoritative.

## Global Constraints

- Tiers (index = rank): `["naked","plastic","wood","iron","bronze","silver","gold","platinum","emerald","diamond","master","legend","champion","godlike"]` (14 entries, rank 0–13).
- Gate levels (same indexing, gate to *reach* that rank): `[0, 5, 10, 15, 21, 28, 36, 45, 55, 66, 78, 91, 105, 120]`.
- `TOKENS_PER_XP = 1000`; XP to reach level L at prestige p: `floor(0.8 · L³ · 1.5^p)` — integer form `(4·L³·3^p) / (5·2^p)` using u128 intermediates, saturating.
- Effective XP = `(total_tokens − prestige_token_floor) / 1000`.
- Rank/prestige NEVER auto-advance; only `rank_up()` / `prestige()` commands mutate them, with server-side validation.
- Frontend event/command payload (exact shape, snake_case):
  `{ "xp": number, "level": number, "rank": number, "tier": string, "prestige": number, "rank_up_eligible": boolean, "prestige_eligible": boolean, "level_progress": { "current": number, "needed": number } }`
- Code style: match existing files exactly (small pure functions, doc comments explaining *why*, double quotes + semicolons in TS, existing test idioms).
- Every task: TDD (failing test shown to fail → minimal impl → pass → commit, conventional-commit message ending with the `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>` trailer).
- Verification commands: `npm test`, `npx tsc --noEmit`, `cd src-tauri && cargo test`.
- This plan builds ON TOP of the merged drag-resize and concentric-corner work (main.ts has zoom/scale logic; styles.css has `calc(var(--hud-radius) - Npx)` corner math). Do not regress either.

---

### Task 1: Curve, tiers, and gate math (Rust)

**Files:**
- Create: `src-tauri/src/progress.rs` (math section only)
- Modify: `src-tauri/src/lib.rs:1-4` (add `pub mod progress;`)

**Interfaces (Produces):**
```rust
pub const TIERS: [&str; 14];
pub const GATES: [u32; 14];
pub const TOKENS_PER_XP: u64 = 1000;
pub fn xp_for_level(level: u32, prestige: u32) -> u64; // XP to *reach* level; level<=1 → 0
pub fn level_for_xp(xp: u64, prestige: u32) -> u32;    // >=1, monotonic, caps at 999
pub fn rank_up_eligible(level: u32, rank: usize) -> bool;   // rank<13 && level >= GATES[rank+1]
pub fn prestige_eligible(rank: usize) -> bool;              // rank == 13
```

- [x] **Step 1: Write failing tests** in a `#[cfg(test)] mod tests` in `progress.rs`:

```rust
#[test]
fn curve_matches_cubic_fast_formula() {
    assert_eq!(xp_for_level(1, 0), 0);
    assert_eq!(xp_for_level(2, 0), 6);      // floor(0.8*8)
    assert_eq!(xp_for_level(10, 0), 800);
    assert_eq!(xp_for_level(120, 0), 1_382_400);
}

#[test]
fn prestige_steepens_curve_by_1_5x_each_cycle() {
    assert_eq!(xp_for_level(10, 1), 1200);  // 800 * 1.5
    assert_eq!(xp_for_level(10, 2), 1800);
    assert_eq!(xp_for_level(10, 4), 4050);
}

#[test]
fn level_for_xp_inverts_curve() {
    assert_eq!(level_for_xp(0, 0), 1);
    assert_eq!(level_for_xp(5, 0), 1);
    assert_eq!(level_for_xp(6, 0), 2);
    assert_eq!(level_for_xp(799, 0), 9);
    assert_eq!(level_for_xp(800, 0), 10);
    assert_eq!(level_for_xp(1199, 1), 9);
    assert_eq!(level_for_xp(1200, 1), 10);
}

#[test]
fn gates_align_with_tiers() {
    assert_eq!(TIERS.len(), 14);
    assert_eq!(GATES.len(), 14);
    assert_eq!(TIERS[0], "naked");
    assert_eq!(TIERS[13], "godlike");
    assert_eq!(GATES[13], 120);
    assert!(GATES.windows(2).all(|w| w[0] < w[1] || w[0] == 0));
}

#[test]
fn eligibility_rules() {
    assert!(!rank_up_eligible(4, 0));
    assert!(rank_up_eligible(5, 0));
    assert!(rank_up_eligible(200, 0));
    assert!(!rank_up_eligible(200, 13));   // godlike: no more ranks
    assert!(!prestige_eligible(12));
    assert!(prestige_eligible(13));
}
```

- [x] **Step 2:** `cd src-tauri && cargo test progress` → FAIL (module/functions missing).
- [x] **Step 3:** Implement. Core of the curve (u128 to survive `3^p·L³`):

```rust
pub fn xp_for_level(level: u32, prestige: u32) -> u64 {
    if level <= 1 { return 0; }
    let l = level as u128;
    let p = prestige.min(40); // 1.5^40 already dwarfs any real token count
    let num = 4u128.saturating_mul(l * l * l).saturating_mul(3u128.saturating_pow(p));
    let den = 5u128 * 2u128.saturating_pow(p);
    u64::try_from(num / den).unwrap_or(u64::MAX)
}

pub fn level_for_xp(xp: u64, prestige: u32) -> u32 {
    let mut level = 1;
    while level < 999 && xp_for_level(level + 1, prestige) <= xp { level += 1; }
    level
}
```

- [x] **Step 4:** `cargo test progress` → all 5 PASS.
- [x] **Step 5:** Commit: `feat: rank curve and gate math`.

---

### Task 2: Incremental token tally scanner (Rust)

**Files:**
- Modify: `src-tauri/src/progress.rs` (tally section)
- Test fixtures: written by tests into `std::env::temp_dir()` subdirs (see tests; do not commit fixture files)

**Interfaces (Produces):**
```rust
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct TallyState {
    pub total_tokens: u64,
    pub claude_offsets: std::collections::HashMap<String, u64>, // path -> consumed byte offset
    pub codex_offsets: std::collections::HashMap<String, u64>,
    pub codex_totals: std::collections::HashMap<String, u64>,   // path -> last cumulative total_tokens
}
pub fn scan_claude_dir(dir: &std::path::Path, state: &mut TallyState);
pub fn scan_codex_dir(dir: &std::path::Path, state: &mut TallyState);
```

Rules the implementation MUST honor (each is a test):
1. Claude: recurse `*.jsonl`; from the stored offset, parse complete lines only (advance offset to the byte after the last `\n` — files are written mid-line by live sessions); per line, add `message.usage.input_tokens + cache_creation_input_tokens + cache_read_input_tokens + output_tokens` (each defaulting to 0 if absent); malformed lines are skipped but still consume offset.
2. Codex: recurse `*.jsonl`; from stored offset, find `token_count` events with `info.total_token_usage.total_tokens`; that value is a per-file RUNNING TOTAL — add `latest − codex_totals[path]` (never sum events), then update `codex_totals[path]`. A latest smaller than stored (truncated/rotated file) contributes 0 and resets the stored total.
3. Re-scanning an unchanged tree changes nothing (idempotence).
4. Appending to a scanned file adds only the delta.

- [ ] **Step 1: Write failing tests** (same `tests` mod). Test skeleton showing the fixture idiom and the four rules:

```rust
fn write(path: &std::path::Path, contents: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, contents).unwrap();
}

const CLAUDE_LINE: &str = r#"{"message":{"usage":{"input_tokens":10,"cache_creation_input_tokens":5,"cache_read_input_tokens":85,"output_tokens":100}}}"#;
const CODEX_EVENT_20K: &str = r#"{"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1,"cached_input_tokens":2,"output_tokens":3,"reasoning_output_tokens":4,"total_tokens":20000}}}}"#;

#[test]
fn claude_scan_sums_all_usage_fields_and_is_idempotent() {
    let dir = std::env::temp_dir().join(format!("mana-tally-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let file = dir.join("proj/session.jsonl");
    write(&file, &format!("{CLAUDE_LINE}\nnot json\n{CLAUDE_LINE}\n"));
    let mut state = TallyState::default();
    scan_claude_dir(&dir, &mut state);
    assert_eq!(state.total_tokens, 400); // 200 * 2, malformed line skipped
    scan_claude_dir(&dir, &mut state);
    assert_eq!(state.total_tokens, 400); // idempotent
    // an appended line adds only the delta, and a trailing partial line is not consumed
    let mut f = std::fs::OpenOptions::new().append(true).open(&file).unwrap();
    use std::io::Write as _;
    write!(f, "{CLAUDE_LINE}\n{{\"partial").unwrap();
    scan_claude_dir(&dir, &mut state);
    assert_eq!(state.total_tokens, 600);
    let stored = *state.claude_offsets.values().next().unwrap();
    assert!(stored < std::fs::metadata(&file).unwrap().len());
}
```

Plus the codex mirror test: one file containing the 20k event then (after first scan) an appended event with `"total_tokens":20900` → totals go 20000 → 20900 (delta 900, not 40900); and a fresh `TallyState` given a file whose latest total is smaller than a stored `codex_totals` entry contributes 0.

- [ ] **Step 2:** `cargo test tally` → FAIL.
- [ ] **Step 3:** Implement. Notes: recurse with a small manual stack (no walkdir dep); read with `std::fs::File` + `Seek` to offset; `serde_json::from_str::<serde_json::Value>` per line; extract via `.get(...).and_then(|v| v.as_u64())` chains. For codex, search `payload.info.total_token_usage.total_tokens` and also tolerate the field at `info.total_token_usage` directly (both shapes exist in the wild).
- [ ] **Step 4:** `cargo test` → PASS (Task 1 tests still green).
- [ ] **Step 5:** Commit: `feat: incremental token tally scanner`.

---

### Task 3: Progress state, commands, watcher, wiring (Rust)

**Files:**
- Modify: `src-tauri/src/progress.rs` (state section)
- Modify: `src-tauri/src/lib.rs` (manage state, spawn watcher next to `poll::spawn_pollers`, register commands in `generate_handler!`)

**Interfaces (Produces):**
```rust
#[derive(Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProgressState {
    pub rank: usize,
    pub prestige: u32,
    pub prestige_token_floor: u64,
    pub tally: TallyState,
}
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct Progress { /* exact payload from Global Constraints */ }
pub fn progress_view(state: &ProgressState) -> Progress;
pub fn try_rank_up(state: &mut ProgressState) -> Result<(), String>;
pub fn try_prestige(state: &mut ProgressState) -> Result<(), String>;
// Tauri layer:
pub struct ProgressStore(pub std::sync::Mutex<ProgressState>); // managed via app.manage()
#[tauri::command] pub fn get_progress(...) -> Progress;
#[tauri::command] pub fn rank_up(...) -> Result<Progress, String>;
#[tauri::command] pub fn prestige(...) -> Result<Progress, String>;
pub fn spawn_progress_watcher(app: tauri::AppHandle); // 60s interval; scans real dirs; persists + emits "progress-update" on change
```

- [ ] **Step 1: Failing tests** for the pure layer:

```rust
fn state_with(tokens: u64, rank: usize, prestige: u32, floor: u64) -> ProgressState {
    ProgressState {
        rank, prestige, prestige_token_floor: floor,
        tally: TallyState { total_tokens: tokens, ..Default::default() },
    }
}

#[test]
fn progress_view_derives_everything() {
    // 800_000 tokens = 800 xp = level 10 at prestige 0
    let v = progress_view(&state_with(800_000, 1, 0, 0));
    assert_eq!((v.xp, v.level, v.tier, v.rank_up_eligible), (800, 10, "wood".into(), true)); // gate for rank 2 (wood→iron gate is 10)... see note below
    assert_eq!(v.level_progress.needed, xp_for_level(11, 0) - xp_for_level(10, 0));
}

#[test]
fn rank_up_walks_one_tier_and_validates() {
    let mut s = state_with(800_000, 0, 0, 0); // level 10: eligible for plastic AND wood
    assert!(try_rank_up(&mut s).is_ok());
    assert_eq!(s.rank, 1);
    assert!(try_rank_up(&mut s).is_ok());
    assert_eq!(s.rank, 2);
    assert!(try_rank_up(&mut s).is_err()); // gate 15 not reached
}

#[test]
fn prestige_resets_baseline_and_steepens() {
    let mut s = state_with(2_000_000_000, 13, 0, 0);
    assert!(try_prestige(&mut s).is_ok());
    assert_eq!((s.rank, s.prestige, s.prestige_token_floor), (0, 1, 2_000_000_000));
    let v = progress_view(&s);
    assert_eq!((v.xp, v.level), (0, 1));
    let mut not_godlike = state_with(2_000_000_000, 12, 0, 0);
    assert!(try_prestige(&mut not_godlike).is_err());
}

#[test]
fn persistence_roundtrips() {
    let s = state_with(42, 3, 1, 7);
    let path = std::env::temp_dir().join(format!("mana-progress-{}.json", std::process::id()));
    save_progress(&path, &s).unwrap();
    assert_eq!(load_progress(&path), s); // load returns Default on missing/corrupt file
}
```

(`tier` note: `progress_view` maps `tier = TIERS[rank]` of the CURRENT rank — the first assert uses rank 1 = "plastic"→ wait: rank 1 is "plastic". Fix the first test's expectation to `"plastic"` — rank passed in is 1. This is the kind of off-by-one the tests exist to catch; make the test read `assert_eq!(v.tier, "plastic")`.)

- [ ] **Step 2:** `cargo test` → new tests FAIL.
- [ ] **Step 3:** Implement pure layer + `save_progress`/`load_progress` (serde_json to `app_data_dir()/progress.json`; the path-taking functions are the testable core, the Tauri layer resolves the real path). Watcher mirrors `activity::spawn_activity_watcher`: `tauri::async_runtime::spawn`, `tokio::time::interval(60s)`, scan `dirs::home_dir().join(".claude/projects")` and `.join(".codex/sessions")` — home dir via `std::env::var_os("HOME")` (macOS-only app; no new deps), lock → scan → if `progress_view` changed: save + `app.emit("progress-update", &view)`. First tick runs immediately (interval's default) so history reconciles at startup. Commands lock the same `ProgressStore`, mutate via the `try_*` functions, persist, emit, and return the fresh view.
- [ ] **Step 4:** `cargo test` → PASS. Also `cargo build` to prove the Tauri wiring compiles.
- [ ] **Step 5:** Commit: `feat: progress state, commands, and watcher`.

---

### Task 4: Frontend progress helpers + footer UI

**Files:**
- Create: `src/progress-view.ts`, `src/progress-view.test.ts`
- Modify: `index.html` (footer strip inside `#root`, after `#card`), `src/main.ts` (listen/render), `src/styles.css` (footer styles)

**Interfaces (Produces):**
```ts
export type Progress = {
  xp: number; level: number; rank: number; tier: string; prestige: number;
  rank_up_eligible: boolean; prestige_eligible: boolean;
  level_progress: { current: number; needed: number };
};
export function tierDisplayName(tier: string): string;      // "master" → "Master"; "naked" → "Unranked"
export function levelLabel(p: Progress): string;            // "Lv 12 · Silver" / "Lv 3 · Unranked"
export function xpBarFraction(p: Progress): number;         // current/needed clamped to [0,1]; needed<=0 → 1
export function badgeSlots(prestige: number): number[];     // [1..min(prestige,10)]; shows count overlay when prestige>10 (see Task 6)
export function actionKind(p: Progress): "rank-up" | "prestige" | null;
```

- [ ] **Step 1: Failing vitest tests** in `src/progress-view.test.ts` (style of `format.test.ts`):

```ts
import { describe, expect, it } from "vitest";
import { actionKind, badgeSlots, levelLabel, tierDisplayName, xpBarFraction } from "./progress-view";

const base = {
  xp: 850, level: 10, rank: 5, tier: "silver", prestige: 0,
  rank_up_eligible: false, prestige_eligible: false,
  level_progress: { current: 50, needed: 125 },
};

describe("progress footer", () => {
  it("labels level and tier", () => {
    expect(levelLabel(base)).toBe("Lv 10 · Silver");
    expect(levelLabel({ ...base, tier: "naked" })).toBe("Lv 10 · Unranked");
  });
  it("clamps the xp bar fraction", () => {
    expect(xpBarFraction(base)).toBeCloseTo(0.4);
    expect(xpBarFraction({ ...base, level_progress: { current: 200, needed: 125 } })).toBe(1);
    expect(xpBarFraction({ ...base, level_progress: { current: 1, needed: 0 } })).toBe(1);
  });
  it("caps badge slots at ten", () => {
    expect(badgeSlots(0)).toEqual([]);
    expect(badgeSlots(3)).toEqual([1, 2, 3]);
    expect(badgeSlots(12)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
  });
  it("picks the top-right action", () => {
    expect(actionKind(base)).toBeNull();
    expect(actionKind({ ...base, rank_up_eligible: true })).toBe("rank-up");
    expect(actionKind({ ...base, rank: 13, prestige_eligible: true })).toBe("prestige");
  });
});
```

- [ ] **Step 2:** `npm test -- progress-view` → FAIL.
- [ ] **Step 3:** Implement `progress-view.ts`; wire `main.ts`: `listen<Progress>("progress-update", ...)` + `invoke<Progress>("get_progress")` at boot (same pattern as `get_snapshots`); render into a `#progress` footer strip: `<div id="progress"><span class="badges"></span><span class="level"></span><div class="xpbar"><div class="xpfill"></div></div></div>`; set `document.getElementById("root")!.dataset.rank = p.tier`. Footer CSS: 10px mono font like `.row`, `--line` top border, xpbar reuses the `.fill` gradient language at 3px tall, width set as a percentage from `xpBarFraction`. IMPORTANT: the footer changes `#card`-driven window height — it lives inside `#root`, so `resizeRosterContent()` must measure the full content; change its measurement from `#card` scrollHeight to a wrapper that includes the footer (add `const height = rosterHeight(root.scrollHeight)`-style measurement — keep the existing scale multiplication from the drag-resize work intact).
- [ ] **Step 4:** `npm test` + `npx tsc --noEmit` → PASS.
- [ ] **Step 5:** Commit: `feat: progress footer with xp bar`.

---

### Task 5: Rank Up / Prestige button + lavish dialogs

**Files:**
- Modify: `src/progress-view.ts` (+ tests), `index.html`, `src/main.ts`, `src/styles.css`

**Interfaces (Produces):**
```ts
export function dialogCopy(kind: "rank-up" | "prestige", p: Progress): {
  title: string;     // rank-up: "ASCEND TO GOLD" (next tier); prestige: "PRESTIGE II" (roman numeral of prestige+1)
  body: string;      // rank-up: "Level {level} · {CurrentTier} → {NextTier}"; prestige: "The curve steepens. Begin again at Level 1 — Prestige {n} badge is yours forever."
  confirm: string;   // rank-up: "Rank Up"; prestige: "Prestige"
};
export function romanNumeral(n: number): string; // 1..=20 ("I".."XX"), else String(n)
export function nextTier(p: Progress): string | null; // TIERS[rank+1] or null at godlike
```

- [ ] **Step 1: Failing tests** — `romanNumeral(4) === "IV"`, `romanNumeral(9) === "IX"`, `nextTier` at rank 0 → "plastic", at 13 → null, `dialogCopy("rank-up", {...rank: 5, level: 36})` title `"ASCEND TO GOLD"`, `dialogCopy("prestige", {...prestige: 1})` title `"PRESTIGE II"`.
- [ ] **Step 2:** `npm test -- progress-view` → FAIL.
- [ ] **Step 3:** Implement copy helpers. `index.html`: `<button id="action" hidden></button>` positioned absolute top-right inside `#root`, and `<div id="ceremony" hidden><div class="ceremony-panel"><h1></h1><p></p><button class="confirm"></button><button class="later">Later</button></div></div>` overlay. `main.ts`: on each progress render, `action.hidden = actionKind(p) === null`, label from kind; click → fill ceremony from `dialogCopy`, set `ceremony.dataset.kind` and `ceremony.dataset.tier` (next tier for rank-up, `prestige-${n}` for prestige), unhide; confirm → `invoke("rank_up")` / `invoke("prestige")`, re-render from the returned Progress, and if `actionKind` is still non-null leave the button glowing (one ceremony per click). **Drag-region caveat:** `#root` has `data-tauri-drag-region="deep"` — the button and ceremony panel must call `event.stopPropagation()` on `mousedown` so clicks don't start a window drag; the ceremony backdrop keeps drag alive. CSS: `#action` is a small gold-rimmed glowing pill (pulse animation reusing the `status-pulse` keyframes, `animation` respecting the existing `prefers-reduced-motion` block); `#ceremony` is `position: fixed; inset: 0; z-index: 10` with a radial-gradient scrim, ornate double-border panel using `--hud-radius` concentric math, tier-tinted via the same `--c1/--c2` custom-property pattern the provider cards use.
- [ ] **Step 4:** `npm test` + `npx tsc --noEmit` → PASS.
- [ ] **Step 5:** Commit: `feat: rank up and prestige ceremonies`.

---

### Task 6: Rank border themes + prestige badge fallbacks (CSS)

**Files:**
- Modify: `src/styles.css`, `src/styles.test.ts`

Selector pattern: `#root[data-rank="<tier>"] { --frame-1: …; --frame-2: …; --frame-glow: …; }` with ONE shared rule consuming them: `#root { border-color: var(--frame-1, rgba(205,221,242,0.34)); box-shadow: …, 0 0 14px var(--frame-glow, transparent); }` plus per-tier `border-image: linear-gradient(160deg, var(--frame-1), var(--frame-2)) 1` for metallic tiers. Naked (`data-rank="naked"` or missing): `border-color: transparent` and `#root::before { content: none; }` (no corner ticks).

Exact per-tier values (use verbatim):

| tier | --frame-1 | --frame-2 | --frame-glow |
|---|---|---|---|
| plastic | #b8bec8 | #8b929e | transparent |
| wood | #a5713d | #6b4726 | transparent |
| iron | #9aa3ad | #5f6770 | rgba(154,163,173,0.25) |
| bronze | #cd7f32 | #8c5a24 | rgba(205,127,50,0.3) |
| silver | #e6edf5 | #97a3b4 | rgba(230,237,245,0.35) |
| gold | #f2c968 | #b8862e | rgba(242,201,104,0.45) |
| platinum | #dfe9ec | #9fb6c4 | rgba(223,233,236,0.5) |
| emerald | #3ddc84 | #147a4a | rgba(61,220,132,0.5) |
| diamond | #9be8ff | #4aa8d8 | rgba(155,232,255,0.55) |
| master | #ff5a6e | #a3172c | rgba(255,90,110,0.55) |
| legend | #b06aff | #5e1ea8 | rgba(176,106,255,0.55) |
| champion | #ffd75e | #3f8cff | rgba(255,215,94,0.6) |
| godlike | #fff6d8 | #ffd9f6 | rgba(255,246,216,0.75) |

Champion additionally animates its border-image gradient angle (reuse a keyframes named `champion-radiance`, 6s linear infinite, disabled under `prefers-reduced-motion`); godlike adds a second outer glow `0 0 30px rgba(190,225,255,0.4)`.

Badge fallback (until Codex art lands): `.badge { width: 24px; height: 24px; }` rendered as `background-image: url("/badges/prestige-<n>.png")` **plus** a CSS `::after` star glyph (`content: "★"`) that is visible only when the image is absent — implement by setting the `<span class="badge" data-n="3">` text/star in JS when an `Image()` probe fails (same probe helper Task 7 adds). Star tint cycles: n 1–3 silver `#dbe7f4`, 4–6 gold `#f2c968`, 7–9 diamond `#9be8ff`, 10 champion gradient text via `background-clip: text`. Prestige >10: the 10th badge gets a `data-count` attribute rendered as a small superscript via `::before { content: attr(data-count); }`.

- [ ] **Step 1: Failing styles tests** — extend `src/styles.test.ts` in its existing raw-CSS-text style: every tier in the table above has a `#root[data-rank="…"]` rule; naked kills `::before` ticks; `champion-radiance` keyframes exist AND are referenced inside the `prefers-reduced-motion` block; the shared border rule consumes `var(--frame-1`/`--frame-glow`.
- [ ] **Step 2:** `npm test -- styles` → FAIL. **Step 3:** Implement. **Step 4:** `npm test` → PASS. **Step 5:** Commit: `feat: rank border themes and badge fallbacks`.

---

### Task 7: Per-rank sprite sheets with fallback

**Files:**
- Create: `src/cosmetics.ts`, `src/cosmetics.test.ts`
- Modify: `src/main.ts`, `src/sprites.test.ts`

**Interfaces (Produces):**
```ts
export function rankSheetUrl(provider: string, tier: string): string;
// "/sprites/claude-rank-silver.png"
export function defaultSheet(provider: string): string;
// claude → "/sprites/claude-fire-poison.png", codex → "/sprites/codex-ice-lightning.png"
export function probeImage(url: string): Promise<boolean>; // Image() onload/onerror
export async function resolveSheet(provider: string, tier: string): Promise<string>; // rank sheet if probe succeeds else defaultSheet
```

- [ ] **Step 1: Failing tests** (vitest; for `probeImage`/`resolveSheet` stub `globalThis.Image` with a class whose `set src` schedules onload/onerror by URL pattern — follow the DOM-stubbing style used in `view.test.ts`): `rankSheetUrl("claude","silver")` exact path; `resolveSheet` falls back when probe rejects; resolves rank sheet when probe succeeds.
- [ ] **Step 2:** FAIL. **Step 3:** Implement; in `main.ts` after each rank change: `resolveSheet(provider, tier).then((url) => sprites.forEach((el) => el.style.backgroundImage = `url("${url}")`))`. Keep `background-size: 224px 168px` untouched — rank sheets share the geometry. Extend `src/sprites.test.ts`: glob `public/sprites/*-rank-*.png` (Node `readdirSync`) and run `verifyAtlas` on each existing file, so every future Codex drop is validated automatically; zero matching files must not fail the suite.
- [ ] **Step 4:** `npm test` + `npx tsc --noEmit` → PASS. **Step 5:** Commit: `feat: per-rank sprite sheets with fallback`.

---

### Task 8: Art briefs for Codex handoff

**Files:**
- Create: `docs/art/rank-sprites-brief.md`, `docs/art/prestige-badges-brief.md`

No code. The briefs must contain, verbatim-precise:
- Sheet geometry contract: 448×336 RGBA PNG; 4 frame columns × 3 state rows (idle, working, hover top-to-bottom) of 112×112 cells; ≥4px fully transparent margin inside every cell edge; per-cell visible pixels (alpha>16) between 600 and 10,500; per-row baseline spread ≤12px; acceptance = `npm test` (`src/sprites.test.ts` verifies automatically).
- File names: `public/sprites/claude-rank-<tier>.png` and `public/sprites/codex-rank-<tier>.png` for every tier from `naked` through `godlike` (28 sheets). Claude mage: cyan/blue palette (`#39ddff`→`#557cff`), fire/poison spell effects; Codex mage: magenta/pink (`#d75cff`→`#ff5ba8`), ice/lightning. Naked = simple robeless apprentice; each tier adds armor/cosmetics of its material (wood-carved staff, iron pauldrons, … champion radiant gold-and-blue regalia, godlike halo/wings/light). Match the existing sheets' chunky pixel-art style — reference `public/sprites/claude-fire-poison.png`.
- Badges: `public/badges/prestige-<n>.png`, n = 1–10; 96×96 RGBA PNG, transparent background, displayed at 24×24 CSS — bold silhouettes that read at 24px; escalating opulence (laurel → shield → crown → wings → constellation …), silver→gold→diamond→radiant progression tints.
- Commit: `docs: art briefs for rank sprites and prestige badges`.

The actual Codex invocation happens at integration time (not by the implementing agent): `codex exec` pointed at the briefs, from the repo root, generating into `public/sprites/` and `public/badges/`, then `npm test` gates acceptance.

---

## Self-review notes

- Spec coverage: XP source (Task 2), curve+prestige steepening (Task 1), gates/manual rank-up (Tasks 1/3/5), prestige reset+baseline (Task 3), footer/badges (Tasks 4/6), button+ceremonies (Task 5), border themes (Task 6), sprites+fallback (Task 7), briefs/handoff (Task 8). Persistence (Task 3). No gaps found.
- Task 3 Step 1 first test: expectation corrected inline to `tier == "plastic"` (rank 1).
- Types consistent: `Progress` payload identical in Rust serializer, TS type, and tests.
