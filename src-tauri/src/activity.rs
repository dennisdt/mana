use std::collections::HashMap;
use std::process::Command;
use tauri::Emitter;

/// True if `line` (a `comm args` string, comm then full args separated by a
/// space) is an interactive `name` CLI process: comm is `name` or ends in
/// `/name`, and the args do not mark it as a resident server/helper
/// (Codex.app's app-server).
pub fn line_matches(line: &str, name: &str) -> bool {
    let line = line.trim_start();
    let mut parts = line.splitn(2, ' ');
    let comm = parts.next().unwrap_or("");
    let args = parts.next().unwrap_or("");
    (comm == name || comm.ends_with(&format!("/{name}")))
        && !args.contains("app-server")
        && !comm.contains(".app/")
}

/// Runs `ps -axo pid=,<field>=` and returns a pid -> field map.
///
/// `field` is deliberately fetched in its own ps invocation with pid first:
/// BSD ps only gives the *last* requested `-o` column its natural width and
/// truncates every earlier column to a small fixed width (comm truncates to
/// 16 chars) even when the `=` (no-header) modifier is used and even when
/// output isn't a tty. A single `-axo comm=,args=` call therefore mangles
/// long executable paths (e.g. Codex.app's or Homebrew's), which breaks the
/// `.app/` / suffix matching in `line_matches`. Keeping `field` last in each
/// call avoids the truncation.
fn ps_field(field: &str) -> HashMap<String, String> {
    let out = match Command::new("/bin/ps")
        .args(["-axo", &format!("pid=,{field}=")])
        .output()
    {
        Ok(o) if o.status.success() => o,
        _ => return HashMap::new(),
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|l| {
            let l = l.trim_start();
            let sp = l.find(' ')?;
            Some((l[..sp].to_string(), l[sp + 1..].to_string()))
        })
        .collect()
}

pub fn is_running(name: &str) -> bool {
    let comms = ps_field("comm");
    let args = ps_field("args");
    let empty = String::new();
    comms.iter().any(|(pid, comm)| {
        let a = args.get(pid).unwrap_or(&empty);
        line_matches(&format!("{comm} {a}"), name)
    })
}

pub fn spawn_activity_watcher(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut last: Option<(bool, bool)> = None;
        let mut tick = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            tick.tick().await;
            let now = (is_running("claude"), is_running("codex"));
            if last != Some(now) {
                last = Some(now);
                let _ = app.emit(
                    "activity",
                    &serde_json::json!({ "claude": now.0, "codex": now.1 }),
                );
            }
        }
    });
}

#[tauri::command]
pub fn get_activity() -> serde_json::Value {
    serde_json::json!({ "claude": is_running("claude"), "codex": is_running("codex") })
}

#[cfg(test)]
mod tests {
    use super::line_matches;

    #[test]
    fn matches_cli_invocations() {
        assert!(line_matches("/opt/homebrew/Caskroom/codex/0.144.0/codex-aarch64-apple-darwin/codex codex", "codex"));
        assert!(line_matches("codex codex exec \"fix bug\"", "codex"));
        assert!(line_matches("/usr/local/bin/claude claude --resume", "claude"));
    }

    #[test]
    fn excludes_servers_and_gui_bundles() {
        assert!(!line_matches("/Applications/Codex.app/Contents/Resources/codex codex -c features.code_mode_ho", "codex"));
        assert!(!line_matches("/opt/homebrew/bin/codex codex -s read-only app-server", "codex"));
        assert!(!line_matches("/Applications/Claude.app/Contents/MacOS/Claude Claude", "claude"));
        assert!(!line_matches("random line", "codex"));
    }
}
