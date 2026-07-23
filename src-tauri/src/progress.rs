pub use crate::progress_store::ProgressStore;
use crate::progress_store::{publish_rebuilt_state, save_state};

/// Cosmetic tiers, indexed by rank. Rank 0 is the unadorned starting state;
/// every later tier only changes cosmetics, never behavior.
pub const TIERS: [&str; 14] = [
    "naked", "plastic", "wood", "iron", "bronze", "silver", "gold", "platinum", "emerald",
    "diamond", "master", "legend", "champion", "godlike",
];

/// Level required to *reach* the same-index rank. Gates widen so late tiers
/// stay aspirational even though the XP curve is already cubic.
pub const GATES: [u32; 14] = [0, 5, 10, 15, 21, 28, 36, 45, 55, 66, 78, 91, 105, 120];

pub const TOKENS_PER_XP: u64 = 1000;

/// Total XP required to reach `level`, using three exact prestige bands:
/// 1.5x for the first three cycles, 1.75x for the next three, then 2x.
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

/// Largest level whose threshold is within `xp`. Linear walk: the 999 cap
/// bounds it, and the curve makes real values land in the low hundreds.
pub fn level_for_xp(xp: u64, prestige: u32) -> u32 {
    let mut level = 1;
    while level < 999 && xp_for_level(level + 1, prestige) <= xp {
        level += 1;
    }
    level
}

/// Whether the next rank's gate is met. Rank never auto-advances; this only
/// gates the manual Rank Up action.
pub fn rank_up_eligible(level: u32, rank: usize) -> bool {
    rank < TIERS.len() - 1 && level >= GATES[rank + 1]
}

/// Prestige is offered only at the final tier.
pub fn prestige_eligible(rank: usize) -> bool {
    rank == TIERS.len() - 1
}

/// Lifetime token tally plus the per-file cursors that make re-scans
/// incremental: byte offsets for append-only Claude transcripts, and last
/// seen cumulative totals for Codex sessions (whose `token_count` events
/// carry running totals, not deltas).
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TallyState {
    pub output_tokens: u64,
    pub claude_offsets: std::collections::HashMap<String, u64>, // path -> consumed byte offset
    pub codex_offsets: std::collections::HashMap<String, u64>,
    pub codex_output_totals: std::collections::HashMap<String, u64>, // path -> last cumulative output total
}

/// All `*.jsonl` files under `dir`, found with a manual stack — a handful of
/// session directories does not justify a recursion-crate dependency.
fn jsonl_files(dir: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(&d) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|ext| ext == "jsonl") {
                files.push(path);
            }
        }
    }
    files
}

/// Complete lines appended to `path` since `offset`, plus the new offset.
/// Live sessions write mid-line, so bytes after the last newline stay
/// un-consumed for the next scan. An offset past EOF means the file was
/// truncated or rotated; start over from zero rather than skip it forever.
fn complete_lines_from(path: &std::path::Path, offset: u64) -> Option<(String, u64)> {
    use std::io::{Read as _, Seek as _};
    let len = std::fs::metadata(path).ok()?.len();
    let start = if offset > len { 0 } else { offset };
    let mut file = std::fs::File::open(path).ok()?;
    file.seek(std::io::SeekFrom::Start(start)).ok()?;
    let mut buf = Vec::with_capacity((len - start) as usize);
    file.read_to_end(&mut buf).ok()?;
    let consumed = buf.iter().rposition(|&b| b == b'\n')? + 1;
    buf.truncate(consumed);
    Some((
        String::from_utf8_lossy(&buf).into_owned(),
        start + consumed as u64,
    ))
}

/// Output tokens for one Claude transcript line. Malformed lines yield 0 but
/// still consume their offset, so a bad line can never wedge the scanner.
fn claude_line_output_tokens(line: &str) -> u64 {
    serde_json::from_str::<serde_json::Value>(line)
        .ok()
        .and_then(|value| {
            value
                .get("message")?
                .get("usage")?
                .get("output_tokens")?
                .as_u64()
        })
        .unwrap_or(0)
}

