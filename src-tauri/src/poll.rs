use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{Emitter, Manager};

use crate::creds;
use crate::parsers::{self, Bar};

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct UsageSnapshot {
    pub provider: String,
    pub bars: Vec<Bar>,
    pub plan: Option<String>,
    pub status: String,
    pub fetched_at: i64,
}

pub type Snapshots = Mutex<HashMap<String, UsageSnapshot>>;

/// A tick's fetch result: Some((bars, plan)) on success, None on any failure
/// (missing creds, HTTP error, unparseable body).
pub type FetchResult = Option<(Vec<Bar>, Option<String>)>;

pub fn fold_snapshot(
    prev: Option<&UsageSnapshot>,
    provider: &str,
    result: FetchResult,
    now: i64,
) -> UsageSnapshot {
    match (result, prev) {
        (Some((bars, plan)), _) => UsageSnapshot {
            provider: provider.into(),
            bars,
            plan,
            status: "ok".into(),
            fetched_at: now,
        },
        (None, Some(p)) => UsageSnapshot { status: "stale".into(), ..p.clone() },
        (None, None) => UsageSnapshot {
            provider: provider.into(),
            bars: Vec::new(),
            plan: None,
            status: "absent".into(),
            fetched_at: now,
        },
    }
}

async fn fetch_claude(client: &reqwest::Client, ua: &str) -> FetchResult {
    let token = creds::read_claude_token()?;
    let v: serde_json::Value = client
        .get("https://api.anthropic.com/api/oauth/usage")
        .bearer_auth(token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", ua)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    let bars = parsers::parse_claude(&v);
    (!bars.is_empty()).then_some((bars, None))
}

async fn fetch_codex(client: &reqwest::Client) -> FetchResult {
    let c = creds::read_codex_creds(&creds::codex_auth_path())?;
    let v: serde_json::Value = client
        .get("https://chatgpt.com/backend-api/wham/usage")
        .bearer_auth(c.access_token)
        .header("chatgpt-account-id", c.account_id)
        .send()
        .await
        .ok()?
        .error_for_status()
        .ok()?
        .json()
        .await
        .ok()?;
    let (bars, plan) = parsers::parse_codex(&v);
    (!bars.is_empty()).then_some((bars, plan))
}

fn epoch_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

pub fn spawn_pollers(app: tauri::AppHandle) {
    for provider in ["claude", "codex"] {
        let app = app.clone();
        tauri::async_runtime::spawn(async move {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("reqwest client");
            let ua = if provider == "claude" { creds::claude_ua() } else { String::new() };
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                tick.tick().await;
                let result = match provider {
                    "claude" => fetch_claude(&client, &ua).await,
                    _ => fetch_codex(&client).await,
                };
                let next = {
                    let state = app.state::<Snapshots>();
                    let mut map = state.lock().unwrap();
                    let next = fold_snapshot(map.get(provider), provider, result, epoch_now());
                    map.insert(provider.to_string(), next.clone());
                    next
                };
                eprintln!("[mana] {} {} bars={}", provider, next.status, next.bars.len());
                let _ = app.emit("usage-update", &next);
            }
        });
    }
}

#[tauri::command]
pub fn get_snapshots(state: tauri::State<'_, Snapshots>) -> Vec<UsageSnapshot> {
    state.lock().unwrap().values().cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar() -> Bar {
        Bar { id: "session".into(), label: "5h".into(), used_percent: 26.0, resets_at: Some(1783712399) }
    }

    #[test]
    fn success_yields_ok() {
        let s = fold_snapshot(None, "claude", Some((vec![bar()], None)), 100);
        assert_eq!(s.status, "ok");
        assert_eq!(s.provider, "claude");
        assert_eq!(s.fetched_at, 100);
        assert_eq!(s.bars.len(), 1);
    }

    #[test]
    fn failure_with_history_goes_stale_keeping_bars_and_time() {
        let prev = fold_snapshot(None, "codex", Some((vec![bar()], Some("prolite".into()))), 100);
        let s = fold_snapshot(Some(&prev), "codex", None, 160);
        assert_eq!(s.status, "stale");
        assert_eq!(s.bars, prev.bars);
        assert_eq!(s.plan.as_deref(), Some("prolite"));
        assert_eq!(s.fetched_at, 100); // age of the DATA, not of the attempt
    }

    #[test]
    fn failure_without_history_is_absent() {
        let s = fold_snapshot(None, "claude", None, 100);
        assert_eq!(s.status, "absent");
        assert!(s.bars.is_empty());
    }

    #[test]
    fn recovery_after_stale_is_ok_again() {
        let prev = fold_snapshot(None, "claude", Some((vec![bar()], None)), 100);
        let stale = fold_snapshot(Some(&prev), "claude", None, 160);
        let s = fold_snapshot(Some(&stale), "claude", Some((vec![bar()], None)), 220);
        assert_eq!(s.status, "ok");
        assert_eq!(s.fetched_at, 220);
    }
}
