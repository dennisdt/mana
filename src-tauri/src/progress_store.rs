use crate::progress::{ProgressState, TallyState, TIERS};
use std::io::{Read, Seek, Write};
use std::sync::atomic::{AtomicU64, Ordering};

pub const SCHEMA_VERSION: u32 = 2;
static SNAPSHOT_TEMPORARY_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressPaths {
    pub primary: std::path::PathBuf,
    pub backup: std::path::PathBuf,
    pub pre_migration: std::path::PathBuf,
    pub temporary: std::path::PathBuf,
}

pub struct ProgressStore {
    pub(crate) state: std::sync::Mutex<ProgressState>,
    pub(crate) paths: ProgressPaths,
}

impl ProgressStore {
    pub fn load(app: &tauri::AppHandle) -> std::io::Result<Self> {
        use tauri::Manager as _;
        let primary = app
            .path()
            .app_data_dir()
            .map_err(std::io::Error::other)?
            .join("progress.json");
        let paths = ProgressPaths::from_primary(primary);
        let outcome = load_state(&paths)?;
        Ok(Self {
            state: std::sync::Mutex::new(outcome.state),
            paths,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoverySource {
    Primary,
    Backup,
    PreMigration,
    Temporary,
    New,
}

#[derive(Debug)]
pub struct LoadOutcome {
    pub state: ProgressState,
    pub source: RecoverySource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveCheckpoint {
    TemporarySynced,
    BackupReplaced,
    PrimaryReplaced,
}

impl ProgressPaths {
    pub fn from_primary(primary: std::path::PathBuf) -> Self {
        let dir = parent_directory(&primary).to_path_buf();
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

fn parent_directory(path: &std::path::Path) -> &std::path::Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
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

struct TemporarySnapshot(std::path::PathBuf);

impl Drop for TemporarySnapshot {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn validate_snapshot(path: &std::path::Path) -> std::io::Result<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        return Err(invalid_data(format!(
            "legacy snapshot is not a regular file: {}",
            path.display()
        )));
    }

    let bytes = std::fs::read(path)?;
    let (_, is_legacy) = decode_state(&bytes)?;
    if !is_legacy {
        return Err(invalid_data(format!(
            "legacy snapshot contains versioned progress: {}",
            path.display()
        )));
    }
    Ok(bytes)
}

fn create_snapshot_temporary(
    path: &std::path::Path,
) -> std::io::Result<(TemporarySnapshot, std::fs::File)> {
    let parent = parent_directory(path);
    let file_name = path
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("progress-snapshot"));

    loop {
        let id = SNAPSHOT_TEMPORARY_ID.fetch_add(1, Ordering::Relaxed);
        let mut temporary_name = file_name.to_os_string();
        temporary_name.push(format!(".tmp-{}-{id}", std::process::id()));
        let temporary_path = parent.join(temporary_name);
        match std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary_path)
        {
            Ok(file) => return Ok((TemporarySnapshot(temporary_path), file)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
}

fn sync_parent_directory(path: &std::path::Path) -> std::io::Result<()> {
    std::fs::File::open(parent_directory(path))?.sync_all()
}

fn write_legacy_snapshot(path: &std::path::Path, bytes: &[u8]) -> std::io::Result<()> {
    let (_, is_legacy) = decode_state(bytes)?;
    if !is_legacy {
        return Err(invalid_data(
            "pre-migration snapshot input is not legacy progress",
        ));
    }

    let (temporary, mut file) = create_snapshot_temporary(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);

    let staged_bytes = std::fs::read(&temporary.0)?;
    let (_, staged_is_legacy) = decode_state(&staged_bytes)?;
    if !staged_is_legacy {
        return Err(invalid_data(
            "staged pre-migration snapshot is not legacy progress",
        ));
    }
    if staged_bytes != bytes {
        return Err(invalid_data(
            "staged pre-migration snapshot differs from legacy progress",
        ));
    }

    match std::fs::hard_link(&temporary.0, path) {
        Ok(()) => {
            let remove_result = std::fs::remove_file(&temporary.0);
            let sync_result = sync_parent_directory(path);
            remove_result?;
            sync_result
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            validate_snapshot(path)?;
            Ok(())
        }
        Err(error) => Err(error),
    }
}

enum CandidateReadError {
    RegularFile(std::io::Error),
    Other(std::io::Error),
}

impl CandidateReadError {
    fn into_inner(self) -> std::io::Error {
        match self {
            Self::RegularFile(error) | Self::Other(error) => error,
        }
    }
}

fn read_candidate(
    path: &std::path::Path,
    require_regular_file: bool,
) -> Result<Option<Vec<u8>>, CandidateReadError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(CandidateReadError::Other(error)),
    };

    if require_regular_file && !metadata.file_type().is_file() {
        return Err(CandidateReadError::Other(invalid_data(format!(
            "legacy snapshot is not a regular file: {}",
            path.display()
        ))));
    }

    std::fs::read(path).map(Some).map_err(|error| {
        if metadata.file_type().is_file() {
            CandidateReadError::RegularFile(error)
        } else {
            CandidateReadError::Other(error)
        }
    })
}

fn primary_contains_valid_state(path: &std::path::Path) -> std::io::Result<bool> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.file_type().is_file() => Ok(false),
        Ok(_) => match decode_state(&std::fs::read(path)?) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::InvalidData => Ok(false),
            Err(error) => Err(error),
        },
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn regular_file_contains_valid_state(
    path: &std::path::Path,
    require_legacy: bool,
) -> std::io::Result<bool> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if !metadata.file_type().is_file() {
        return Ok(false);
    }

    match decode_state(&std::fs::read(path)?) {
        Ok((_, is_legacy)) => Ok(!require_legacy || is_legacy),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => Ok(false),
        Err(error) => Err(error),
    }
}

fn has_stable_recovery_source(
    paths: &ProgressPaths,
    primary_is_valid: bool,
) -> std::io::Result<bool> {
    if primary_is_valid || regular_file_contains_valid_state(&paths.backup, false)? {
        return Ok(true);
    }
    regular_file_contains_valid_state(&paths.pre_migration, true)
}

fn open_save_temporary(
    paths: &ProgressPaths,
    primary_is_valid: bool,
) -> std::io::Result<std::fs::File> {
    let open_new = || {
        std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .open(&paths.temporary)
    };

    match open_new() {
        Ok(file) => Ok(file),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            let metadata = std::fs::symlink_metadata(&paths.temporary)?;
            if !metadata.file_type().is_file() {
                return Err(invalid_data(format!(
                    "progress temporary is not a regular file: {}",
                    paths.temporary.display()
                )));
            }
            if !has_stable_recovery_source(paths, primary_is_valid)? {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!(
                        "progress temporary has no separate stable recovery source: {}",
                        paths.temporary.display()
                    ),
                ));
            }

            std::fs::remove_file(&paths.temporary)?;
            open_new()
        }
        Err(error) => Err(error),
    }
}

