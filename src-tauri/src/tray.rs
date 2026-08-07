use std::collections::HashMap;

use crate::poll::UsageSnapshot;

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
        ("claude", "✳", &["weekly", "model"]),
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
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳ 61·12 ⎔ 54"));
    }

    #[test]
    fn claude_always_precedes_codex() {
        let snapshots = map(vec![
            ("codex", snap(true, vec![bar("weekly", 0.0)])),
            ("claude", snap(true, vec![bar("weekly", 0.0)])),
        ]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳ 100 ⎔ 100"));
    }

    #[test]
    fn single_claude_bar_has_no_separator() {
        let snapshots = map(vec![("claude", snap(true, vec![bar("model", 39.4)]))]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳ 61"));
    }

    #[test]
    fn session_and_unknown_bars_are_excluded() {
        let snapshots = map(vec![
            ("claude", snap(true, vec![bar("session", 10.0), bar("weekly", 50.0)])),
            ("codex", snap(true, vec![bar("session", 10.0), bar("primary", 20.0)])),
        ]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳ 50"));
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
    fn out_of_range_percents_clamp() {
        let snapshots = map(vec![(
            "claude",
            snap(true, vec![bar("weekly", 120.0), bar("model", -5.0)]),
        )]);
        assert_eq!(tray_title(&snapshots).as_deref(), Some("✳ 0·100"));
    }
}