pub fn scan_claude_dir(dir: &std::path::Path, state: &mut TallyState) {
    for path in jsonl_files(dir) {
        let key = path.to_string_lossy().into_owned();
        let offset = state.claude_offsets.get(&key).copied().unwrap_or(0);
        let Some((text, new_offset)) = complete_lines_from(&path, offset) else {
            continue;
        };
        let added: u64 = text.lines().map(claude_line_output_tokens).sum();
        state.output_tokens = state.output_tokens.saturating_add(added);
        state.claude_offsets.insert(key, new_offset);
    }
}

/// Cumulative output total from one Codex `token_count` event line. Some
/// builds emit `info` at the top level, so both observed shapes are accepted.
fn codex_line_output_total(line: &str) -> Option<u64> {
    let value = serde_json::from_str::<serde_json::Value>(line).ok()?;
    let info = value
        .get("payload")
        .and_then(|payload| payload.get("info"))
        .or_else(|| value.get("info"))?;
    let usage = info.get("total_token_usage")?;
    let output = usage
        .get("output_tokens")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    let reasoning = usage
        .get("reasoning_output_tokens")
        .and_then(|value| value.as_u64())
        .unwrap_or(0);
    Some(output.saturating_add(reasoning))
}

pub fn scan_codex_dir(dir: &std::path::Path, state: &mut TallyState) {
    for path in jsonl_files(dir) {
        let key = path.to_string_lossy().into_owned();
        let offset = state.codex_offsets.get(&key).copied().unwrap_or(0);
        let Some((text, new_offset)) = complete_lines_from(&path, offset) else {
            continue;
        };
        state.codex_offsets.insert(key.clone(), new_offset);
        // Only the latest total matters: it is a per-file running total, so
        // summing events would multiply-count every earlier token.
        let Some(latest) = text.lines().rev().find_map(codex_line_output_total) else {
            continue;
        };
        let stored = state.codex_output_totals.get(&key).copied().unwrap_or(0);
        if latest > stored {
            state.output_tokens = state.output_tokens.saturating_add(latest - stored);
        }
        // A latest below the stored total means truncation/rotation: resync
        // the baseline without counting anything.
        state.codex_output_totals.insert(key, latest);
    }
}

/// The whole persisted progression: cosmetic choices (rank/prestige), the
/// prestige XP baseline, and the tally cursors. Saved as one document so the
/// on-disk snapshot is always internally consistent — tokens and the offsets
/// that produced them can never diverge.
#[derive(Debug, Default, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgressState {
    pub rank: usize,
    pub prestige: u32,
    pub prestige_token_floor: u64,
    /// False only for a genuinely new install until its first full scan banks
    /// pre-existing token history as the XP baseline. Every successfully
    /// decoded legacy progress file is treated as initialized.
    pub initialized: bool,
    pub tally: TallyState,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct LevelProgress {
    pub current: u64,
    pub needed: u64,
}

/// Everything the frontend renders, derived on demand — the UI never does
/// progression math of its own.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Progress {
    #[serde(serialize_with = "serialize_u64_decimal")]
    pub lifetime_output_tokens: u64,
    pub xp: u64,
    pub level: u32,
    pub rank: usize,
    pub tier: String,
    pub prestige: u32,
    pub rank_up_eligible: bool,
    pub prestige_eligible: bool,
    pub level_progress: LevelProgress,
}

