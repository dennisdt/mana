use std::process::Command;
use tauri::Emitter;

fn is_running(name: &str) -> bool {
    Command::new("/usr/bin/pgrep")
        .args(["-x", name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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
