use std::path::{Path, PathBuf};
use std::process::Command;

pub struct CodexCreds {
    pub access_token: String,
    pub account_id: String,
}

pub struct ClaudeCreds {
    pub token: String,
    pub plan: Option<String>,
}

pub fn parse_claude_creds(v: &serde_json::Value) -> Option<ClaudeCreds> {
    Some(ClaudeCreds {
        token: v.pointer("/claudeAiOauth/accessToken")?.as_str()?.to_string(),
        plan: v
            .pointer("/claudeAiOauth/subscriptionType")
            .and_then(|p| p.as_str())
            .map(String::from),
    })
}

/// $CODEX_HOME/auth.json, default ~/.codex/auth.json
pub fn codex_auth_path() -> PathBuf {
    std::env::var("CODEX_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(std::env::var("HOME").unwrap_or_default()).join(".codex")
        })
        .join("auth.json")
}

pub fn read_codex_creds(path: &Path) -> Option<CodexCreds> {
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    Some(CodexCreds {
        access_token: v.pointer("/tokens/access_token")?.as_str()?.to_string(),
        account_id: v.pointer("/tokens/account_id")?.as_str()?.to_string(),
    })
}

pub fn parse_version(s: &str) -> Option<String> {
    s.split_whitespace()
        .find(|t| {
            t.chars().next().is_some_and(|c| c.is_ascii_digit())
                && t.matches('.').count() >= 2
                && t.chars().all(|c| c.is_ascii_digit() || c == '.')
        })
        .map(str::to_string)
}

/// Read-only Keychain lookup of Claude Code's OAuth credential record.
pub fn read_claude_creds() -> Option<ClaudeCreds> {
    let out = Command::new("/usr/bin/security")
        .args(["find-generic-password", "-s", "Claude Code-credentials", "-w"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    parse_claude_creds(&serde_json::from_slice(&out.stdout).ok()?)
}

/// UA the Claude usage endpoint requires; real CLI version when available.
pub fn claude_ua() -> String {
    let version = Command::new("claude")
        .arg("--version")
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| parse_version(&String::from_utf8_lossy(&o.stdout)));
    format!("claude-code/{}", version.unwrap_or_else(|| "2.1.0".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codex_creds_from_auth_json() {
        let dir = std::env::temp_dir().join("mana-test-codex");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("auth.json");
        std::fs::write(
            &p,
            r#"{"openai_api_key": null, "tokens": {"id_token": "x.y.z", "access_token": "AT-123", "refresh_token": "RT-DO-NOT-TOUCH", "account_id": "acc-9"}, "last_refresh": "2026-07-02T21:46:07Z"}"#,
        )
        .unwrap();
        let c = read_codex_creds(&p).unwrap();
        assert_eq!(c.access_token, "AT-123");
        assert_eq!(c.account_id, "acc-9");
    }

    #[test]
    fn codex_creds_missing_file_or_fields() {
        assert!(read_codex_creds(Path::new("/nonexistent/auth.json")).is_none());
        let dir = std::env::temp_dir().join("mana-test-codex2");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("auth.json");
        std::fs::write(&p, r#"{"tokens": {}}"#).unwrap();
        assert!(read_codex_creds(&p).is_none());
    }

    #[test]
    fn version_from_cli_output() {
        assert_eq!(parse_version("2.1.34 (Claude Code)"), Some("2.1.34".into()));
        assert_eq!(parse_version("claude v2.2.0"), None);
        assert_eq!(parse_version("garbage"), None);
    }

    #[test]
    fn claude_creds_with_plan() {
        let v: serde_json::Value = serde_json::from_str(
            r#"{"claudeAiOauth": {"accessToken": "AT-1", "refreshToken": "RT-DO-NOT-TOUCH", "subscriptionType": "max"}}"#,
        )
        .unwrap();
        let c = parse_claude_creds(&v).unwrap();
        assert_eq!(c.token, "AT-1");
        assert_eq!(c.plan.as_deref(), Some("max"));
    }

    #[test]
    fn claude_creds_without_plan_or_token() {
        let v: serde_json::Value =
            serde_json::from_str(r#"{"claudeAiOauth": {"accessToken": "AT-1"}}"#).unwrap();
        let c = parse_claude_creds(&v).unwrap();
        assert_eq!(c.plan, None);
        let empty: serde_json::Value = serde_json::from_str(r#"{}"#).unwrap();
        assert!(parse_claude_creds(&empty).is_none());
    }
}
