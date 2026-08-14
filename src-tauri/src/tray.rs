use std::collections::HashMap;
use std::sync::Mutex;

use tauri::Manager;

use crate::poll::{Snapshots, UsageSnapshot};

/// Last title written to the tray. Serializes concurrent poller updates and
/// skips writes when the title is unchanged.
#[derive(Default)]
pub struct TrayTitle(Mutex<Option<String>>);

fn remaining(used_percent: f64) -> i64 {
    (100.0 - used_percent).round().clamp(0.0, 100.0) as i64
}

fn provider_segment(glyph: &str, bar_ids: &[&str], snapshot: &UsageSnapshot) -> Option<String> {
    if !snapshot.authenticated {
        return None;
    }
    let find = |id: &str| snapshot.bars.iter().find(|b| b.id == id);
    let numbers: Vec<String> = bar_ids
        .iter()
        .filter_map(|id| find(id))
        .map(|b| remaining(b.used_percent).to_string())
        .collect();
    if numbers.is_empty() {
        return None;
    }
    Some(format!("{glyph} {}", numbers.join("·")))
}

pub fn tray_title(snapshots: &HashMap<String, UsageSnapshot>) -> Option<String> {
    let providers: [(&str, &str, &[&str]); 2] = [
        ("claude", "✳\u{fe0e}", &["weekly", "model"]),
        ("codex", "⎔", &["weekly"]),
    ];
    let segments: Vec<String> = providers
        .into_iter()
        .filter_map(|(key, glyph, bar_ids)| {
            snapshots.get(key).and_then(|s| provider_segment(glyph, bar_ids, s))
        })
        .collect();
    if segments.is_empty() {
        None
    } else {
        Some(segments.join(" "))
    }
}

/// Recompute the title from the full snapshot map and apply it when changed.
/// Lock order is TrayTitle, then Snapshots; the main thread only ever takes
/// Snapshots (`get_snapshots`), so holding TrayTitle across `set_title`'s
/// blocking main-thread round trip cannot deadlock.
pub fn update_tray_title(app: &tauri::AppHandle) {
    let title_state = app.state::<TrayTitle>();
    let mut last = title_state.0.lock().unwrap();
    let title = {
        let snapshots = app.state::<Snapshots>();
        let map = snapshots.lock().unwrap();
        tray_title(&map)
    };
    if *last == title {
        return;
    }
    let Some(tray) = app.tray_by_id("main") else {
        eprintln!("[mana] tray \"main\" missing; title not updated");
        return;
    };
    // tray-icon's macOS backend ignores a None title, so writing an empty
    // string is the only way to clear a previously shown title.
    match tray.set_title(Some(title.as_deref().unwrap_or(""))) {
        Ok(()) => *last = title,
        Err(error) => eprintln!("[mana] tray title update failed: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parsers::Bar;

    fn bar(id: &str, used_percent: f64) -> Bar {
        Bar {
            id: id.into(),
            label: id.into(),
            used_percent,
            resets_at: None,
        }
    }

    fn snap(authenticated: bool, bars: Vec<Bar>) -> UsageSnapshot {
        UsageSnapshot {
            provider: String::new(),
            bars,
            plan: None,
            authenticated,
            status: "ok".into(),
            fetched_at: 0,
        }
    }

    fn map(entries: Vec<(&str, UsageSnapshot)>) -> HashMap<String, UsageSnapshot> {
        entries.into_iter().map(|(k, v)| (k.into(), v)).collect()
    }

    #[test]
    fn claude_weekly_model_codex_weekly() {
        let snapshots = map(vec![
            ("claude", snap(true, vec![bar("weekly", 39.0), bar("model", 88.0)])),
            ("codex", snap(true, vec![bar("session", 12.0), bar("weekly", 46.0)])),
        ]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳\u{fe0e} 61·12 ⎔ 54"));
    }

    #[test]
    fn claude_always_precedes_codex() {
        let snapshots = map(vec![
            ("codex", snap(true, vec![bar("weekly", 0.0)])),
            ("claude", snap(true, vec![bar("weekly", 0.0)])),
        ]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳\u{fe0e} 100 ⎔ 100"));
    }

    #[test]
    fn single_claude_bar_has_no_separator() {
        let snapshots = map(vec![("claude", snap(true, vec![bar("model", 39.4)]))]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳\u{fe0e} 61"));
    }

    #[test]
    fn session_and_unknown_bars_are_excluded() {
        let snapshots = map(vec![
            ("claude", snap(true, vec![bar("session", 10.0), bar("weekly", 50.0)])),
            ("codex", snap(true, vec![bar("session", 10.0), bar("primary", 20.0)])),
        ]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳\u{fe0e} 50"));
    }

    #[test]
    fn unauthenticated_and_empty_providers_are_omitted() {
        let snapshots = map(vec![
            ("claude", snap(false, vec![bar("weekly", 10.0)])),
            ("codex", snap(true, vec![])),
        ]);
        assert_eq!(tray_title(&snapshots), None);
    }

    #[test]
    fn empty_map_is_none() {
        assert_eq!(tray_title(&HashMap::new()), None);
    }

    #[test]
    fn update_clears_title_with_empty_string() {
        let source = include_str!("tray.rs");
        assert!(source.contains("unwrap_or(\"\")"));
        let no_op_clear = concat!("set_title(", "None)");
        assert!(
            !source.contains(no_op_clear),
            "clearing the title must write an empty string; a None title is a macOS no-op"
        );
    }

    #[test]
    fn out_of_range_percents_clamp() {
        let snapshots = map(vec![(
            "claude",
            snap(true, vec![bar("weekly", 120.0), bar("model", -5.0)]),
        )]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳\u{fe0e} 0·100"));
    }
}