fn validate_staged_temporary(
    path: &std::path::Path,
    opened_metadata: &std::fs::Metadata,
) -> std::io::Result<()> {
    let path_metadata = std::fs::symlink_metadata(path)?;
    if !opened_metadata.file_type().is_file() || !path_metadata.file_type().is_file() {
        return Err(invalid_data(format!(
            "progress temporary is not a regular file: {}",
            path.display()
        )));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if opened_metadata.dev() != path_metadata.dev()
            || opened_metadata.ino() != path_metadata.ino()
        {
            return Err(invalid_data(format!(
                "progress temporary changed before promotion: {}",
                path.display()
            )));
        }
    }

    Ok(())
}

fn save_state_with_hook(
    paths: &ProgressPaths,
    state: &ProgressState,
    mut hook: impl FnMut(SaveCheckpoint) -> std::io::Result<()>,
) -> std::io::Result<()> {
    let bytes = encode_state(state)?;
    std::fs::create_dir_all(parent_directory(&paths.primary))?;
    let primary_is_valid = primary_contains_valid_state(&paths.primary)?;
    let mut temporary = open_save_temporary(paths, primary_is_valid)?;
    let temporary_metadata = temporary.metadata()?;
    if !temporary_metadata.file_type().is_file() {
        return Err(invalid_data(format!(
            "progress temporary is not a regular file: {}",
            paths.temporary.display()
        )));
    }
    temporary.write_all(&bytes)?;
    temporary.sync_all()?;
    temporary.rewind()?;
    let mut staged_bytes = Vec::new();
    temporary.read_to_end(&mut staged_bytes)?;
    decode_state(&staged_bytes)?;
    validate_staged_temporary(&paths.temporary, &temporary_metadata)?;
    hook(SaveCheckpoint::TemporarySynced)?;

    if primary_is_valid {
        std::fs::rename(&paths.primary, &paths.backup)?;
        hook(SaveCheckpoint::BackupReplaced)?;
    }

    validate_staged_temporary(&paths.temporary, &temporary_metadata)?;
    drop(temporary);
    std::fs::rename(&paths.temporary, &paths.primary)?;
    hook(SaveCheckpoint::PrimaryReplaced)?;
    sync_parent_directory(&paths.primary)
}

