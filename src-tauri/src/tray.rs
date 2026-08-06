use std::collections::HashMap;

use crate::poll::UsageSnapshot;

fn remaining(used_percent: f64) -> i64 {
    (100.0 - used_percent).round().clamp(0.0, 100.0) as i64
}

fn provider_segment(letter: &str, snapshot: &UsageSnapshot) -> Option<String> {
    if !snapshot.authenticated {
        return None;
    }
    let find = |id: &str| snapshot.bars.iter().find(|b| b.id == id);
    let numbers: Vec<String> = [find("session"), find("weekly")]
        .into_iter()
        .flatten()
        .map(|b| remaining(b.used_percent).to_string())
        .collect();
    if numbers.is_empty() {
        return None;
    }
    Some(format!("{letter} {}", numbers.join("·")))
}

pub fn tray_title(snapshots: &HashMap<String, UsageSnapshot>) -> Option<String> {
    let segments: Vec<String> = [("claude", "C"), ("codex", "X")]
        .into_iter()
        .filter_map(|(key, letter)| {
            snapshots.get(key).and_then(|s| provider_segment(letter, s))
        })
        .collect();
    if segments.is_empty() {
        None
    } else {
        Some(segments.join(" "))
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
    fn both_providers_session_and_weekly() {
        let snapshots = map(vec![
            ("claude", snap(true, vec![bar("session", 39.0), bar("weekly", 88.0)])),
            ("codex", snap(true, vec![bar("session", 12.0), bar("weekly", 46.0)])),
        ]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("C 61·12 X 88·54"));
    }

    #[test]
    fn claude_always_precedes_codex() {
        let snapshots = map(vec![
            ("codex", snap(true, vec![bar("session", 0.0)])),
            ("claude", snap(true, vec![bar("session", 0.0)])),
        ]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("C 100 X 100"));
    }

    #[test]
    fn single_bar_has_no_separator() {
        let snapshots = map(vec![("claude", snap(true, vec![bar("session", 39.4)]))]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("C 61"));
    }

    #[test]
    fn model_and_unknown_bars_are_excluded() {
        let snapshots = map(vec![(
            "claude",
            snap(true, vec![bar("model", 10.0), bar("weekly", 50.0)]),
        )]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("C 50"));
    }

    #[test]
    fn unauthenticated_and_empty_providers_are_omitted() {
        let snapshots = map(vec![
            ("claude", snap(false, vec![bar("session", 10.0)])),
            ("codex", snap(true, vec![])),
        ]);
        assert_eq!(tray_title(&snapshots), None);
    }

    #[test]
    fn empty_map_is_none() {
        assert_eq!(tray_title(&HashMap::new()), None);
    }

    #[test]
    fn out_of_range_percents_clamp() {
        let snapshots = map(vec![(
            "codex",
            snap(true, vec![bar("session", 120.0), bar("weekly", -5.0)]),
        )]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("X 0·100"));
    }
}
