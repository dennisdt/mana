use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::Emitter;

const ACTIVITY_GRACE: Duration = Duration::from_millis(2500);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FileFingerprint {
    len: u64,
    modified: SystemTime,
}

fn jsonl_fingerprints(root: &Path) -> HashMap<PathBuf, FileFingerprint> {
    let mut result = HashMap::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(metadata) = entry.metadata() else {
                continue;
            };
            if metadata.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "jsonl")
            {
                result.insert(
                    path,
                    FileFingerprint {
                        len: metadata.len(),
                        modified: metadata.modified().unwrap_or(UNIX_EPOCH),
                    },
                );
            }
        }
    }
    result
}

#[derive(Default)]
struct ProviderActivity {
    previous: Option<HashMap<PathBuf, FileFingerprint>>,
    last_write_at: Option<Instant>,
}

impl ProviderActivity {
    fn update(
        &mut self,
        current: HashMap<PathBuf, FileFingerprint>,
        now: Instant,
    ) -> bool {
        let wrote = self.previous.as_ref().is_some_and(|previous| {
            current
                .iter()
                .any(|(path, fingerprint)| previous.get(path) != Some(fingerprint))
        });
        self.previous = Some(current);
        if wrote {
            self.last_write_at = Some(now);
        }
        self.last_write_at.is_some_and(|last_write| {
            now.saturating_duration_since(last_write) < ACTIVITY_GRACE
        })
    }
}

/// True if the (`comm`, `args`) pair describes an interactive `name` CLI
/// process: comm is `name` or ends in `/name`, and the args do not mark it
/// as a resident server/helper (Codex.app's app-server). Takes the pair
/// directly — joining comm+args into one string and re-splitting on space
/// would misparse a comm path that itself contains a space.
pub fn proc_matches(comm: &str, args: &str, name: &str) -> bool {
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
/// `.app/` / suffix matching in `proc_matches`. Keeping `field` last in each
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
        proc_matches(comm, a, name)
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
    use super::*;

    fn fingerprints(entries: &[(&str, u64, u64)]) -> HashMap<PathBuf, FileFingerprint> {
        entries
            .iter()
            .map(|(path, len, modified_ms)| {
                (
                    PathBuf::from(path),
                    FileFingerprint {
                        len: *len,
                        modified: std::time::UNIX_EPOCH
                            + Duration::from_millis(*modified_ms),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn initial_scan_is_quiet_and_a_later_write_attacks() {
        let start = Instant::now();
        let mut tracker = ProviderActivity::default();
        assert!(!tracker.update(fingerprints(&[("session.jsonl", 10, 1)]), start));
        assert!(tracker.update(
            fingerprints(&[("session.jsonl", 20, 2)]),
            start + Duration::from_secs(1),
        ));
    }

    #[test]
    fn new_file_after_baseline_attacks_but_deleted_files_do_not() {
        let start = Instant::now();
        let mut tracker = ProviderActivity::default();
        assert!(!tracker.update(fingerprints(&[("old.jsonl", 10, 1)]), start));
        assert!(!tracker.update(HashMap::new(), start + Duration::from_secs(1)));
        assert!(tracker.update(
            fingerprints(&[("new.jsonl", 1, 2)]),
            start + Duration::from_secs(2),
        ));
    }

    #[test]
    fn activity_expires_at_the_grace_boundary() {
        let start = Instant::now();
        let mut tracker = ProviderActivity::default();
        tracker.update(fingerprints(&[("session.jsonl", 10, 1)]), start);
        tracker.update(
            fingerprints(&[("session.jsonl", 20, 2)]),
            start + Duration::from_secs(1),
        );
        assert!(tracker.update(
            fingerprints(&[("session.jsonl", 20, 2)]),
            start + Duration::from_millis(3499),
        ));
        assert!(!tracker.update(
            fingerprints(&[("session.jsonl", 20, 2)]),
            start + Duration::from_millis(3500),
        ));
    }

    #[test]
    fn missing_directory_scans_as_quiet() {
        let root = std::env::temp_dir().join(format!(
            "mana-missing-activity-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        assert!(jsonl_fingerprints(&root).is_empty());
    }
}
