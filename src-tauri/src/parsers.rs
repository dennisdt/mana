use serde::Serialize;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct Bar {
    pub id: String,
    pub label: String,
    pub used_percent: f64,
    pub resets_at: Option<i64>,
}

fn iso_to_epoch(s: &str) -> Option<i64> {
    time::OffsetDateTime::parse(s, &time::format_description::well_known::Rfc3339)
        .ok()
        .map(|t| t.unix_timestamp())
}

pub fn parse_claude(v: &serde_json::Value) -> Vec<Bar> {
    let mut bars = Vec::new();
    if let Some(limits) = v.get("limits").and_then(|l| l.as_array()) {
        for l in limits {
            let Some(percent) = l.get("percent").and_then(|p| p.as_f64()) else {
                continue;
            };
            let resets_at = l
                .get("resets_at")
                .and_then(|r| r.as_str())
                .and_then(iso_to_epoch);
            let (id, label) = match l.get("kind").and_then(|k| k.as_str()) {
                Some("session") => ("session", "5 hour".to_string()),
                Some("weekly_all") => ("weekly", "Weekly".to_string()),
                Some("weekly_scoped") => (
                    "model",
                    l.pointer("/scope/model/display_name")
                        .and_then(|n| n.as_str())
                        .unwrap_or("Model")
                        .to_string(),
                ),
                _ => continue,
            };
            bars.push(Bar {
                id: id.into(),
                label,
                used_percent: percent,
                resets_at,
            });
        }
    }
    if !bars.is_empty() {
        return bars;
    }
    for (key, id, label) in [
        ("five_hour", "session", "5 hour"),
        ("seven_day", "weekly", "Weekly"),
    ] {
        if let Some(pct) = v
            .pointer(&format!("/{key}/utilization"))
            .and_then(|u| u.as_f64())
        {
            bars.push(Bar {
                id: id.into(),
                label: label.into(),
                used_percent: pct,
                resets_at: v
                    .pointer(&format!("/{key}/resets_at"))
                    .and_then(|r| r.as_str())
                    .and_then(iso_to_epoch),
            });
        }
    }
    bars
}

fn codex_window_identity(key: &str, window: &serde_json::Value) -> (&'static str, &'static str) {
    match window.get("limit_window_seconds").and_then(|v| v.as_i64()) {
        Some(18_000) => ("session", "5 hour"),
        Some(604_800) => ("weekly", "Weekly"),
        _ if key == "primary_window" => ("primary", "Primary"),
        _ => ("secondary", "Secondary"),
    }
}

pub fn parse_codex(v: &serde_json::Value) -> (Vec<Bar>, Option<String>) {
    let plan = v
        .get("plan_type")
        .and_then(|p| p.as_str())
        .map(String::from);
    let mut bars = Vec::new();
    for key in ["primary_window", "secondary_window"] {
        if let Some(w) = v.pointer(&format!("/rate_limit/{key}")) {
            if let Some(pct) = w.get("used_percent").and_then(|p| p.as_f64()) {
                let (id, label) = codex_window_identity(key, w);
                bars.push(Bar {
                    id: id.into(),
                    label: label.into(),
                    used_percent: pct,
                    resets_at: w.get("reset_at").and_then(|r| r.as_i64()),
                });
            }
        }
    }
    (bars, plan)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn load(s: &str) -> serde_json::Value {
        serde_json::from_str(s).unwrap()
    }

    #[test]
    fn claude_limits_array() {
        let bars = parse_claude(&load(include_str!("../tests/fixtures/claude_limits.json")));
        assert_eq!(bars.len(), 3);
        assert_eq!(
            bars[0],
            Bar {
                id: "session".into(),
                label: "5 hour".into(),
                used_percent: 26.0,
                resets_at: Some(1783712399),
            }
        );
        assert_eq!(bars[1].id, "weekly");
        assert_eq!(bars[1].used_percent, 19.0);
        assert_eq!(
            bars[2],
            Bar {
                id: "model".into(),
                label: "Fable".into(),
                used_percent: 32.0,
                resets_at: Some(1784062799),
            }
        );
    }

    #[test]
    fn claude_legacy_fallback() {
        let bars = parse_claude(&load(include_str!("../tests/fixtures/claude_legacy.json")));
        assert_eq!(bars.len(), 2);
        assert_eq!(bars[0].id, "session");
        assert_eq!(bars[0].used_percent, 26.0);
        assert_eq!(bars[0].resets_at, Some(1783712399));
        assert_eq!(bars[1].id, "weekly");
    }

    #[test]
    fn claude_garbage_yields_empty() {
        assert!(parse_claude(&load(r#"{"unexpected": true}"#)).is_empty());
        assert!(parse_claude(&load(r#"{"limits": [{"kind": "session"}]}"#)).is_empty());
    }

    #[test]
    fn codex_windows() {
        let (bars, plan) = parse_codex(&load(include_str!("../tests/fixtures/codex_wham.json")));
        assert_eq!(plan.as_deref(), Some("prolite"));
        assert_eq!(
            bars,
            vec![
                Bar {
                    id: "session".into(),
                    label: "5 hour".into(),
                    used_percent: 4.0,
                    resets_at: Some(1783727913)
                },
                Bar {
                    id: "weekly".into(),
                    label: "Weekly".into(),
                    used_percent: 1.0,
                    resets_at: Some(1784314713)
                },
            ]
        );
    }

    #[test]
    fn codex_pro_weekly_primary() {
        let (bars, plan) = parse_codex(&load(include_str!(
            "../tests/fixtures/codex_pro_weekly.json"
        )));
        assert_eq!(plan.as_deref(), Some("pro"));
        assert_eq!(
            bars,
            vec![Bar {
                id: "weekly".into(),
                label: "Weekly".into(),
                used_percent: 55.0,
                resets_at: Some(1784487600),
            }]
        );
    }

    #[test]
    fn codex_unknown_duration_uses_neutral_identity() {
        let (bars, _) = parse_codex(&load(
            r#"{
            "rate_limit": {
                "primary_window": {
                    "used_percent": 12,
                    "limit_window_seconds": 86400,
                    "reset_at": 1784487600
                }
            }
        }"#,
        ));
        assert_eq!(bars[0].id, "primary");
        assert_eq!(bars[0].label, "Primary");
    }

    #[test]
    fn codex_garbage_yields_empty() {
        let (bars, plan) = parse_codex(&load(r#"{"detail": "unauthorized"}"#));
        assert!(bars.is_empty());
        assert!(plan.is_none());
    }
}
