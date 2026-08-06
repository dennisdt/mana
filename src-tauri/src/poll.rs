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
    pub authenticated: bool,
    pub status: String,
    pub fetched_at: i64,
}

pub type Snapshots = Mutex<HashMap<String, UsageSnapshot>>;

pub enum FetchResult {
    Unauthenticated,
    Failed,
    Success {
        bars: Vec<Bar>,
        plan: Option<String>,
    },
}

pub fn fold_snapshot(
    prev: Option<&UsageSnapshot>,
    provider: &str,
    result: FetchResult,
    now: i64,
) -> UsageSnapshot {
    match result {
        FetchResult::Success { bars, plan } => UsageSnapshot {
            provider: provider.into(),
            bars,
            plan,
            authenticated: true,
            status: "ok".into(),
            fetched_at: now,
        },
        FetchResult::Failed => match prev {
            Some(p) if !p.bars.is_empty() => UsageSnapshot {
                authenticated: true,
                status: "stale".into(),
                ..p.clone()
            },
            _ => UsageSnapshot {
                provider: provider.into(),
                bars: Vec::new(),
                plan: None,
                authenticated: true,
                status: "absent".into(),
                fetched_at: now,
            },
        },
        FetchResult::Unauthenticated => UsageSnapshot {
            provider: provider.into(),
            bars: Vec::new(),
            plan: None,
            authenticated: false,
            status: "absent".into(),
            fetched_at: now,
        },
    }
}

fn credentials_are_valid(fields: &[&str]) -> bool {
    fields.iter().all(|field| {
        !field.trim().is_empty() && reqwest::header::HeaderValue::from_str(field).is_ok()
    })
}

async fn fetch_claude(client: &reqwest::Client, ua: &str) -> FetchResult {
    let Some(creds) = creds::read_claude_creds() else {
        return FetchResult::Unauthenticated;
    };
    if !credentials_are_valid(&[&creds.token]) {
        return FetchResult::Unauthenticated;
    }
    let response = match client
        .get("https://api.anthropic.com/api/oauth/usage")
        .bearer_auth(creds.token)
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("User-Agent", ua)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return FetchResult::Failed,
    };
    let response = match response.error_for_status() {
        Ok(response) => response,
        Err(_) => return FetchResult::Failed,
    };
    let v: serde_json::Value = match response.json().await {
        Ok(value) => value,
        Err(_) => return FetchResult::Failed,
    };
    let bars = parsers::parse_claude(&v);
    if bars.is_empty() {
        FetchResult::Failed
    } else {
        FetchResult::Success {
            bars,
            plan: creds.plan,
        }
    }
}

