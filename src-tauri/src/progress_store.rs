use crate::progress::{ProgressState, TallyState, TIERS};
use std::io::Write;

pub const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressPaths {
    pub primary: std::path::PathBuf,
    pub backup: std::path::PathBuf,
    pub pre_migration: std::path::PathBuf,
    pub temporary: std::path::PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverySource {
    Primary,
    Backup,
    PreMigration,
    New,
}

#[derive(Debug)]
pub struct LoadOutcome {
    pub state: ProgressState,
    pub source: RecoverySource,
}

impl ProgressPaths {
    pub fn from_primary(primary: std::path::PathBuf) -> Self {
        let dir = primary
            .parent()
            .unwrap_or_else(|| std::path::Path::new(""))
            .to_path_buf();
        Self {
            primary,
            backup: dir.join("progress.json.bak"),
            pre_migration: dir.join("progress.pre-migration-v1.json"),
            temporary: dir.join("progress.json.tmp"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgressEnvelope {
    pub schema_version: u32,
    pub state: ProgressState,
}

#[derive(serde::Deserialize)]
struct LegacyProgressState {
    rank: usize,
    prestige: u32,
    prestige_token_floor: u64,
    tally: TallyState,
}

fn invalid_data(error: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}

fn validate_rank(state: ProgressState) -> std::io::Result<ProgressState> {
    if state.rank >= TIERS.len() {
        return Err(invalid_data(
            "progress rank is outside the known tier table",
        ));
    }
    Ok(state)
}

pub fn encode_state(state: &ProgressState) -> std::io::Result<Vec<u8>> {
    serde_json::to_vec(&ProgressEnvelope {
        schema_version: SCHEMA_VERSION,
        state: state.clone(),
    })
    .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
}

pub fn decode_state(bytes: &[u8]) -> std::io::Result<(ProgressState, bool)> {
    let value = serde_json::from_slice::<serde_json::Value>(bytes).map_err(invalid_data)?;
    if value.get("schema_version").is_some() {
        let envelope = serde_json::from_value::<ProgressEnvelope>(value).map_err(invalid_data)?;
        if envelope.schema_version != SCHEMA_VERSION {
            return Err(invalid_data("unsupported progress schema version"));
        }
        return Ok((validate_rank(envelope.state)?, false));
    }

    let legacy = serde_json::from_value::<LegacyProgressState>(value).map_err(invalid_data)?;
    let state = ProgressState {
        rank: legacy.rank,
        prestige: legacy.prestige,
        prestige_token_floor: legacy.prestige_token_floor,
        // A successful legacy parse represents existing user progress.
        initialized: true,
        tally: legacy.tally,
    };
    Ok((validate_rank(state)?, true))
}

fn write_legacy_snapshot(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let mut file = match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => return Ok(()),
        Err(error) => return Err(error),
    };
    file.write_all(bytes)
}

pub fn load_state(paths: &ProgressPaths) -> std::io::Result<LoadOutcome> {
    let candidates = [
        (&paths.primary, RecoverySource::Primary),
        (&paths.backup, RecoverySource::Backup),
        (&paths.pre_migration, RecoverySource::PreMigration),
    ];
    let mut invalid_error = None;

    for (path, source) in candidates {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
        };

        match decode_state(&bytes) {
            Ok((state, is_legacy)) => {
                if source == RecoverySource::Primary && is_legacy {
                    write_legacy_snapshot(&paths.pre_migration, &bytes)?;
                }
                return Ok(LoadOutcome { state, source });
            }
            Err(error) => invalid_error = Some(error),
        }
    }

    if let Some(error) = invalid_error {
        return Err(error);
    }

    Ok(LoadOutcome {
        state: ProgressState::default(),
        source: RecoverySource::New,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        decode_state, encode_state, load_state, ProgressEnvelope, ProgressPaths, RecoverySource,
        SCHEMA_VERSION,
    };
    use crate::progress::ProgressState;

    static TEST_DIRECTORY_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    struct TestDirectory(std::path::PathBuf);

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn test_paths(label: &str) -> (TestDirectory, ProgressPaths) {
        use std::sync::atomic::Ordering;
        let id = TEST_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir()
            .join(format!("mana-progress-{label}-{}-{id}", std::process::id(),));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let paths = ProgressPaths::from_primary(dir.join("progress.json"));
        (TestDirectory(dir), paths)
    }

    fn fixture_state() -> ProgressState {
        serde_json::from_slice(include_bytes!("../tests/fixtures/progress_v1.json")).unwrap()
    }

    #[test]
    fn corrupt_primary_recovers_from_backup() {
        let (dir, paths) = test_paths("backup-recovery");
        std::fs::write(&paths.primary, b"not json").unwrap();
        std::fs::write(&paths.backup, encode_state(&fixture_state()).unwrap()).unwrap();
        let loaded = load_state(&paths).unwrap();
        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state, fixture_state());
        drop(dir);
    }

    #[test]
    fn existing_invalid_files_never_become_default() {
        let (_dir, paths) = test_paths("invalid-existing");
        std::fs::write(&paths.primary, b"broken").unwrap();
        let error = load_state(&paths).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn no_files_is_the_only_new_install_path() {
        let (_dir, paths) = test_paths("new-install");
        let loaded = load_state(&paths).unwrap();
        assert_eq!(loaded.source, RecoverySource::New);
        assert!(!loaded.state.initialized);
    }

    #[test]
    fn legacy_snapshot_is_written_once_and_never_overwritten() {
        let (_dir, paths) = test_paths("immutable-legacy");
        let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.primary, legacy).unwrap();
        load_state(&paths).unwrap();
        let original = std::fs::read(&paths.pre_migration).unwrap();
        std::fs::write(&paths.primary, br#"{"rank":0}"#).unwrap();
        let _ = load_state(&paths);
        assert_eq!(std::fs::read(&paths.pre_migration).unwrap(), original);
    }

    #[test]
    fn migrates_current_unversioned_state_without_changing_progress() {
        let bytes = include_bytes!("../tests/fixtures/progress_v1.json");
        let (state, migrated) = decode_state(bytes).unwrap();
        assert!(migrated);
        assert_eq!((state.rank, state.prestige), (8, 7));
        assert_eq!(state.prestige_token_floor, 123456);
        assert!(state.initialized);
        assert_eq!(state.tally.total_tokens, 987654321);
        assert_eq!(state.tally.claude_offsets["/fixture/claude.jsonl"], 44);
        assert_eq!(state.tally.codex_offsets["/fixture/codex.jsonl"], 55);
        assert_eq!(state.tally.codex_totals["/fixture/codex.jsonl"], 66);
    }

    #[test]
    fn legacy_state_without_initialized_is_treated_as_initialized() {
        let bytes = include_bytes!("../tests/fixtures/progress_v1_early.json");
        let (state, migrated) = decode_state(bytes).unwrap();
        assert!(migrated);
        assert!(state.initialized);
        assert_eq!((state.rank, state.prestige), (5, 2));
        assert_eq!(
            state.tally.claude_offsets["/fixture/early-claude.jsonl"],
            11
        );
        assert!(state.tally.codex_offsets.is_empty());
        assert!(state.tally.codex_totals.is_empty());
    }

    #[test]
    fn legacy_state_with_initialized_false_is_treated_as_initialized() {
        let bytes = include_bytes!("../tests/fixtures/progress_v1_initialized_false.json");
        let (state, migrated) = decode_state(bytes).unwrap();
        assert!(migrated);
        assert!(state.initialized);
        assert_eq!((state.rank, state.prestige), (8, 7));
        assert_eq!(state.prestige_token_floor, 123456);
        assert_eq!(state.tally.total_tokens, 987654321);
        assert_eq!(state.tally.claude_offsets["/fixture/claude.jsonl"], 44);
        assert_eq!(state.tally.codex_offsets["/fixture/codex.jsonl"], 55);
        assert_eq!(state.tally.codex_totals["/fixture/codex.jsonl"], 66);
    }

    #[test]
    fn version_two_roundtrips_every_field() {
        let original = fixture_state();
        let bytes = encode_state(&original).unwrap();
        let (decoded, migrated) = decode_state(&bytes).unwrap();
        assert!(!migrated);
        assert_eq!(decoded, original);
    }

    #[test]
    fn rejects_unknown_future_schema() {
        let error = decode_state(br#"{"schema_version":99,"state":{}}"#).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_flattened_document_with_future_schema_version() {
        let error = decode_state(
            br#"{"schema_version":99,"rank":8,"prestige":7,"prestige_token_floor":123456,"initialized":true,"tally":{}}"#,
        )
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_rank_outside_the_known_tier_table() {
        let mut invalid = fixture_state();
        invalid.rank = crate::progress::TIERS.len();
        let bytes = serde_json::to_vec(&ProgressEnvelope {
            schema_version: SCHEMA_VERSION,
            state: invalid,
        })
        .unwrap();
        let error = decode_state(&bytes).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }
}