pub fn save_state(paths: &ProgressPaths, state: &ProgressState) -> std::io::Result<()> {
    save_state_with_hook(paths, state, |_| Ok(()))
}

fn remove_temporary(path: &std::path::Path) -> std::io::Result<()> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn open_recovery_temporary(path: &std::path::Path) -> std::io::Result<Option<std::fs::File>> {
    let path_metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    };
    if !path_metadata.file_type().is_file() {
        return Err(invalid_data(format!(
            "progress temporary is not a regular file: {}",
            path.display()
        )));
    }

    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let file = options.open(path)?;
    let opened_metadata = file.metadata()?;
    validate_staged_temporary(path, &opened_metadata)?;
    Ok(Some(file))
}

fn recover_temporary(paths: &ProgressPaths) -> std::io::Result<Option<LoadOutcome>> {
    let Some(mut temporary) = open_recovery_temporary(&paths.temporary)? else {
        return Ok(None);
    };
    let temporary_metadata = temporary.metadata()?;
    let mut bytes = Vec::new();
    temporary.read_to_end(&mut bytes)?;
    let (state, is_legacy) = decode_state(&bytes)?;
    if is_legacy {
        return Err(invalid_data(format!(
            "progress temporary contains legacy progress: {}",
            paths.temporary.display()
        )));
    }

    validate_staged_temporary(&paths.temporary, &temporary_metadata)?;
    drop(temporary);
    std::fs::rename(&paths.temporary, &paths.primary)?;
    sync_parent_directory(&paths.primary)?;
    Ok(Some(LoadOutcome {
        state,
        source: RecoverySource::Temporary,
    }))
}