async fn fetch_codex(client: &reqwest::Client) -> FetchResult {
    let Some(c) = creds::read_codex_creds(&creds::codex_auth_path()) else {
        return FetchResult::Unauthenticated;
    };
    if !credentials_are_valid(&[&c.access_token, &c.account_id]) {
        return FetchResult::Unauthenticated;
    }
    let response = match client
        .get("https://chatgpt.com/backend-api/wham/usage")
        .bearer_auth(c.access_token)
        .header("chatgpt-account-id", c.account_id)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return FetchResult::Failed,
    };
    let response = match response.error_for_status() {
        Ok(response) => response,
        Err(_) => return FetchResult::Failed,
    };
    let v: serde_json::Value = match response.json().await {
        Ok(value) => value,
        Err(_) => return FetchResult::Failed,
    };
    let (bars, plan) = parsers::parse_codex(&v);
    if bars.is_empty() {
        FetchResult::Failed
    } else {
        FetchResult::Success { bars, plan }
    }
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
            let ua = if provider == "claude" {
                creds::claude_ua()
            } else {
                String::new()
            };
            let mut tick = tokio::time::interval(std::time::Duration::from_secs(60));
            loop {
                tick.tick().await;
                let result = match provider {
                    "claude" => fetch_claude(&client, &ua).await,
                    _ => fetch_codex(&client).await,
                };
                let (next, title) = {
                    let state = app.state::<Snapshots>();
                    let mut map = state.lock().unwrap();
                    let next = fold_snapshot(map.get(provider), provider, result, epoch_now());
                    map.insert(provider.to_string(), next.clone());
                    (next, crate::tray::tray_title(&map))
                };
                eprintln!(
                    "[mana] {} {} bars={}",
                    provider,
                    next.status,
                    next.bars.len()
                );
                let _ = app.emit("usage-update", &next);
                if let Some(tray) = app.tray_by_id("main") {
                    let _ = tray.set_title(title.as_deref());
                }
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
        Bar {
            id: "session".into(),
            label: "5h".into(),
            used_percent: 26.0,
            resets_at: Some(1783712399),
        }
    }

    #[test]
    fn blank_claude_token_is_not_authenticated() {
        assert!(!credentials_are_valid(&[" "]));
    }

    #[test]
    fn blank_codex_token_or_account_id_is_not_authenticated() {
        assert!(!credentials_are_valid(&["token", ""]));
        assert!(!credentials_are_valid(&["\t", "account"]));
        assert!(!credentials_are_valid(&["token", "\n"]));
        assert!(credentials_are_valid(&["token", "account"]));
    }

    #[test]
    fn malformed_token_header_value_is_not_authenticated() {
        assert!(!credentials_are_valid(&["token\r\ninjected: value"]));
        assert!(!credentials_are_valid(&["token\0suffix"]));
        assert!(credentials_are_valid(&["sk-ant-oat01-abc_123.def-456"]));
    }

    #[test]
    fn malformed_codex_account_header_value_is_not_authenticated() {
        assert!(!credentials_are_valid(&[
            "token",
            "account\r\ninjected: value"
        ]));
        assert!(!credentials_are_valid(&["token", "account\0suffix"]));
        assert!(credentials_are_valid(&[
            "eyJhbGciOiJSUzI1NiJ9.payload.signature",
            "acc_123"
        ]));
    }

    #[test]
    fn success_yields_ok() {
        let s = fold_snapshot(
            None,
            "claude",
            FetchResult::Success {
                bars: vec![bar()],
                plan: None,
            },
            100,
        );
        assert_eq!(s.status, "ok");
        assert!(s.authenticated);
        assert_eq!(s.provider, "claude");
        assert_eq!(s.fetched_at, 100);
        assert_eq!(s.bars.len(), 1);
    }

    #[test]
    fn missing_credentials_are_unauthenticated() {
        let s = fold_snapshot(None, "claude", FetchResult::Unauthenticated, 100);
        assert!(!s.authenticated);
        assert_eq!(s.status, "absent");
        assert!(s.bars.is_empty());
    }

    #[test]
    fn authenticated_failure_without_history_remains_visible() {
        let s = fold_snapshot(None, "codex", FetchResult::Failed, 100);
        assert!(s.authenticated);
        assert_eq!(s.status, "absent");
    }

    #[test]
    fn repeated_authenticated_failure_without_usage_history_remains_absent() {
        let first = fold_snapshot(None, "codex", FetchResult::Failed, 100);
        let second = fold_snapshot(Some(&first), "codex", FetchResult::Failed, 160);
        assert!(second.authenticated);
        assert_eq!(second.status, "absent");
        assert!(second.bars.is_empty());
        assert_eq!(second.plan, None);
        assert_eq!(second.fetched_at, 160);
    }

    #[test]
    fn unauthenticated_snapshot_then_failure_is_authenticated_absent() {
        let unauthenticated = fold_snapshot(None, "codex", FetchResult::Unauthenticated, 100);
        let failed = fold_snapshot(Some(&unauthenticated), "codex", FetchResult::Failed, 160);
        assert!(failed.authenticated);
        assert_eq!(failed.status, "absent");
        assert!(failed.bars.is_empty());
        assert_eq!(failed.plan, None);
        assert_eq!(failed.fetched_at, 160);
    }

    #[test]
    fn authenticated_failure_with_history_goes_stale_keeping_bars_and_time() {
        let prev = fold_snapshot(
            None,
            "codex",
            FetchResult::Success {
                bars: vec![bar()],
                plan: Some("prolite".into()),
            },
            100,
        );
        let s = fold_snapshot(Some(&prev), "codex", FetchResult::Failed, 160);
        assert_eq!(s.status, "stale");
        assert!(s.authenticated);
        assert_eq!(s.bars, prev.bars);
        assert_eq!(s.plan.as_deref(), Some("prolite"));
        assert_eq!(s.fetched_at, 100); // age of the DATA, not of the attempt
    }

    #[test]
    fn credential_removal_discards_history() {
        let prev = fold_snapshot(
            None,
            "claude",
            FetchResult::Success {
                bars: vec![bar()],
                plan: Some("max".into()),
            },
            100,
        );
        let s = fold_snapshot(Some(&prev), "claude", FetchResult::Unauthenticated, 160);
        assert_eq!(s.status, "absent");
        assert!(!s.authenticated);
        assert!(s.bars.is_empty());
        assert_eq!(s.plan, None);
        assert_eq!(s.fetched_at, 160);
    }

    #[test]
    fn later_success_restores_authenticated_fresh_snapshot() {
        let absent = fold_snapshot(None, "claude", FetchResult::Unauthenticated, 100);
        let s = fold_snapshot(
            Some(&absent),
            "claude",
            FetchResult::Success {
                bars: vec![bar()],
                plan: None,
            },
            220,
        );
        assert_eq!(s.status, "ok");
        assert!(s.authenticated);
        assert_eq!(s.fetched_at, 220);
    }

    #[test]
    fn poll_loop_refreshes_tray_title() {
        let source = include_str!("poll.rs");
        let title_call = concat!("tray::", "tray_title");
        assert!(source.contains(title_call));
        assert!(source.contains("tray_by_id(\"main\")"));
    }
}
