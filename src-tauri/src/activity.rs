use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Activity {
    claude: bool,
    codex: bool,
}

#[derive(Default)]
pub struct ActivityStore(pub Mutex<Activity>);

pub fn spawn_activity_watcher(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        use tauri::Manager as _;

        let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
            return;
        };
        let claude_root = home.join(".claude/projects");
        let codex_root = home.join(".codex/sessions");
        let mut claude = ProviderActivity::default();
        let mut codex = ProviderActivity::default();
        let mut tick = tokio::time::interval(Duration::from_secs(1));

        loop {
            tick.tick().await;
            let now = Instant::now();
            let next = Activity {
                claude: claude.update(jsonl_fingerprints(&claude_root), now),
                codex: codex.update(jsonl_fingerprints(&codex_root), now),
            };
            let changed = {
                let store = app.state::<ActivityStore>();
                let current = &mut *store.0.lock().unwrap();
                if *current == next {
                    false
                } else {
                    *current = next;
                    true
                }
            };
            if changed {
                let _ = app.emit("activity", next);
            }
        }
    });
}

#[tauri::command]
pub fn get_activity(store: tauri::State<'_, ActivityStore>) -> Activity {
    *store.0.lock().unwrap()
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

    #[test]
    fn providers_track_independently() {
        let start = Instant::now();
        let mut claude = ProviderActivity::default();
        let mut codex = ProviderActivity::default();
        claude.update(fingerprints(&[("claude.jsonl", 10, 1)]), start);
        codex.update(fingerprints(&[("codex.jsonl", 10, 1)]), start);

        assert!(claude.update(
            fingerprints(&[("claude.jsonl", 20, 2)]),
            start + Duration::from_secs(1),
        ));
        assert!(!codex.update(
            fingerprints(&[("codex.jsonl", 10, 1)]),
            start + Duration::from_secs(1),
        ));
    }

    #[test]
    fn activity_store_starts_quiet() {
        assert_eq!(
            *ActivityStore::default().0.lock().unwrap(),
            Activity {
                claude: false,
                codex: false,
            }
        );
    }
}
