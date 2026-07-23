use crate::progress_store::save_state;
pub use crate::progress_store::ProgressStore;

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

/// Total XP required to reach `level`: floor(0.8 · L³ · 1.5^prestige), as the
/// exact integer form `4·L³·3^p / (5·2^p)`. u128 intermediates because
/// `3^p · L³` overflows u64 long before the inputs look unreasonable.
pub fn xp_for_level(level: u32, prestige: u32) -> u64 {
    if level <= 1 {
        return 0;
    }
    let l = level as u128;
    let p = prestige.min(40); // 1.5^40 already dwarfs any real token count
    let num = 4u128
        .saturating_mul(l * l * l)
        .saturating_mul(3u128.saturating_pow(p));
    let den = 5u128 * 2u128.saturating_pow(p);
    u64::try_from(num / den).unwrap_or(u64::MAX)
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
#[serde(default)]
pub struct TallyState {
    pub total_tokens: u64,
    pub claude_offsets: std::collections::HashMap<String, u64>, // path -> consumed byte offset
    pub codex_offsets: std::collections::HashMap<String, u64>,
    pub codex_totals: std::collections::HashMap<String, u64>, // path -> last cumulative total_tokens
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

/// Token count for one Claude transcript line. Every `message.usage` field
/// counts — cache reads dominate real sessions and are spent tokens too.
/// Malformed lines yield 0 but still consume offset, so a bad line can never
/// wedge the scanner.
fn claude_line_tokens(line: &str) -> u64 {
    let Ok(v) = serde_json::from_str::<serde_json::Value>(line) else {
        return 0;
    };
    let Some(usage) = v.get("message").and_then(|m| m.get("usage")) else {
        return 0;
    };
    [
        "input_tokens",
        "cache_creation_input_tokens",
        "cache_read_input_tokens",
        "output_tokens",
    ]
    .iter()
    .map(|key| usage.get(key).and_then(|n| n.as_u64()).unwrap_or(0))
    .sum()
}

pub fn scan_claude_dir(dir: &std::path::Path, state: &mut TallyState) {
    for path in jsonl_files(dir) {
        let key = path.to_string_lossy().into_owned();
        let offset = state.claude_offsets.get(&key).copied().unwrap_or(0);
        let Some((text, new_offset)) = complete_lines_from(&path, offset) else {
            continue;
        };
        let added: u64 = text.lines().map(claude_line_tokens).sum();
        state.total_tokens = state.total_tokens.saturating_add(added);
        state.claude_offsets.insert(key, new_offset);
    }
}

/// Cumulative total from one Codex `token_count` event line. The field lives
/// at `payload.info.total_token_usage.total_tokens`, but some builds emit
/// `info` at the top level — both shapes exist in the wild.
fn codex_line_total(line: &str) -> Option<u64> {
    let v = serde_json::from_str::<serde_json::Value>(line).ok()?;
    let info = v
        .get("payload")
        .and_then(|p| p.get("info"))
        .or_else(|| v.get("info"))?;
    info.get("total_token_usage")?.get("total_tokens")?.as_u64()
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
        let Some(latest) = text.lines().rev().find_map(codex_line_total) else {
            continue;
        };
        let stored = state.codex_totals.get(&key).copied().unwrap_or(0);
        if latest > stored {
            state.total_tokens = state.total_tokens.saturating_add(latest - stored);
        }
        // A latest below the stored total means truncation/rotation: resync
        // the baseline without counting anything.
        state.codex_totals.insert(key, latest);
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
    /// False until the first full scan banks pre-existing token history as
    /// the XP baseline. `serde(default)` deliberately re-baselines any state
    /// file written before this field existed: progression earned against
    /// lifetime history was unearned, so everybody starts back at zero.
    #[serde(default)]
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
    pub xp: u64,
    pub level: u32,
    pub rank: usize,
    pub tier: String,
    pub prestige: u32,
    pub rank_up_eligible: bool,
    pub prestige_eligible: bool,
    pub level_progress: LevelProgress,
}

/// XP since the last prestige: earlier tokens stay in the lifetime tally but
/// no longer count toward levels.
fn effective_xp(state: &ProgressState) -> u64 {
    state
        .tally
        .total_tokens
        .saturating_sub(state.prestige_token_floor)
        / TOKENS_PER_XP
}

pub fn progress_view(state: &ProgressState) -> Progress {
    let xp = effective_xp(state);
    let level = level_for_xp(xp, state.prestige);
    let floor = xp_for_level(level, state.prestige);
    let needed = xp_for_level(level + 1, state.prestige).saturating_sub(floor);
    Progress {
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

/// One-time reset after the first full scan: pre-existing token history is
/// banked as the XP floor and any progression derived from it is revoked.
/// Levels are meant to be earned from live usage, not imported.
pub fn initialize_baseline(state: &mut ProgressState) {
    state.rank = 0;
    state.prestige = 0;
    state.prestige_token_floor = state.tally.total_tokens;
    state.initialized = true;
}

/// Prestige: back to naked, curve steepens, and the token floor moves to
/// "now" so only future tokens level the next cycle.
pub fn try_prestige(state: &mut ProgressState) -> Result<(), String> {
    if !prestige_eligible(state.rank) {
        return Err("prestige requires the final tier".into());
    }
    state.prestige += 1;
    state.rank = 0;
    state.prestige_token_floor = state.tally.total_tokens;
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

#[tauri::command]
pub fn get_progress(store: tauri::State<'_, ProgressStore>) -> Progress {
    progress_view(&store.state.lock().unwrap())
}

#[tauri::command]
pub fn rank_up(
    app: tauri::AppHandle,
    store: tauri::State<'_, ProgressStore>,
) -> Result<Progress, String> {
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

/// Rescans the real session directories every 60s (first tick immediate, so
/// history reconciles at startup). Every tally/cursor delta is persisted,
/// while sub-XP token growth remains silent because only visible changes emit.
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
                let mut candidate = current.clone();
                scan_claude_dir(&claude_dir, &mut candidate.tally);
                scan_codex_dir(&codex_dir, &mut candidate.tally);
                if !candidate.initialized {
                    initialize_baseline(&mut candidate);
                }
                match commit_scanned_candidate(&mut current, candidate, |candidate| {
                    save_state(&store.paths, candidate)
                        .map_err(|error| format!("progress persistence failed: {error}"))
                }) {
                    Ok(event) => event,
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
    fn prestige_steepens_curve_by_1_5x_each_cycle() {
        assert_eq!(xp_for_level(10, 1), 1200); // 800 * 1.5
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
        assert!(!rank_up_eligible(200, 13)); // godlike: no more ranks
        assert!(!prestige_eligible(12));
        assert!(prestige_eligible(13));
    }

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

    #[test]
    fn codex_scan_adds_running_total_deltas_not_event_sums() {
        let dir = std::env::temp_dir().join(format!("mana-tally-codex-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("2026/session.jsonl");
        write(&file, &format!("{CODEX_EVENT_20K}\n"));
        let mut state = TallyState::default();
        scan_codex_dir(&dir, &mut state);
        assert_eq!(state.total_tokens, 20000);
        scan_codex_dir(&dir, &mut state);
        assert_eq!(state.total_tokens, 20000); // idempotent
        let mut f = std::fs::OpenOptions::new().append(true).open(&file).unwrap();
        use std::io::Write as _;
        writeln!(f, "{}", CODEX_EVENT_20K.replace("20000", "20900")).unwrap();
        scan_codex_dir(&dir, &mut state);
        assert_eq!(state.total_tokens, 20900); // delta 900, never 40900
    }

    fn state_with(tokens: u64, rank: usize, prestige: u32, floor: u64) -> ProgressState {
        ProgressState {
            rank,
            prestige,
            prestige_token_floor: floor,
            initialized: true,
            tally: TallyState {
                total_tokens: tokens,
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
        current.tally.codex_totals.insert("codex.jsonl".into(), 789);
        let before = current.clone();
        let mut candidate = current.clone();
        try_rank_up(&mut candidate).unwrap();
        candidate.tally.total_tokens += 42;
        candidate
            .tally
            .claude_offsets
            .insert("claude.jsonl".into(), 999);

        let result = commit_candidate(&mut current, candidate, |_| Err("disk full".into()));

        assert!(result.is_err());
        assert_eq!(current, before);
    }

    #[test]
    fn scanned_sub_xp_changes_are_persisted_without_visible_event() {
        let mut current = state_with(1_000, 0, 0, 0);
        let old_view = progress_view(&current);
        let mut candidate = current.clone();
        candidate.tally.total_tokens += 42;
        candidate
            .tally
            .claude_offsets
            .insert("claude.jsonl".into(), 321);
        assert_eq!(progress_view(&candidate), old_view);
        let persisted = std::cell::RefCell::new(None);

        let event =
            commit_scanned_candidate(&mut current, candidate.clone(), |state: &ProgressState| {
                persisted.replace(Some(state.clone()));
                Ok(())
            })
            .unwrap();

        assert_eq!(persisted.into_inner(), Some(candidate.clone()));
        assert_eq!(current, candidate);
        assert_eq!(event, None);
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
    fn first_scan_banks_history_and_zeroes_progression() {
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
    fn legacy_state_files_rebaseline_on_load() {
        // Progression earned against pre-update lifetime history was unearned;
        // files written before `initialized` existed must come back false so
        // the next scan resets them to zero.
        let json = r#"{"rank":5,"prestige":1,"prestige_token_floor":7,"tally":{"total_tokens":42,"claude_offsets":{},"codex_offsets":{},"codex_totals":{}}}"#;
        let s: ProgressState = serde_json::from_slice(json.as_bytes()).unwrap();
        assert!(!s.initialized);
    }

    #[test]
    fn codex_shrunken_total_contributes_zero_and_resets_baseline() {
        let dir = std::env::temp_dir().join(format!("mana-tally-shrunk-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let file = dir.join("session.jsonl");
        write(&file, &format!("{CODEX_EVENT_20K}\n"));
        let key = file.to_string_lossy().into_owned();
        let mut state = TallyState::default();
        state.codex_totals.insert(key.clone(), 50000);
        scan_codex_dir(&dir, &mut state);
        assert_eq!(state.total_tokens, 0); // truncated/rotated file: never double-count
        assert_eq!(state.codex_totals[&key], 20000); // baseline resynced
    }
}