fn serialize_u64_decimal<S>(value: &u64, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

/// XP since the last prestige: earlier tokens stay in the lifetime tally but
/// no longer count toward levels.
fn effective_xp(state: &ProgressState) -> u64 {
    state
        .tally
        .output_tokens
        .saturating_sub(state.prestige_token_floor)
        / TOKENS_PER_XP
}

pub fn progress_view(state: &ProgressState) -> Progress {
    let xp = effective_xp(state);
    let level = level_for_xp(xp, state.prestige);
    let floor = xp_for_level(level, state.prestige);
    let needed = xp_for_level(level + 1, state.prestige).saturating_sub(floor);
    Progress {
        lifetime_output_tokens: state.tally.output_tokens,
        xp,
        level,
        rank: state.rank,
        tier: TIERS[state.rank.min(TIERS.len() - 1)].to_string(),
        prestige: state.prestige,
        rank_up_eligible: rank_up_eligible(level, state.rank),
        prestige_eligible: prestige_eligible(state.rank),
        level_progress: LevelProgress {
            current: xp - floor,
            needed,
        },
    }
}

pub fn recalculate_from_output_history(state: &mut ProgressState) {
    let mut prestige = 0u32;
    let mut floor = 0u64;
    loop {
        let cost = prestige_cycle_token_cost(prestige);
        let remaining = state.tally.output_tokens.saturating_sub(floor);
        if cost == 0 || remaining < cost {
            break;
        }
        floor = floor
            .checked_add(cost)
            .expect("affordability guarantees a u64 sum");
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

/// Advances exactly one tier. Validation lives here, not in the UI: the
/// frontend button is cosmetic and a stale or forged invoke must not skip
/// gates.
pub fn try_rank_up(state: &mut ProgressState) -> Result<(), String> {
    let level = level_for_xp(effective_xp(state), state.prestige);
    if !rank_up_eligible(level, state.rank) {
        return Err(format!("level {level} has not reached the next rank gate"));
    }
    state.rank += 1;
    Ok(())
}

/// One-time initialization after a genuinely new install's first full scan.
/// Pre-existing output history is banked as the XP floor, so levels are earned
/// from live usage rather than imported. Saved and legacy progress is already
/// initialized and must never take this path.
pub fn initialize_baseline(state: &mut ProgressState) {
    state.rank = 0;
    state.prestige = 0;
    state.prestige_token_floor = state.tally.output_tokens;
    state.initialized = true;
}

/// Prestige spends the exact current cycle cost, preserving any surplus
/// output toward the next cycle.
pub fn try_prestige(state: &mut ProgressState) -> Result<(), String> {
    if !prestige_eligible(state.rank) {
        return Err("prestige requires the final tier".into());
    }
    let cost = prestige_cycle_token_cost(state.prestige);
    let effective_output = state
        .tally
        .output_tokens
        .saturating_sub(state.prestige_token_floor);
    if effective_output < cost {
        return Err("prestige requires the complete current cycle".into());
    }
    state.prestige_token_floor = state
        .prestige_token_floor
        .checked_add(cost)
        .expect("affordability guarantees a u64 floor");
    state.prestige = state.prestige.saturating_add(1);
    state.rank = 0;
    Ok(())
}

fn commit_candidate<F>(
    current: &mut ProgressState,
    candidate: ProgressState,
    persist: F,
) -> Result<Progress, String>
where
    F: FnOnce(&ProgressState) -> Result<(), String>,
{
    persist(&candidate)?;
    let view = progress_view(&candidate);
    *current = candidate;
    Ok(view)
}

/// Commits a scan only when state changed. A tally increment changes the
/// rendered lifetime output total even below an XP threshold, so it returns a
/// view for the watcher to emit in that case.
fn commit_scanned_candidate<F>(
    current: &mut ProgressState,
    candidate: ProgressState,
    persist: F,
) -> Result<Option<Progress>, String>
where
    F: FnOnce(&ProgressState) -> Result<(), String>,
{
    if candidate == *current {
        return Ok(None);
    }
    let previous_view = progress_view(current);
    let view = commit_candidate(current, candidate, persist)?;
    Ok((view != previous_view).then_some(view))
}

/// Scans session output incrementally. Only an uninitialized state from a
/// genuinely new install banks existing history; established rank, prestige,
/// and floor choices remain manual progression state.
fn scan_progress_dirs(
    claude_dir: &std::path::Path,
    codex_dir: &std::path::Path,
    state: &mut ProgressState,
) {
    scan_claude_dir(claude_dir, &mut state.tally);
    scan_codex_dir(codex_dir, &mut state.tally);
    if !state.initialized {
        initialize_baseline(state);
    }
}

fn scan_and_commit_progress(
    current: &mut ProgressState,
    rebuild_pending: bool,
    claude_dir: &std::path::Path,
    codex_dir: &std::path::Path,
    persist_normal: impl FnOnce(&ProgressState) -> Result<(), String>,
    persist_rebuild: impl FnOnce(&ProgressState) -> Result<(), String>,
) -> Result<(Option<Progress>, bool), String> {
    if rebuild_pending {
        let mut candidate = ProgressState::default();
        scan_claude_dir(claude_dir, &mut candidate.tally);
        scan_codex_dir(codex_dir, &mut candidate.tally);
        recalculate_from_output_history(&mut candidate);
        let view = commit_candidate(current, candidate, persist_rebuild)?;
        return Ok((Some(view), true));
    }

    let mut candidate = current.clone();
    scan_progress_dirs(claude_dir, codex_dir, &mut candidate);
    let event = commit_scanned_candidate(current, candidate, persist_normal)?;
    Ok((event, false))
}

fn require_completed_output_rebuild(rebuild_pending: bool) -> Result<(), String> {
    if rebuild_pending {
        Err("output history rebuild is still in progress".into())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn get_progress(store: tauri::State<'_, ProgressStore>) -> Progress {
    progress_view(&store.state.lock().unwrap())
}

#[tauri::command]
pub fn rank_up(
    app: tauri::AppHandle,
    store: tauri::State<'_, ProgressStore>,
) -> Result<Progress, String> {
    require_completed_output_rebuild(store.output_rebuild_pending())?;
    let view = {
        let mut current = store
            .state
            .lock()
            .map_err(|_| "progress state lock poisoned".to_string())?;
        let mut candidate = current.clone();
        try_rank_up(&mut candidate)?;
        commit_candidate(&mut current, candidate, |candidate| {
            save_state(&store.paths, candidate)
                .map_err(|error| format!("progress persistence failed: {error}"))
        })?
    };
    use tauri::Emitter as _;
    let _ = app.emit("progress-update", &view);
    Ok(view)
}

#[tauri::command]
pub fn prestige(
    app: tauri::AppHandle,
    store: tauri::State<'_, ProgressStore>,
) -> Result<Progress, String> {
    require_completed_output_rebuild(store.output_rebuild_pending())?;
    let view = {
        let mut current = store
            .state
            .lock()
            .map_err(|_| "progress state lock poisoned".to_string())?;
        let mut candidate = current.clone();
        try_prestige(&mut candidate)?;
        commit_candidate(&mut current, candidate, |candidate| {
            save_state(&store.paths, candidate)
                .map_err(|error| format!("progress persistence failed: {error}"))
        })?
    };
    use tauri::Emitter as _;
    let _ = app.emit("progress-update", &view);
    Ok(view)
}

/// Rescans the real session directories every 60s. The immediate first tick
/// rebuilds retained output history for a migrated store, or banks history for
/// a genuinely new install. Later tally/cursor deltas use ordinary v3 saves.
pub fn spawn_progress_watcher(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        use tauri::Manager as _;
        let Some(home) = std::env::var_os("HOME").map(std::path::PathBuf::from) else {
            return;
        };
        let claude_dir = home.join(".claude/projects");
        let codex_dir = home.join(".codex/sessions");
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            tick.tick().await;
            let event = {
                let store = app.state::<ProgressStore>();
                let mut current = store.state.lock().unwrap();
                let rebuild_pending = store.output_rebuild_pending();
                match scan_and_commit_progress(
                    &mut current,
                    rebuild_pending,
                    &claude_dir,
                    &codex_dir,
                    |candidate| {
                        save_state(&store.paths, candidate)
                            .map_err(|error| format!("progress persistence failed: {error}"))
                    },
                    |candidate| {
                        publish_rebuilt_state(&store.paths, candidate)
                            .map_err(|error| format!("progress persistence failed: {error}"))
                    },
                ) {
                    Ok((event, rebuild_completed)) => {
                        if rebuild_completed {
                            store.finish_output_rebuild();
                        }
                        event
                    }
                    Err(error) => {
                        eprintln!("{error}");
                        None
                    }
                }
            };
            if let Some(view) = event {
                use tauri::Emitter as _;
                let _ = app.emit("progress-update", &view);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_matches_cubic_fast_formula() {
        assert_eq!(xp_for_level(1, 0), 0);
        assert_eq!(xp_for_level(2, 0), 6); // floor(0.8*8)
        assert_eq!(xp_for_level(10, 0), 800);
        assert_eq!(xp_for_level(120, 0), 1_382_400);
    }

    #[test]
    fn tiered_curve_matches_exact_thresholds_through_prestige_ten() {
        let expected = [
            800, 1_200, 1_800, 2_700, 4_725, 8_268, 14_470, 28_940, 57_881, 115_762, 231_525,
        ];
        for (prestige, xp) in expected.into_iter().enumerate() {
            assert_eq!(xp_for_level(10, prestige as u32), xp);
        }
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
        assert!(!rank_up_eligible(200, 13)); // godlike: no more ranks
        assert!(!prestige_eligible(12));
        assert!(prestige_eligible(13));
    }

    fn write(path: &std::path::Path, contents: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }

    const CLAUDE_LINE: &str = r#"{"message":{"usage":{"input_tokens":10,"cache_creation_input_tokens":5,"cache_read_input_tokens":85,"output_tokens":100}}}"#;
    const CODEX_EVENT: &str = r#"{"type":"event_msg","payload":{"type":"token_count","info":{"total_token_usage":{"input_tokens":1000,"cached_input_tokens":900,"output_tokens":3,"reasoning_output_tokens":4,"total_tokens":20000}}}}"#;

    fn tally_test_dir(label: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("mana-output-{label}-{}", std::process::id(),));
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
        assert_eq!(state.output_tokens, 200);
        scan_claude_dir(&dir, &mut state);
        assert_eq!(state.output_tokens, 200);
        // an appended line adds only the delta, and a trailing partial line is not consumed
        let mut f = std::fs::OpenOptions::new()
            .append(true)
            .open(&file)
            .unwrap();
        use std::io::Write as _;
        write!(f, "{CLAUDE_LINE}\n{{\"partial").unwrap();
        scan_claude_dir(&dir, &mut state);
        assert_eq!(state.output_tokens, 300);
        let stored = *state.claude_offsets.values().next().unwrap();
        assert!(stored < std::fs::metadata(&file).unwrap().len());
    }

    #[test]
    fn codex_scan_counts_output_and_reasoning_deltas_only() {
        let dir = tally_test_dir("codex");
        let file = dir.join("session.jsonl");
        write(&file, &format!("{CODEX_EVENT}\n"));
        let mut state = TallyState::default();
        scan_codex_dir(&dir, &mut state);
        assert_eq!(state.output_tokens, 7);
        scan_codex_dir(&dir, &mut state);
        assert_eq!(state.output_tokens, 7);
        let appended = CODEX_EVENT
            .replace("\"output_tokens\":3", "\"output_tokens\":30")
            .replace(
                "\"reasoning_output_tokens\":4",
                "\"reasoning_output_tokens\":20",
            );
        append_line(&file, &appended);
        scan_codex_dir(&dir, &mut state);
        assert_eq!(state.output_tokens, 50);
    }

    #[test]
    fn scanners_ignore_malformed_negative_and_non_integer_values() {
        let claude_dir = tally_test_dir("claude-invalid");
        write(
            &claude_dir.join("session.jsonl"),
            concat!(
                "{\"message\":{\"usage\":{\"output_tokens\":-1}}}\n",
                "{\"message\":{\"usage\":{\"output_tokens\":1.5}}}\n",
                "{\"message\":{\"usage\":{\"output_tokens\":\"100\"}}}\n",
                "not json\n"
            ),
        );
        let mut claude = TallyState::default();
        scan_claude_dir(&claude_dir, &mut claude);
        assert_eq!(claude.output_tokens, 0);

        let codex_dir = tally_test_dir("codex-invalid");
        write(
            &codex_dir.join("session.jsonl"),
            concat!(
                "{\"payload\":{\"info\":{\"total_token_usage\":{\"output_tokens\":-1,\"reasoning_output_tokens\":-2}}}}\n",
                "{\"payload\":{\"info\":{\"total_token_usage\":{\"output_tokens\":1.5,\"reasoning_output_tokens\":\"2\"}}}}\n",
                "not json\n"
            ),
        );
        let mut codex = TallyState::default();
        scan_codex_dir(&codex_dir, &mut codex);
        assert_eq!(codex.output_tokens, 0);
    }

    fn state_with(tokens: u64, rank: usize, prestige: u32, floor: u64) -> ProgressState {
        ProgressState {
            rank,
            prestige,
            prestige_token_floor: floor,
            initialized: true,
            tally: TallyState {
                output_tokens: tokens,
                ..Default::default()
            },
        }
    }

    #[test]
    fn progress_view_derives_everything() {
        // 800_000 tokens = 800 xp = level 10 at prestige 0
        let v = progress_view(&state_with(800_000, 1, 0, 0));
        assert_eq!(
            (v.xp, v.level, v.tier, v.rank_up_eligible),
            (800, 10, "plastic".into(), true) // rank 1 = plastic; gate for rank 2 (wood) is 10
        );
        assert_eq!(
            v.level_progress.needed,
            xp_for_level(11, 0) - xp_for_level(10, 0)
        );
        assert_eq!(v.lifetime_output_tokens, 800_000);
    }

    #[test]
    fn progress_serializes_lifetime_output_tokens_as_a_decimal_string() {
        let value = serde_json::to_value(progress_view(&state_with(u64::MAX, 0, 0, 0))).unwrap();
        assert_eq!(value["lifetime_output_tokens"], u64::MAX.to_string());
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
    fn progression_commands_reject_while_output_rebuild_is_pending() {
        assert_eq!(
            require_completed_output_rebuild(true).unwrap_err(),
            "output history rebuild is still in progress"
        );
        assert!(require_completed_output_rebuild(false).is_ok());
    }

    #[test]
    fn failed_persistence_does_not_commit_candidate() {
        let mut current = state_with(800_000, 0, 0, 0);
        current
            .tally
            .claude_offsets
            .insert("claude.jsonl".into(), 123);
        current
            .tally
            .codex_offsets
            .insert("codex.jsonl".into(), 456);
        current
            .tally
            .codex_output_totals
            .insert("codex.jsonl".into(), 789);
        let before = current.clone();
        let mut candidate = current.clone();
        try_rank_up(&mut candidate).unwrap();
        candidate.tally.output_tokens += 42;
        candidate
            .tally
            .claude_offsets
            .insert("claude.jsonl".into(), 999);

        let result = commit_candidate(&mut current, candidate, |_| Err("disk full".into()));

        assert!(result.is_err());
        assert_eq!(current, before);
    }

    #[test]
    fn scanned_sub_xp_changes_are_persisted_with_lifetime_output_event() {
        let mut current = state_with(1_000, 0, 0, 0);
        let old_view = progress_view(&current);
        let mut candidate = current.clone();
        candidate.tally.output_tokens += 42;
        candidate
            .tally
            .claude_offsets
            .insert("claude.jsonl".into(), 321);
        assert_ne!(progress_view(&candidate), old_view);
        let persisted = std::cell::RefCell::new(None);

        let event =
            commit_scanned_candidate(&mut current, candidate.clone(), |state: &ProgressState| {
                persisted.replace(Some(state.clone()));
                Ok(())
            })
            .unwrap();

        assert_eq!(persisted.into_inner(), Some(candidate.clone()));
        assert_eq!(current, candidate);
        assert_eq!(event, Some(progress_view(&candidate)));
    }

    #[test]
    fn ordinary_scan_candidate_preserves_initialized_manual_progression() {
        let claude_dir = tally_test_dir("manual-progression-claude");
        let codex_dir = tally_test_dir("manual-progression-codex");
        write(
            &claude_dir.join("session.jsonl"),
            &format!("{CLAUDE_LINE}\n"),
        );

        let first = prestige_cycle_token_cost(0);
        let second = prestige_cycle_token_cost(1);
        let third = prestige_cycle_token_cost(2);
        let mut candidate = state_with(first + second + third, 4, 1, first);
        let manual_progression = (
            candidate.rank,
            candidate.prestige,
            candidate.prestige_token_floor,
        );

        scan_progress_dirs(&claude_dir, &codex_dir, &mut candidate);

        assert_eq!(candidate.tally.output_tokens, first + second + third + 100);
        assert_eq!(
            (
                candidate.rank,
                candidate.prestige,
                candidate.prestige_token_floor,
            ),
            manual_progression,
        );
    }

    #[test]
    fn pending_rebuild_scans_all_history_and_commits_before_flag_clear() {
        let claude_dir = tally_test_dir("rebuild-claude");
        let codex_dir = tally_test_dir("rebuild-codex");
        let first_cycle = prestige_cycle_token_cost(0);
        let retained_claude = CLAUDE_LINE.replace(
            "\"output_tokens\":100",
            &format!("\"output_tokens\":{first_cycle}"),
        );
        write(
            &claude_dir.join("retained/session.jsonl"),
            &format!("{retained_claude}\n"),
        );
        write(
            &codex_dir.join("retained/session.jsonl"),
            &format!("{CODEX_EVENT}\n"),
        );

        let mut current = ProgressState::default();
        let rebuild_pending = std::cell::Cell::new(true);
        let persisted = std::cell::RefCell::new(None);
        let (event, rebuild_completed) = scan_and_commit_progress(
            &mut current,
            rebuild_pending.get(),
            &claude_dir,
            &codex_dir,
            |_| panic!("a rebuild must not use ordinary persistence"),
            |state| {
                assert!(rebuild_pending.get());
                persisted.replace(Some(state.clone()));
                Ok(())
            },
        )
        .unwrap();
        if rebuild_completed {
            rebuild_pending.set(false);
        }

        assert_eq!(current.tally.output_tokens, first_cycle + 7);
        assert!(!current.tally.claude_offsets.is_empty());
        assert!(!current.tally.codex_offsets.is_empty());
        assert_eq!(current.tally.codex_output_totals.values().next(), Some(&7));
        assert!(current.initialized);
        assert_eq!(current.prestige, 1);
        assert_eq!(current.prestige_token_floor, first_cycle);
        assert_eq!(persisted.into_inner(), Some(current.clone()));
        assert_eq!(event, Some(progress_view(&current)));
        assert!(rebuild_completed);
        assert!(!rebuild_pending.get());
    }

    #[test]
    fn failed_rebuild_persistence_keeps_live_state_and_pending_flag() {
        let claude_dir = tally_test_dir("failed-rebuild-claude");
        let codex_dir = tally_test_dir("failed-rebuild-codex");
        write(
            &claude_dir.join("retained/session.jsonl"),
            &format!("{CLAUDE_LINE}\n"),
        );
        write(
            &codex_dir.join("retained/session.jsonl"),
            &format!("{CODEX_EVENT}\n"),
        );

        let mut current = state_with(42, 0, 0, 0);
        let before = current.clone();
        let rebuild_pending = std::cell::Cell::new(true);
        let result = scan_and_commit_progress(
            &mut current,
            rebuild_pending.get(),
            &claude_dir,
            &codex_dir,
            |_| panic!("a rebuild must not use ordinary persistence"),
            |_| Err("disk full".into()),
        );

        assert_eq!(result.unwrap_err(), "disk full");
        assert_eq!(current, before);
        assert!(rebuild_pending.get());
    }

    #[test]
    fn recalculation_spends_complete_cycles_and_keeps_remainder() {
        let first = prestige_cycle_token_cost(0);
        let second = prestige_cycle_token_cost(1);
        let remainder = xp_for_level(10, 2) * TOKENS_PER_XP;
        let mut state = state_with(first + second + remainder, 13, 8, 99);
        recalculate_from_output_history(&mut state);
        assert_eq!(
            (state.prestige, state.prestige_token_floor),
            (2, first + second)
        );
        assert_eq!(progress_view(&state).level, 10);
        assert_eq!(state.rank, 2);
    }

    #[test]
    fn recalculation_handles_zero_exact_final_level_multiple_cycles_and_maximum() {
        let mut zero = state_with(0, 13, 9, 99);
        recalculate_from_output_history(&mut zero);
        assert_eq!(
            (zero.prestige, zero.prestige_token_floor, zero.rank),
            (0, 0, 0)
        );

        let exact_level = prestige_cycle_token_cost(0);
        let mut exact = state_with(exact_level, 0, 0, 0);
        recalculate_from_output_history(&mut exact);
        assert_eq!(
            (exact.prestige, exact.prestige_token_floor, exact.rank),
            (1, exact_level, 0)
        );

        let first = prestige_cycle_token_cost(0);
        let second = prestige_cycle_token_cost(1);
        let third = prestige_cycle_token_cost(2);
        let mut multiple = state_with(first + second + third, 0, 0, 0);
        recalculate_from_output_history(&mut multiple);
        assert_eq!(
            (multiple.prestige, multiple.prestige_token_floor),
            (3, first + second + third)
        );

        let mut maximum = state_with(u64::MAX, 0, 0, 0);
        recalculate_from_output_history(&mut maximum);
        assert!(maximum.initialized);
        assert!(maximum.prestige > 0);
        assert!(maximum.prestige_token_floor <= u64::MAX);
    }

    #[test]
    fn prestige_spends_exact_cycle_cost_and_preserves_surplus() {
        let cost = prestige_cycle_token_cost(0);
        let surplus = xp_for_level(10, 1) * TOKENS_PER_XP;
        let mut state = state_with(cost + surplus, TIERS.len() - 1, 0, 0);
        try_prestige(&mut state).unwrap();
        assert_eq!(
            (state.prestige, state.rank, state.prestige_token_floor),
            (1, 0, cost)
        );
        assert_eq!(progress_view(&state).level, 10);
    }

    #[test]
    fn prestige_rejects_final_rank_without_the_complete_cycle_cost() {
        let cost = prestige_cycle_token_cost(0);
        let mut state = state_with(cost - 1, TIERS.len() - 1, 0, 0);
        assert!(try_prestige(&mut state).is_err());
    }

    #[test]
    fn prestige_requires_final_rank() {
        let mut not_godlike = state_with(u64::MAX, 12, 0, 0);
        assert!(try_prestige(&mut not_godlike).is_err());
    }

    #[test]
    fn initialization_banks_history_and_zeroes_progression() {
        let mut s = state_with(16_000_000_000, 13, 1, 0);
        s.initialized = false;
        initialize_baseline(&mut s);
        assert!(s.initialized);
        assert_eq!(
            (s.rank, s.prestige, s.prestige_token_floor),
            (0, 0, 16_000_000_000),
        );
        let v = progress_view(&s);
        assert_eq!((v.xp, v.level, v.rank_up_eligible), (0, 1, false));
    }

    #[test]
    fn codex_shrunken_total_contributes_zero_and_resets_baseline() {
        let dir = std::env::temp_dir().join(format!("mana-tally-shrunk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("session.jsonl");
        write(&file, &format!("{CODEX_EVENT}\n"));
        let key = file.to_string_lossy().into_owned();
        let mut state = TallyState::default();
        state.codex_output_totals.insert(key.clone(), 50000);
        scan_codex_dir(&dir, &mut state);
        assert_eq!(state.output_tokens, 0); // truncated/rotated file: never double-count
        assert_eq!(state.codex_output_totals[&key], 7); // baseline resynced
    }
}