pub fn load_state(paths: &ProgressPaths) -> std::io::Result<LoadOutcome> {
    let candidates = [
        (&paths.primary, RecoverySource::Primary),
        (&paths.backup, RecoverySource::Backup),
        (&paths.pre_migration, RecoverySource::PreMigration),
    ];
    let mut recovery_error = None;

    for (path, source) in candidates {
        let bytes = match read_candidate(path, source == RecoverySource::PreMigration) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => continue,
            Err(CandidateReadError::RegularFile(error)) if source == RecoverySource::Primary => {
                return Err(error);
            }
            Err(error) => {
                if recovery_error.is_none() {
                    recovery_error = Some(error.into_inner());
                }
                continue;
            }
        };

        match decode_state(&bytes) {
            Ok((state, is_legacy)) => {
                if source == RecoverySource::PreMigration && !is_legacy {
                    if recovery_error.is_none() {
                        recovery_error = Some(invalid_data(
                            "pre-migration snapshot contains versioned progress",
                        ));
                    }
                    continue;
                }
                if source != RecoverySource::PreMigration && is_legacy {
                    write_legacy_snapshot(&paths.pre_migration, &bytes)?;
                }
                if source != RecoverySource::Primary || is_legacy {
                    save_state(paths, &state)?;
                }
                remove_temporary(&paths.temporary)?;
                return Ok(LoadOutcome { state, source });
            }
            Err(error) => {
                if recovery_error.is_none() {
                    recovery_error = Some(error);
                }
            }
        }
    }

    match recover_temporary(paths) {
        Ok(Some(outcome)) => return Ok(outcome),
        Ok(None) => {}
        Err(error) => return Err(error),
    }

    if let Some(error) = recovery_error {
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
        decode_state, encode_state, load_state, save_state, save_state_with_hook, ProgressEnvelope,
        ProgressPaths, RecoverySource, SaveCheckpoint, SCHEMA_VERSION,
    };
    use crate::progress::ProgressState;

    static TEST_DIRECTORY_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    static TEST_CURRENT_DIRECTORY: std::sync::Mutex<()> = std::sync::Mutex::new(());

    struct TestDirectory(std::path::PathBuf);

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    struct CurrentDirectory(std::path::PathBuf);

    #[cfg(unix)]
    struct RestoredPermissions {
        path: std::path::PathBuf,
        permissions: std::fs::Permissions,
    }

    #[cfg(unix)]
    impl RestoredPermissions {
        fn new(path: &std::path::Path) -> Self {
            Self {
                path: path.to_path_buf(),
                permissions: std::fs::metadata(path).unwrap().permissions(),
            }
        }
    }

    #[cfg(unix)]
    impl Drop for RestoredPermissions {
        fn drop(&mut self) {
            std::fs::set_permissions(&self.path, self.permissions.clone()).unwrap();
        }
    }

    impl CurrentDirectory {
        fn set(path: &std::path::Path) -> Self {
            let original = std::env::current_dir().unwrap();
            std::env::set_current_dir(path).unwrap();
            Self(original)
        }
    }

    impl Drop for CurrentDirectory {
        fn drop(&mut self) {
            std::env::set_current_dir(&self.0).unwrap();
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

    fn state_with_rank(rank: usize) -> ProgressState {
        let mut state = fixture_state();
        state.rank = rank;
        state
    }

    fn load_exact(path: &std::path::Path) -> std::io::Result<ProgressState> {
        let bytes = std::fs::read(path)?;
        decode_state(&bytes).map(|(state, _)| state)
    }

    #[test]
    fn save_keeps_previous_primary_as_backup() {
        let (_dir, paths) = test_paths("backup-rotation");
        let old = state_with_rank(3);
        let new = state_with_rank(4);
        save_state(&paths, &old).unwrap();
        save_state(&paths, &new).unwrap();
        assert_eq!(load_exact(&paths.primary).unwrap(), new);
        assert_eq!(load_exact(&paths.backup).unwrap(), old);
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_valid_primary_is_not_replaced_from_an_older_backup() {
        use std::os::unix::fs::PermissionsExt;

        let (_dir, paths) = test_paths("unreadable-primary");
        let primary = encode_state(&state_with_rank(4)).unwrap();
        let backup = encode_state(&state_with_rank(3)).unwrap();
        std::fs::write(&paths.primary, &primary).unwrap();
        std::fs::write(&paths.backup, &backup).unwrap();

        let error = {
            let _permissions = RestoredPermissions::new(&paths.primary);
            let mut unreadable = std::fs::metadata(&paths.primary).unwrap().permissions();
            unreadable.set_mode(0);
            std::fs::set_permissions(&paths.primary, unreadable).unwrap();
            load_state(&paths).unwrap_err()
        };

        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), primary);
        assert_eq!(std::fs::read(&paths.backup).unwrap(), backup);
        assert!(!paths.temporary.exists());
    }

    #[cfg(unix)]
    #[test]
    fn save_rejects_temporary_symlink_without_clobbering_its_target() {
        use std::os::unix::fs::symlink;

        let (dir, paths) = test_paths("temporary-symlink-sentinel");
        let primary = encode_state(&state_with_rank(3)).unwrap();
        let sentinel_path = dir.0.join("sentinel");
        let sentinel = b"unrelated sentinel data";
        std::fs::write(&paths.primary, &primary).unwrap();
        std::fs::write(&sentinel_path, sentinel).unwrap();
        symlink(&sentinel_path, &paths.temporary).unwrap();

        let error = save_state(&paths, &state_with_rank(4)).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), primary);
        assert_eq!(std::fs::read(&sentinel_path).unwrap(), sentinel);
        assert_eq!(std::fs::read_link(&paths.temporary).unwrap(), sentinel_path);
        assert!(!paths.backup.exists());
    }

    #[cfg(unix)]
    #[test]
    fn save_rejects_temporary_symlink_without_clobbering_backup() {
        use std::os::unix::fs::symlink;

        let (_dir, paths) = test_paths("temporary-symlink-backup");
        let primary = encode_state(&state_with_rank(3)).unwrap();
        let backup = encode_state(&state_with_rank(2)).unwrap();
        std::fs::write(&paths.primary, &primary).unwrap();
        std::fs::write(&paths.backup, &backup).unwrap();
        symlink(&paths.backup, &paths.temporary).unwrap();

        let error = save_state(&paths, &state_with_rank(4)).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), primary);
        assert_eq!(std::fs::read(&paths.backup).unwrap(), backup);
        assert_eq!(std::fs::read_link(&paths.temporary).unwrap(), paths.backup);
    }

    #[test]
    fn save_retries_after_regular_temporary_when_stable_primary_exists() {
        let (_dir, paths) = test_paths("regular-temporary-retry");
        save_state(&paths, &state_with_rank(2)).unwrap();
        save_state_with_hook(&paths, &state_with_rank(3), |checkpoint| {
            if checkpoint == SaveCheckpoint::TemporarySynced {
                return Err(std::io::Error::other("simulated interruption"));
            }
            Ok(())
        })
        .unwrap_err();

        save_state(&paths, &state_with_rank(4)).unwrap();

        assert_eq!(load_exact(&paths.primary).unwrap().rank, 4);
        assert_eq!(load_exact(&paths.backup).unwrap().rank, 2);
        assert!(!paths.temporary.exists());
    }

    #[test]
    fn save_preserves_regular_temporary_without_another_stable_state() {
        let (_dir, paths) = test_paths("regular-temporary-no-stable-state");
        let temporary = encode_state(&state_with_rank(3)).unwrap();
        std::fs::write(&paths.temporary, &temporary).unwrap();

        let error = save_state(&paths, &state_with_rank(4)).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(std::fs::read(&paths.temporary).unwrap(), temporary);
        assert!(!paths.primary.exists());
        assert!(!paths.backup.exists());
    }

    #[test]
    fn first_save_interrupted_after_temporary_sync_recovers_exact_state() {
        let (_dir, paths) = test_paths("first-save-temporary-recovery");
        let expected = state_with_rank(4);
        let error = save_state_with_hook(&paths, &expected, |checkpoint| {
            if checkpoint == SaveCheckpoint::TemporarySynced {
                return Err(std::io::Error::other("simulated interruption"));
            }
            Ok(())
        })
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Other);
        assert!(!paths.primary.exists());
        assert_eq!(load_exact(&paths.temporary).unwrap(), expected);

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Temporary);
        assert_eq!(loaded.state, expected);
        assert_eq!(load_exact(&paths.primary).unwrap(), expected);
        assert!(!paths.temporary.exists());
    }

    #[test]
    fn malformed_temporary_only_artifact_fails_closed_and_is_preserved() {
        let (_dir, paths) = test_paths("malformed-temporary-only");
        let malformed = b"{\"schema_version\":2";
        std::fs::write(&paths.temporary, malformed).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&paths.temporary).unwrap(), malformed);
        assert!(!paths.primary.exists());
    }

    #[test]
    fn legacy_temporary_only_artifact_fails_closed_and_is_preserved() {
        let (_dir, paths) = test_paths("legacy-temporary-only");
        let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.temporary, legacy).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&paths.temporary).unwrap(), legacy);
        assert!(!paths.primary.exists());
        assert!(!paths.pre_migration.exists());
    }

    #[test]
    fn directory_at_temporary_path_fails_closed_and_is_preserved() {
        let (_dir, paths) = test_paths("directory-temporary-only");
        std::fs::create_dir(&paths.temporary).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(std::fs::symlink_metadata(&paths.temporary)
            .unwrap()
            .file_type()
            .is_dir());
        assert!(!paths.primary.exists());
    }

    #[cfg(unix)]
    #[test]
    fn symlink_temporary_only_artifact_fails_closed_without_reading_target() {
        use std::os::unix::fs::symlink;

        let (dir, paths) = test_paths("symlink-temporary-only");
        let target = dir.0.join("temporary-target.json");
        let target_bytes = encode_state(&state_with_rank(4)).unwrap();
        std::fs::write(&target, &target_bytes).unwrap();
        symlink(&target, &paths.temporary).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&target).unwrap(), target_bytes);
        assert_eq!(std::fs::read_link(&paths.temporary).unwrap(), target);
        assert!(!paths.primary.exists());
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_temporary_only_artifact_fails_closed_and_is_preserved() {
        use std::os::unix::fs::PermissionsExt;

        let (_dir, paths) = test_paths("unreadable-temporary-only");
        let temporary = encode_state(&state_with_rank(4)).unwrap();
        std::fs::write(&paths.temporary, &temporary).unwrap();

        let error = {
            let _permissions = RestoredPermissions::new(&paths.temporary);
            let mut unreadable = std::fs::metadata(&paths.temporary).unwrap().permissions();
            unreadable.set_mode(0);
            std::fs::set_permissions(&paths.temporary, unreadable).unwrap();
            load_state(&paths).unwrap_err()
        };

        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert_eq!(std::fs::read(&paths.temporary).unwrap(), temporary);
        assert!(!paths.primary.exists());
    }

    #[test]
    fn every_interrupted_boundary_leaves_a_recoverable_state() {
        for checkpoint in [
            SaveCheckpoint::TemporarySynced,
            SaveCheckpoint::BackupReplaced,
            SaveCheckpoint::PrimaryReplaced,
        ] {
            let (_dir, paths) = test_paths(&format!("interrupt-{checkpoint:?}"));
            save_state(&paths, &state_with_rank(3)).unwrap();
            let error = save_state_with_hook(&paths, &state_with_rank(4), |reached| {
                if reached == checkpoint {
                    return Err(std::io::Error::other("simulated interruption"));
                }
                Ok(())
            })
            .unwrap_err();
            assert_eq!(error.kind(), std::io::ErrorKind::Other);
            match checkpoint {
                SaveCheckpoint::TemporarySynced => {
                    assert_eq!(load_exact(&paths.primary).unwrap().rank, 3);
                    assert_eq!(load_exact(&paths.temporary).unwrap().rank, 4);
                }
                SaveCheckpoint::BackupReplaced => {
                    assert!(!paths.primary.exists());
                    assert_eq!(load_exact(&paths.backup).unwrap().rank, 3);
                    assert_eq!(load_exact(&paths.temporary).unwrap().rank, 4);
                }
                SaveCheckpoint::PrimaryReplaced => {
                    assert_eq!(load_exact(&paths.primary).unwrap().rank, 4);
                    assert_eq!(load_exact(&paths.backup).unwrap().rank, 3);
                    assert!(!paths.temporary.exists());
                }
            }
            let loaded = load_state(&paths).unwrap();
            assert!([3, 4].contains(&loaded.state.rank));
        }
    }

    #[test]
    fn corrupt_primary_recovers_from_backup() {
        let (dir, paths) = test_paths("backup-recovery");
        std::fs::write(&paths.primary, b"not json").unwrap();
        let backup = encode_state(&fixture_state()).unwrap();
        std::fs::write(&paths.backup, &backup).unwrap();
        let loaded = load_state(&paths).unwrap();
        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state, fixture_state());
        assert_eq!(load_exact(&paths.primary).unwrap(), fixture_state());
        assert_eq!(std::fs::read(&paths.backup).unwrap(), backup);
        drop(dir);
    }

    #[test]
    fn valid_load_removes_an_abandoned_temporary_file() {
        let (_dir, paths) = test_paths("temporary-cleanup");
        let state = state_with_rank(3);
        std::fs::write(&paths.primary, encode_state(&state).unwrap()).unwrap();
        std::fs::write(&paths.temporary, b"abandoned temporary").unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.state, state);
        assert!(!paths.temporary.exists());
    }

    #[test]
    fn failed_load_preserves_an_abandoned_temporary_file() {
        let (_dir, paths) = test_paths("temporary-preserved");
        std::fs::write(&paths.primary, b"invalid primary").unwrap();
        std::fs::write(&paths.temporary, b"abandoned temporary").unwrap();

        load_state(&paths).unwrap_err();

        assert_eq!(
            std::fs::read(&paths.temporary).unwrap(),
            b"abandoned temporary"
        );
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
        let first_legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        let second_legacy = include_bytes!("../tests/fixtures/progress_v1_early.json");

        std::fs::write(&paths.primary, first_legacy).unwrap();
        let first_load = load_state(&paths).unwrap();
        assert_eq!(first_load.source, RecoverySource::Primary);
        assert_eq!(std::fs::read(&paths.pre_migration).unwrap(), first_legacy);

        std::fs::write(&paths.primary, second_legacy).unwrap();
        let second_load = load_state(&paths).unwrap();
        assert_eq!(second_load.source, RecoverySource::Primary);
        assert_eq!(std::fs::read(&paths.pre_migration).unwrap(), first_legacy);
    }

    #[test]
    fn invalid_existing_snapshot_rejects_legacy_primary_without_changing_snapshot() {
        for (label, invalid_snapshot) in [
            ("empty-snapshot", &b""[..]),
            ("partial-snapshot", &br#"{"rank":"#[..]),
        ] {
            let (_dir, paths) = test_paths(label);
            let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
            std::fs::write(&paths.primary, legacy).unwrap();
            std::fs::write(&paths.pre_migration, invalid_snapshot).unwrap();

            let error = load_state(&paths).unwrap_err();

            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
            assert_eq!(
                std::fs::read(&paths.pre_migration).unwrap(),
                invalid_snapshot
            );
        }
    }

    #[test]
    fn versioned_existing_snapshot_rejects_legacy_primary_without_changing_snapshot() {
        let (_dir, paths) = test_paths("versioned-snapshot");
        let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        let versioned_snapshot = encode_state(&state_with_rank(3)).unwrap();
        std::fs::write(&paths.primary, legacy).unwrap();
        std::fs::write(&paths.pre_migration, &versioned_snapshot).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(
            std::fs::read(&paths.pre_migration).unwrap(),
            versioned_snapshot
        );
    }

    #[test]
    fn versioned_pre_migration_snapshot_is_not_a_recovery_source() {
        let (_dir, paths) = test_paths("versioned-snapshot-recovery");
        let versioned_snapshot = encode_state(&state_with_rank(3)).unwrap();
        std::fs::write(&paths.pre_migration, &versioned_snapshot).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(
            std::fs::read(&paths.pre_migration).unwrap(),
            versioned_snapshot
        );
    }

    #[test]
    fn relative_primary_publishes_legacy_snapshot_and_syncs_its_directory() {
        let _current_directory_lock = TEST_CURRENT_DIRECTORY
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (dir, _) = test_paths("relative-primary");
        let _current_directory = CurrentDirectory::set(&dir.0);
        let paths = ProgressPaths::from_primary(std::path::PathBuf::from("progress.json"));
        let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.primary, legacy).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Primary);
        assert_eq!(std::fs::read(&paths.pre_migration).unwrap(), legacy);
    }

    #[test]
    fn malformed_snapshot_input_is_not_published() {
        let (_dir, paths) = test_paths("malformed-snapshot-input");

        let error = super::write_legacy_snapshot(&paths.pre_migration, b"{").unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(!paths.pre_migration.exists());
    }

    #[test]
    fn primary_takes_precedence_over_backup_and_snapshot() {
        let (_dir, paths) = test_paths("primary-precedence");
        std::fs::write(&paths.primary, encode_state(&state_with_rank(3)).unwrap()).unwrap();
        std::fs::write(&paths.backup, encode_state(&state_with_rank(4)).unwrap()).unwrap();
        std::fs::write(
            &paths.pre_migration,
            encode_state(&state_with_rank(5)).unwrap(),
        )
        .unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Primary);
        assert_eq!(loaded.state.rank, 3);
    }

    #[test]
    fn backup_takes_precedence_over_snapshot() {
        let (_dir, paths) = test_paths("backup-precedence");
        std::fs::write(&paths.backup, encode_state(&state_with_rank(4)).unwrap()).unwrap();
        std::fs::write(
            &paths.pre_migration,
            encode_state(&state_with_rank(5)).unwrap(),
        )
        .unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state.rank, 4);
    }

    #[test]
    fn legacy_backup_is_snapshotted_before_primary_repair() {
        let (_dir, paths) = test_paths("legacy-backup");
        let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.primary, b"invalid primary").unwrap();
        std::fs::write(&paths.backup, legacy).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state, fixture_state());
        assert_eq!(std::fs::read(&paths.pre_migration).unwrap(), legacy);
        assert_eq!(load_exact(&paths.primary).unwrap(), fixture_state());
        assert_eq!(std::fs::read(&paths.backup).unwrap(), legacy);
    }

    #[test]
    fn invalid_primary_and_backup_recover_from_pre_migration_snapshot() {
        let (_dir, paths) = test_paths("snapshot-recovery");
        std::fs::write(&paths.primary, b"invalid primary").unwrap();
        std::fs::write(&paths.backup, b"invalid backup").unwrap();
        let snapshot = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.pre_migration, snapshot).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::PreMigration);
        assert_eq!(loaded.state, fixture_state());
        assert_eq!(load_exact(&paths.primary).unwrap(), fixture_state());
        assert_eq!(std::fs::read(&paths.backup).unwrap(), b"invalid backup");
    }

    #[test]
    fn snapshot_recovery_repairs_only_the_invalid_primary() {
        let (_dir, paths) = test_paths("candidate-preservation");
        let invalid_backup = b"{\"schema_version\":2}";
        let snapshot = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.primary, b"invalid primary").unwrap();
        std::fs::write(&paths.backup, invalid_backup).unwrap();
        std::fs::write(&paths.pre_migration, snapshot).unwrap();

        load_state(&paths).unwrap();

        assert_eq!(load_exact(&paths.primary).unwrap(), fixture_state());
        assert_eq!(std::fs::read(&paths.backup).unwrap(), invalid_backup);
        assert_eq!(std::fs::read(&paths.pre_migration).unwrap(), snapshot);
    }

    #[cfg(unix)]
    #[test]
    fn dangling_candidate_symlink_is_not_treated_as_a_new_install() {
        use std::os::unix::fs::symlink;

        let (_dir, paths) = test_paths("dangling-symlink");
        symlink("missing-progress.json", &paths.primary).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        assert!(std::fs::symlink_metadata(&paths.primary)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[cfg(unix)]
    #[test]
    fn dangling_primary_recovers_from_valid_backup() {
        use std::os::unix::fs::symlink;

        let (_dir, paths) = test_paths("dangling-primary-backup");
        symlink("missing-progress.json", &paths.primary).unwrap();
        std::fs::write(&paths.backup, encode_state(&state_with_rank(4)).unwrap()).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state.rank, 4);
        assert!(std::fs::symlink_metadata(&paths.primary)
            .unwrap()
            .file_type()
            .is_file());
        assert_eq!(load_exact(&paths.primary).unwrap().rank, 4);
        assert_eq!(load_exact(&paths.backup).unwrap().rank, 4);
    }

    #[cfg(unix)]
    #[test]
    fn snapshot_symlink_is_rejected_even_when_its_target_is_valid() {
        use std::os::unix::fs::symlink;

        let (dir, paths) = test_paths("snapshot-symlink");
        let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        let target = dir.0.join("snapshot-target.json");
        std::fs::write(&paths.primary, legacy).unwrap();
        std::fs::write(&target, legacy).unwrap();
        symlink(&target, &paths.pre_migration).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&target).unwrap(), legacy);
        assert!(std::fs::symlink_metadata(&paths.pre_migration)
            .unwrap()
            .file_type()
            .is_symlink());
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

    #[test]
    #[ignore = "requires MANA_PROGRESS_V1_FIXTURE"]
    fn external_v1_fixture_preserves_all_progress_invariants() {
        let source = std::path::PathBuf::from(
            std::env::var_os("MANA_PROGRESS_V1_FIXTURE").expect("fixture path"),
        );
        let bytes = std::fs::read(source).unwrap();
        let (before, _) = decode_state(&bytes).unwrap();
        let (_dir, paths) = test_paths("external-migration");
        std::fs::write(&paths.primary, bytes).unwrap();
        let after = load_state(&paths).unwrap().state;
        assert_eq!(after, before);
        assert!(paths.pre_migration.exists());
    }
}
