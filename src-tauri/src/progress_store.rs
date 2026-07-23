use crate::progress::{ProgressState, TallyState, TIERS};
use std::io::{Read, Seek, Write};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

pub const SCHEMA_VERSION: u32 = 3;
pub const PRIMARY_PROGRESS_FILENAME: &str = "progress.json";
pub const BACKUP_PROGRESS_FILENAME: &str = "progress.json.bak";
static SNAPSHOT_TEMPORARY_ID: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, PartialEq)]
pub struct ProgressPaths {
    pub primary: std::path::PathBuf,
    pub backup: std::path::PathBuf,
    pub pre_migration_v1: std::path::PathBuf,
    pub pre_migration_v2: std::path::PathBuf,
    pub temporary: std::path::PathBuf,
}

pub struct ProgressStore {
    pub(crate) state: std::sync::Mutex<ProgressState>,
    pub(crate) paths: ProgressPaths,
    output_rebuild_pending: AtomicBool,
}

impl ProgressStore {
    pub fn load(app: &tauri::AppHandle) -> std::io::Result<Self> {
        use tauri::Manager as _;
        let primary = app
            .path()
            .app_data_dir()
            .map_err(std::io::Error::other)?
            .join(PRIMARY_PROGRESS_FILENAME);
        let paths = ProgressPaths::from_primary(primary);
        let outcome = load_state(&paths)?;
        Ok(Self::from_outcome(paths, outcome))
    }

    fn from_outcome(paths: ProgressPaths, outcome: LoadOutcome) -> Self {
        Self {
            state: std::sync::Mutex::new(outcome.state),
            paths,
            output_rebuild_pending: AtomicBool::new(outcome.needs_output_rebuild),
        }
    }

    pub(crate) fn output_rebuild_pending(&self) -> bool {
        self.output_rebuild_pending.load(Ordering::Acquire)
    }

    pub(crate) fn finish_output_rebuild(&self) {
        self.output_rebuild_pending.store(false, Ordering::Release);
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
    pub needs_output_rebuild: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SaveCheckpoint {
    TemporarySynced,
    BackupReplaced,
    PrimaryReplaced,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RebuildCheckpoint {
    TemporarySynced,
    PrimaryReplaced,
}

impl ProgressPaths {
    pub fn from_primary(primary: std::path::PathBuf) -> Self {
        let dir = parent_directory(&primary).to_path_buf();
        Self {
            primary,
            backup: dir.join(BACKUP_PROGRESS_FILENAME),
            pre_migration_v1: dir.join("progress.pre-migration-v1.json"),
            pre_migration_v2: dir.join("progress.pre-migration-v2.json"),
            temporary: dir.join("progress.json.tmp"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct ProgressEnvelope {
    pub schema_version: u32,
    pub state: ProgressState,
}

#[derive(serde::Deserialize)]
struct ProgressEnvelopeV3Wire {
    schema_version: u32,
    state: ProgressStateV3Wire,
}

#[derive(serde::Deserialize)]
struct ProgressStateV3Wire {
    rank: usize,
    prestige: u32,
    prestige_token_floor: u64,
    initialized: bool,
    tally: TallyStateV3Wire,
}

#[derive(serde::Deserialize)]
struct TallyStateV3Wire {
    output_tokens: u64,
    claude_offsets: std::collections::HashMap<String, u64>,
    codex_offsets: std::collections::HashMap<String, u64>,
    codex_output_totals: std::collections::HashMap<String, u64>,
}

#[derive(serde::Deserialize)]
struct ProgressEnvelopeV2Wire {
    schema_version: u32,
    state: ProgressStateV2Wire,
}

#[derive(serde::Deserialize)]
struct ProgressStateV2Wire {
    rank: usize,
    prestige: u32,
    prestige_token_floor: u64,
    initialized: bool,
    tally: TallyStateV2Wire,
}

#[derive(serde::Deserialize)]
struct TallyStateV2Wire {
    total_tokens: u64,
    claude_offsets: std::collections::HashMap<String, u64>,
    codex_offsets: std::collections::HashMap<String, u64>,
    codex_totals: std::collections::HashMap<String, u64>,
}

#[derive(serde::Deserialize)]
struct LegacyProgressStateWire {
    rank: usize,
    prestige: u32,
    prestige_token_floor: u64,
    #[serde(default, rename = "initialized")]
    _initialized: bool,
    tally: LegacyTallyStateWire,
}

#[derive(serde::Deserialize)]
struct LegacyTallyStateWire {
    total_tokens: u64,
    claude_offsets: std::collections::HashMap<String, u64>,
    #[serde(default)]
    codex_offsets: std::collections::HashMap<String, u64>,
    #[serde(default)]
    codex_totals: std::collections::HashMap<String, u64>,
}

impl ProgressStateV2Wire {
    fn validate(self) -> std::io::Result<()> {
        let rank = self.rank;
        let _required_fields = (
            self.prestige,
            self.prestige_token_floor,
            self.initialized,
            self.tally.total_tokens,
            self.tally.claude_offsets,
            self.tally.codex_offsets,
            self.tally.codex_totals,
        );
        validate_wire_rank(rank)
    }
}

impl LegacyProgressStateWire {
    fn validate(self) -> std::io::Result<()> {
        let rank = self.rank;
        let _required_fields = (
            self.prestige,
            self.prestige_token_floor,
            self._initialized,
            self.tally.total_tokens,
            self.tally.claude_offsets,
            self.tally.codex_offsets,
            self.tally.codex_totals,
        );
        validate_wire_rank(rank)
    }
}

impl From<TallyStateV3Wire> for TallyState {
    fn from(wire: TallyStateV3Wire) -> Self {
        Self {
            output_tokens: wire.output_tokens,
            claude_offsets: wire.claude_offsets,
            codex_offsets: wire.codex_offsets,
            codex_output_totals: wire.codex_output_totals,
        }
    }
}

impl From<ProgressStateV3Wire> for ProgressState {
    fn from(wire: ProgressStateV3Wire) -> Self {
        Self {
            rank: wire.rank,
            prestige: wire.prestige,
            prestige_token_floor: wire.prestige_token_floor,
            initialized: wire.initialized,
            tally: wire.tally.into(),
        }
    }
}

#[derive(Debug, PartialEq)]
enum DecodedState {
    Current(ProgressState),
    NeedsOutputRebuild { source_schema: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SnapshotSchema {
    V1,
    V2,
}

fn invalid_data(error: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string())
}

fn unsupported_schema(error: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Unsupported, error.to_string())
}

#[cfg(test)]
fn invalid_input(error: impl std::fmt::Display) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, error.to_string())
}

#[cfg(all(test, target_os = "macos"))]
fn live_primary_path() -> Option<std::path::PathBuf> {
    std::env::var_os("HOME").map(|home| {
        std::path::PathBuf::from(home)
            .join("Library/Application Support")
            .join("com.vantasoft.mana")
            .join(PRIMARY_PROGRESS_FILENAME)
    })
}

#[cfg(all(test, not(target_os = "macos")))]
fn live_primary_path() -> Option<std::path::PathBuf> {
    None
}

#[cfg(test)]
fn validate_external_v1_fixture(
    source: &std::path::Path,
    live_primary: Option<&std::path::Path>,
) -> std::io::Result<std::path::PathBuf> {
    let metadata = std::fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(invalid_input(format!(
            "external progress fixture is not a regular non-symlink file: {}",
            source.display()
        )));
    }

    let file_name = source.file_name().and_then(|name| name.to_str());
    if file_name == Some(PRIMARY_PROGRESS_FILENAME)
        || !file_name.is_some_and(|name| {
            name.strip_prefix("progress.manual-before-v2-")
                .and_then(|suffix| suffix.strip_suffix(".json"))
                .is_some_and(|suffix| !suffix.is_empty())
        })
    {
        return Err(invalid_input(format!(
            "external progress fixture must be a manual pre-v2 copy: {}",
            source.display()
        )));
    }

    let source = source.canonicalize()?;
    let source_metadata = std::fs::metadata(&source)?;
    if let Some(live_primary) = live_primary.and_then(|path| path.canonicalize().ok()) {
        let live_primary_metadata = std::fs::metadata(&live_primary)?;
        if source == live_primary || files_share_identity(&source_metadata, &live_primary_metadata)
        {
            return Err(invalid_input(
                "external progress fixture resolves to the live primary progress file",
            ));
        }
    }
    Ok(source)
}

#[cfg(all(test, unix))]
fn files_share_identity(source: &std::fs::Metadata, live_primary: &std::fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    source.dev() == live_primary.dev() && source.ino() == live_primary.ino()
}

#[cfg(all(test, not(unix)))]
fn files_share_identity(_: &std::fs::Metadata, _: &std::fs::Metadata) -> bool {
    false
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

fn validate_wire_rank(rank: usize) -> std::io::Result<()> {
    if rank >= TIERS.len() {
        return Err(invalid_data(
            "progress rank is outside the known tier table",
        ));
    }
    Ok(())
}

pub fn encode_state(state: &ProgressState) -> std::io::Result<Vec<u8>> {
    serde_json::to_vec(&ProgressEnvelope {
        schema_version: SCHEMA_VERSION,
        state: state.clone(),
    })
    .map_err(|error| std::io::Error::new(std::io::ErrorKind::Other, error))
}

fn decode_state(bytes: &[u8]) -> std::io::Result<DecodedState> {
    let value = serde_json::from_slice::<serde_json::Value>(bytes).map_err(invalid_data)?;
    if let Some(version_value) = value.get("schema_version") {
        let version = version_value
            .as_u64()
            .ok_or_else(|| invalid_data("progress schema version is not an unsigned integer"))?;
        return match version {
            version if version == u64::from(SCHEMA_VERSION) => {
                let envelope = serde_json::from_value::<ProgressEnvelopeV3Wire>(value)
                    .map_err(invalid_data)?;
                if envelope.schema_version != SCHEMA_VERSION {
                    return Err(unsupported_schema("unsupported progress schema version"));
                }
                Ok(DecodedState::Current(validate_rank(envelope.state.into())?))
            }
            2 => {
                let envelope = serde_json::from_value::<ProgressEnvelopeV2Wire>(value)
                    .map_err(invalid_data)?;
                if envelope.schema_version != 2 {
                    return Err(unsupported_schema("unsupported progress schema version"));
                }
                envelope.state.validate()?;
                Ok(DecodedState::NeedsOutputRebuild { source_schema: 2 })
            }
            _ => Err(unsupported_schema("unsupported progress schema version")),
        };
    }

    let legacy = serde_json::from_value::<LegacyProgressStateWire>(value).map_err(invalid_data)?;
    legacy.validate()?;
    Ok(DecodedState::NeedsOutputRebuild { source_schema: 1 })
}

struct TemporarySnapshot(std::path::PathBuf);

impl Drop for TemporarySnapshot {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn snapshot_matches(decoded: &DecodedState, expected_schema: SnapshotSchema) -> bool {
    matches!(
        (decoded, expected_schema),
        (
            DecodedState::NeedsOutputRebuild { source_schema: 1 },
            SnapshotSchema::V1
        ) | (
            DecodedState::NeedsOutputRebuild { source_schema: 2 },
            SnapshotSchema::V2
        )
    )
}

fn validate_snapshot(
    path: &std::path::Path,
    expected_schema: SnapshotSchema,
) -> std::io::Result<Vec<u8>> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.file_type().is_file() {
        return Err(invalid_data(format!(
            "pre-migration snapshot is not a regular file: {}",
            path.display()
        )));
    }

    let bytes = std::fs::read(path)?;
    let decoded = decode_state(&bytes)?;
    if !snapshot_matches(&decoded, expected_schema) {
        return Err(invalid_data(format!(
            "pre-migration snapshot contains the wrong progress schema: {}",
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

fn write_immutable_snapshot(
    path: &std::path::Path,
    bytes: &[u8],
    expected_schema: SnapshotSchema,
) -> std::io::Result<()> {
    let decoded = decode_state(bytes)?;
    if !snapshot_matches(&decoded, expected_schema) {
        return Err(invalid_data(
            "pre-migration snapshot input has the wrong progress schema",
        ));
    }

    let (temporary, mut file) = create_snapshot_temporary(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    drop(file);

    let staged_bytes = std::fs::read(&temporary.0)?;
    let staged = decode_state(&staged_bytes)?;
    if !snapshot_matches(&staged, expected_schema) {
        return Err(invalid_data(
            "staged pre-migration snapshot has the wrong progress schema",
        ));
    }
    if staged_bytes != bytes {
        return Err(invalid_data(
            "staged pre-migration snapshot differs from source progress",
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
            validate_snapshot(path, expected_schema)?;
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
    expected_snapshot: Option<SnapshotSchema>,
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
        Ok(decoded) => Ok(expected_snapshot
            .map(|schema| snapshot_matches(&decoded, schema))
            .unwrap_or(true)),
        Err(error) if error.kind() == std::io::ErrorKind::InvalidData => Ok(false),
        Err(error) => Err(error),
    }
}

fn has_stable_recovery_source(
    paths: &ProgressPaths,
    primary_is_valid: bool,
) -> std::io::Result<bool> {
    if primary_is_valid || regular_file_contains_valid_state(&paths.backup, None)? {
        return Ok(true);
    }
    if regular_file_contains_valid_state(&paths.pre_migration_v2, Some(SnapshotSchema::V2))? {
        return Ok(true);
    }
    regular_file_contains_valid_state(&paths.pre_migration_v1, Some(SnapshotSchema::V1))
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
    if !matches!(decode_state(&staged_bytes)?, DecodedState::Current(_)) {
        return Err(invalid_data(
            "staged progress temporary is not current progress",
        ));
    }
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

fn snapshot_for_source(
    paths: &ProgressPaths,
    source: &[u8],
    source_schema: u32,
) -> std::io::Result<()> {
    let (path, expected_schema) = match source_schema {
        1 => (&paths.pre_migration_v1, SnapshotSchema::V1),
        2 => (&paths.pre_migration_v2, SnapshotSchema::V2),
        _ => return Err(invalid_data("unsupported pre-migration schema")),
    };
    let snapshot = validate_snapshot(path, expected_schema)?;
    if snapshot != source {
        return Err(invalid_data(format!(
            "pre-migration snapshot does not match source progress: {}",
            path.display()
        )));
    }
    Ok(())
}

enum MigrationPublicationStatus {
    Pending,
    AlreadyPublished,
}

fn require_any_migration_snapshot(paths: &ProgressPaths) -> std::io::Result<()> {
    let mut snapshot_error = None;
    for (path, schema) in [
        (&paths.pre_migration_v2, SnapshotSchema::V2),
        (&paths.pre_migration_v1, SnapshotSchema::V1),
    ] {
        match validate_snapshot(path, schema) {
            Ok(_) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                if snapshot_error.is_none() {
                    snapshot_error = Some(error);
                }
            }
        }
    }
    Err(snapshot_error.unwrap_or_else(|| {
        invalid_data("output rebuild has no valid pre-migration source snapshot")
    }))
}

fn require_matching_migration_snapshot(
    paths: &ProgressPaths,
    rebuilt: &ProgressState,
) -> std::io::Result<MigrationPublicationStatus> {
    let candidates = [(&paths.primary, true), (&paths.backup, false)];
    let mut recovery_error = None;

    for (path, is_primary) in candidates {
        let bytes = match read_candidate(path, false) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => continue,
            Err(CandidateReadError::RegularFile(error)) if is_primary => return Err(error),
            Err(error) => {
                if recovery_error.is_none() {
                    recovery_error = Some(error.into_inner());
                }
                continue;
            }
        };
        match decode_state(&bytes) {
            Ok(DecodedState::NeedsOutputRebuild { source_schema }) => {
                snapshot_for_source(paths, &bytes, source_schema)?;
                return Ok(MigrationPublicationStatus::Pending);
            }
            Ok(DecodedState::Current(state)) if is_primary && state == *rebuilt => {
                require_any_migration_snapshot(paths)?;
                return Ok(MigrationPublicationStatus::AlreadyPublished);
            }
            Ok(DecodedState::Current(_)) => {
                return Err(invalid_data(
                    "cannot publish a migration rebuild over current progress",
                ));
            }
            Err(error) if is_primary && error.kind() == std::io::ErrorKind::Unsupported => {
                return Err(error);
            }
            Err(error) => {
                if recovery_error.is_none() {
                    recovery_error = Some(error);
                }
            }
        }
    }

    for (path, schema) in [
        (&paths.pre_migration_v2, SnapshotSchema::V2),
        (&paths.pre_migration_v1, SnapshotSchema::V1),
    ] {
        match validate_snapshot(path, schema) {
            Ok(_) => return Ok(MigrationPublicationStatus::Pending),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                if recovery_error.is_none() {
                    recovery_error = Some(error);
                }
            }
        }
    }

    Err(recovery_error.unwrap_or_else(|| {
        invalid_data("output rebuild has no valid pre-migration source snapshot")
    }))
}

fn rebuilt_primary_is_visible(
    paths: &ProgressPaths,
    expected_bytes: &[u8],
) -> std::io::Result<bool> {
    let metadata = std::fs::symlink_metadata(&paths.primary)?;
    if !metadata.file_type().is_file() {
        return Ok(false);
    }
    let bytes = std::fs::read(&paths.primary)?;
    if bytes != expected_bytes {
        return Ok(false);
    }
    Ok(matches!(decode_state(&bytes)?, DecodedState::Current(_)))
}

fn committed_or_error(
    paths: &ProgressPaths,
    expected_bytes: &[u8],
    error: std::io::Error,
) -> std::io::Result<()> {
    if rebuilt_primary_is_visible(paths, expected_bytes)? {
        Ok(())
    } else {
        Err(error)
    }
}

fn publish_rebuilt_state_with_hook(
    paths: &ProgressPaths,
    state: &ProgressState,
    mut hook: impl FnMut(RebuildCheckpoint) -> std::io::Result<()>,
) -> std::io::Result<()> {
    if matches!(
        require_matching_migration_snapshot(paths, state)?,
        MigrationPublicationStatus::AlreadyPublished
    ) {
        return Ok(());
    }
    let bytes = encode_state(state)?;
    let DecodedState::Current(encoded_state) = decode_state(&bytes)? else {
        return Err(invalid_data("rebuilt progress is not schema v3"));
    };
    if encoded_state != *state {
        return Err(invalid_data(
            "encoded rebuilt progress differs from requested state",
        ));
    }

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
    if staged_bytes != bytes {
        return Err(invalid_data(
            "staged rebuilt progress differs from encoded progress",
        ));
    }
    let DecodedState::Current(staged_state) = decode_state(&staged_bytes)? else {
        return Err(invalid_data("staged rebuilt progress is not schema v3"));
    };
    if staged_state != *state {
        return Err(invalid_data(
            "staged rebuilt progress differs from requested state",
        ));
    }
    validate_staged_temporary(&paths.temporary, &temporary_metadata)?;
    hook(RebuildCheckpoint::TemporarySynced)?;

    validate_staged_temporary(&paths.temporary, &temporary_metadata)?;
    if matches!(
        require_matching_migration_snapshot(paths, state)?,
        MigrationPublicationStatus::AlreadyPublished
    ) {
        drop(temporary);
        let _ = remove_temporary(&paths.temporary);
        return Ok(());
    }
    drop(temporary);
    std::fs::rename(&paths.temporary, &paths.primary)?;
    if let Err(error) = hook(RebuildCheckpoint::PrimaryReplaced) {
        return committed_or_error(paths, &bytes, error);
    }
    if let Err(error) = sync_parent_directory(&paths.primary) {
        return committed_or_error(paths, &bytes, error);
    }
    Ok(())
}

pub fn publish_rebuilt_state(paths: &ProgressPaths, state: &ProgressState) -> std::io::Result<()> {
    publish_rebuilt_state_with_hook(paths, state, |_| Ok(()))
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
    let DecodedState::Current(state) = decode_state(&bytes)? else {
        return Err(invalid_data(format!(
            "progress temporary contains pre-v3 progress: {}",
            paths.temporary.display()
        )));
    };

    validate_staged_temporary(&paths.temporary, &temporary_metadata)?;
    drop(temporary);
    std::fs::rename(&paths.temporary, &paths.primary)?;
    sync_parent_directory(&paths.primary)?;
    Ok(Some(LoadOutcome {
        state,
        source: RecoverySource::Temporary,
        needs_output_rebuild: false,
    }))
}

pub fn load_state(paths: &ProgressPaths) -> std::io::Result<LoadOutcome> {
    let candidates = [
        (&paths.primary, RecoverySource::Primary, None),
        (&paths.backup, RecoverySource::Backup, None),
        (
            &paths.pre_migration_v2,
            RecoverySource::PreMigration,
            Some(SnapshotSchema::V2),
        ),
        (
            &paths.pre_migration_v1,
            RecoverySource::PreMigration,
            Some(SnapshotSchema::V1),
        ),
    ];
    let mut recovery_error = None;

    for (path, source, expected_snapshot) in candidates {
        let bytes = match read_candidate(path, expected_snapshot.is_some()) {
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
            Ok(DecodedState::Current(state)) => {
                if expected_snapshot.is_some() {
                    if recovery_error.is_none() {
                        recovery_error = Some(invalid_data(
                            "pre-migration snapshot contains current progress",
                        ));
                    }
                    continue;
                }
                if source != RecoverySource::Primary {
                    save_state(paths, &state)?;
                }
                remove_temporary(&paths.temporary)?;
                return Ok(LoadOutcome {
                    state,
                    source,
                    needs_output_rebuild: false,
                });
            }
            Ok(DecodedState::NeedsOutputRebuild { source_schema }) => {
                let snapshot_schema = match source_schema {
                    1 => SnapshotSchema::V1,
                    2 => SnapshotSchema::V2,
                    _ => {
                        if recovery_error.is_none() {
                            recovery_error = Some(invalid_data("unsupported pre-migration schema"));
                        }
                        continue;
                    }
                };
                if expected_snapshot.is_some_and(|expected| expected != snapshot_schema) {
                    if recovery_error.is_none() {
                        recovery_error = Some(invalid_data(
                            "pre-migration snapshot contains the wrong progress schema",
                        ));
                    }
                    continue;
                }
                if expected_snapshot.is_none() {
                    let snapshot_path = match snapshot_schema {
                        SnapshotSchema::V1 => &paths.pre_migration_v1,
                        SnapshotSchema::V2 => &paths.pre_migration_v2,
                    };
                    write_immutable_snapshot(snapshot_path, &bytes, snapshot_schema)?;
                }
                remove_temporary(&paths.temporary)?;
                return Ok(LoadOutcome {
                    state: ProgressState::default(),
                    source,
                    needs_output_rebuild: true,
                });
            }
            Err(error) => {
                if source == RecoverySource::Primary
                    && error.kind() == std::io::ErrorKind::Unsupported
                {
                    return Err(error);
                }
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
        needs_output_rebuild: false,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        decode_state, encode_state, load_state, publish_rebuilt_state,
        publish_rebuilt_state_with_hook, save_state, save_state_with_hook,
        validate_external_v1_fixture, DecodedState, ProgressEnvelope, ProgressPaths, ProgressStore,
        RebuildCheckpoint, RecoverySource, SaveCheckpoint, SCHEMA_VERSION,
    };
    use crate::progress::{ProgressState, TallyState};

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

    fn v2_paths(label: &str) -> (TestDirectory, ProgressPaths) {
        let (directory, paths) = test_paths(label);
        std::fs::write(
            &paths.primary,
            include_bytes!("../tests/fixtures/progress_v2.json"),
        )
        .unwrap();
        let loaded = load_state(&paths).unwrap();
        assert!(loaded.needs_output_rebuild);
        (directory, paths)
    }

    fn fixture_state() -> ProgressState {
        output_fixture_state()
    }

    fn output_fixture_state() -> ProgressState {
        ProgressState {
            rank: 8,
            prestige: 7,
            prestige_token_floor: 123_456,
            initialized: true,
            tally: TallyState {
                output_tokens: 987_654_321,
                claude_offsets: std::collections::HashMap::from([(
                    "/fixture/claude.jsonl".into(),
                    44,
                )]),
                codex_offsets: std::collections::HashMap::from([(
                    "/fixture/codex.jsonl".into(),
                    55,
                )]),
                codex_output_totals: std::collections::HashMap::from([(
                    "/fixture/codex.jsonl".into(),
                    66,
                )]),
            },
        }
    }

    fn state_with_rank(rank: usize) -> ProgressState {
        let mut state = fixture_state();
        state.rank = rank;
        state
    }

    fn load_exact(path: &std::path::Path) -> std::io::Result<ProgressState> {
        let bytes = std::fs::read(path)?;
        match decode_state(&bytes)? {
            DecodedState::Current(state) => Ok(state),
            DecodedState::NeedsOutputRebuild { .. } => {
                Err(super::invalid_data("progress is not schema v3"))
            }
        }
    }

    fn load_exact_v3(path: &std::path::Path) -> std::io::Result<ProgressState> {
        match decode_state(&std::fs::read(path)?)? {
            DecodedState::Current(state) => Ok(state),
            DecodedState::NeedsOutputRebuild { .. } => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "expected schema v3",
            )),
        }
    }

    #[test]
    fn interrupted_rebuild_staging_keeps_v2_primary_authoritative() {
        let (_dir, paths) = v2_paths("rebuild-stage-failure");
        let v2 = std::fs::read(&paths.primary).unwrap();
        let rebuilt = output_fixture_state();

        publish_rebuilt_state_with_hook(&paths, &rebuilt, |checkpoint| {
            if checkpoint == RebuildCheckpoint::TemporarySynced {
                return Err(std::io::Error::other("simulated interruption"));
            }
            Ok(())
        })
        .unwrap_err();

        assert_eq!(std::fs::read(&paths.primary).unwrap(), v2);
        assert_eq!(std::fs::read(&paths.pre_migration_v2).unwrap(), v2);
    }

    #[test]
    fn successful_rebuild_replaces_v2_with_complete_v3() {
        let (_dir, paths) = v2_paths("rebuild-success");
        let rebuilt = output_fixture_state();

        publish_rebuilt_state(&paths, &rebuilt).unwrap();

        assert_eq!(load_exact_v3(&paths.primary).unwrap(), rebuilt);
        assert!(!paths.backup.exists());
        assert!(!load_state(&paths).unwrap().needs_output_rebuild);
    }

    #[test]
    fn interruption_after_rebuild_rename_is_a_committed_success() {
        let (_dir, paths) = v2_paths("rebuild-post-rename");
        let rebuilt = output_fixture_state();

        publish_rebuilt_state_with_hook(&paths, &rebuilt, |checkpoint| {
            if checkpoint == RebuildCheckpoint::PrimaryReplaced {
                return Err(std::io::Error::other("simulated directory sync failure"));
            }
            Ok(())
        })
        .unwrap();

        assert_eq!(load_exact_v3(&paths.primary).unwrap(), rebuilt);
    }

    #[test]
    fn rebuild_retries_after_interrupted_staging() {
        let (_dir, paths) = v2_paths("rebuild-stage-retry");
        let rebuilt = output_fixture_state();
        publish_rebuilt_state_with_hook(&paths, &rebuilt, |checkpoint| {
            if checkpoint == RebuildCheckpoint::TemporarySynced {
                return Err(std::io::Error::other("simulated interruption"));
            }
            Ok(())
        })
        .unwrap_err();

        publish_rebuilt_state(&paths, &rebuilt).unwrap();

        assert_eq!(load_exact_v3(&paths.primary).unwrap(), rebuilt);
        assert!(!paths.temporary.exists());
    }

    #[test]
    fn rebuild_does_not_rotate_v2_over_an_existing_backup() {
        let (_dir, paths) = v2_paths("rebuild-preserves-backup");
        let backup = encode_state(&state_with_rank(3)).unwrap();
        std::fs::write(&paths.backup, &backup).unwrap();

        publish_rebuilt_state(&paths, &output_fixture_state()).unwrap();

        assert_eq!(std::fs::read(&paths.backup).unwrap(), backup);
    }

    #[test]
    fn rebuild_does_not_replace_future_schema_that_appears_after_staging() {
        let (_dir, paths) = v2_paths("rebuild-future-primary-race");
        let rebuilt = output_fixture_state();
        let mut future = serde_json::to_value(ProgressEnvelope {
            schema_version: SCHEMA_VERSION,
            state: state_with_rank(3),
        })
        .unwrap();
        future["schema_version"] = (SCHEMA_VERSION + 1).into();
        let future = serde_json::to_vec(&future).unwrap();

        let error = publish_rebuilt_state_with_hook(&paths, &rebuilt, |checkpoint| {
            if checkpoint == RebuildCheckpoint::TemporarySynced {
                std::fs::write(&paths.primary, &future).unwrap();
            }
            Ok(())
        })
        .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), future);
        assert_eq!(
            std::fs::read(&paths.pre_migration_v2).unwrap(),
            include_bytes!("../tests/fixtures/progress_v2.json")
        );
    }

    #[test]
    fn progress_store_retains_and_clears_loaded_rebuild_status() {
        let (_dir, paths) = v2_paths("store-rebuild-flag");
        let loaded = load_state(&paths).unwrap();
        let store = ProgressStore::from_outcome(paths, loaded);

        assert!(store.output_rebuild_pending());
        store.finish_output_rebuild();
        assert!(!store.output_rebuild_pending());
    }

    #[test]
    fn restarted_v3_store_has_no_pending_rebuild() {
        let (_dir, paths) = v2_paths("store-v3-restart");
        publish_rebuilt_state(&paths, &output_fixture_state()).unwrap();
        let loaded = load_state(&paths).unwrap();
        let store = ProgressStore::from_outcome(paths, loaded);

        assert!(!store.output_rebuild_pending());
    }

    #[test]
    fn rebuild_rejects_snapshot_that_does_not_match_v2_primary() {
        let (_dir, paths) = test_paths("rebuild-mismatched-v2-snapshot");
        let snapshot = include_bytes!("../tests/fixtures/progress_v2.json");
        let mut changed = serde_json::from_slice::<serde_json::Value>(snapshot).unwrap();
        changed["state"]["rank"] = 3.into();
        let changed = serde_json::to_vec(&changed).unwrap();
        std::fs::write(&paths.pre_migration_v2, snapshot).unwrap();
        std::fs::write(&paths.primary, &changed).unwrap();
        assert!(load_state(&paths).unwrap().needs_output_rebuild);

        let error = publish_rebuilt_state(&paths, &output_fixture_state()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), changed);
        assert_eq!(std::fs::read(&paths.pre_migration_v2).unwrap(), snapshot);
    }

    #[test]
    fn successful_legacy_rebuild_uses_v1_snapshot() {
        let (_dir, paths) = test_paths("rebuild-v1-success");
        let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.primary, legacy).unwrap();
        assert!(load_state(&paths).unwrap().needs_output_rebuild);
        let rebuilt = output_fixture_state();

        publish_rebuilt_state(&paths, &rebuilt).unwrap();

        assert_eq!(load_exact_v3(&paths.primary).unwrap(), rebuilt);
        assert_eq!(std::fs::read(&paths.pre_migration_v1).unwrap(), legacy);
    }

    #[test]
    fn rebuild_publisher_rejects_unrelated_current_v3_without_a_snapshot() {
        let (_dir, paths) = test_paths("rebuild-current-v3-no-snapshot");
        let current = encode_state(&output_fixture_state()).unwrap();
        std::fs::write(&paths.primary, &current).unwrap();

        let error = publish_rebuilt_state(&paths, &output_fixture_state()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), current);
        assert!(!paths.temporary.exists());
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
        assert!(!paths.pre_migration_v1.exists());
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
        assert!(!loaded.needs_output_rebuild);
        assert!(!loaded.state.initialized);
    }

    #[test]
    fn valid_v2_primary_returns_fresh_rebuild_without_replacing_primary() {
        let (_dir, paths) = test_paths("v2-rebuild");
        let v2 = include_bytes!("../tests/fixtures/progress_v2.json");
        std::fs::write(&paths.primary, v2).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert!(loaded.needs_output_rebuild);
        assert_eq!(loaded.state, ProgressState::default());
        assert_eq!(std::fs::read(&paths.primary).unwrap(), v2);
        assert_eq!(std::fs::read(&paths.pre_migration_v2).unwrap(), v2);
    }

    #[test]
    fn existing_valid_v2_recovery_is_never_overwritten() {
        let (_dir, paths) = test_paths("v2-immutable");
        let first = include_bytes!("../tests/fixtures/progress_v2.json");
        let mut second = serde_json::from_slice::<serde_json::Value>(first).unwrap();
        second["state"]["rank"] = 3.into();
        std::fs::write(&paths.pre_migration_v2, first).unwrap();
        std::fs::write(&paths.primary, serde_json::to_vec(&second).unwrap()).unwrap();

        load_state(&paths).unwrap();

        assert_eq!(std::fs::read(&paths.pre_migration_v2).unwrap(), first);
    }

    #[test]
    fn valid_v3_primary_never_requests_output_rebuild() {
        let (_dir, paths) = test_paths("v3-no-rebuild");
        let state = output_fixture_state();
        std::fs::write(&paths.primary, encode_state(&state).unwrap()).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.state, state);
        assert!(!loaded.needs_output_rebuild);
    }

    #[test]
    fn incomplete_v2_primary_uses_valid_v3_backup() {
        let (_dir, paths) = test_paths("incomplete-v2-valid-v3-backup");
        let mut damaged = serde_json::from_slice::<serde_json::Value>(include_bytes!(
            "../tests/fixtures/progress_v2.json"
        ))
        .unwrap();
        damaged["state"]["tally"]
            .as_object_mut()
            .unwrap()
            .remove("total_tokens");
        std::fs::write(&paths.primary, serde_json::to_vec(&damaged).unwrap()).unwrap();
        let backup_state = output_fixture_state();
        let backup = encode_state(&backup_state).unwrap();
        std::fs::write(&paths.backup, &backup).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state, backup_state);
        assert!(!loaded.needs_output_rebuild);
        assert_eq!(std::fs::read(&paths.backup).unwrap(), backup);
        assert!(!paths.pre_migration_v2.exists());
    }

    #[test]
    fn incomplete_v3_primary_uses_valid_v3_backup() {
        let (_dir, paths) = test_paths("incomplete-v3-valid-v3-backup");
        let mut damaged = serde_json::to_value(ProgressEnvelope {
            schema_version: SCHEMA_VERSION,
            state: output_fixture_state(),
        })
        .unwrap();
        damaged["state"]["tally"]
            .as_object_mut()
            .unwrap()
            .remove("output_tokens");
        std::fs::write(&paths.primary, serde_json::to_vec(&damaged).unwrap()).unwrap();
        let backup_state = state_with_rank(7);
        let backup = encode_state(&backup_state).unwrap();
        std::fs::write(&paths.backup, &backup).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state, backup_state);
        assert!(!loaded.needs_output_rebuild);
        assert_eq!(std::fs::read(&paths.backup).unwrap(), backup);
        assert!(!paths.pre_migration_v1.exists());
        assert!(!paths.pre_migration_v2.exists());
    }

    #[test]
    fn future_schema_primary_fails_closed_before_backup_recovery() {
        let (_dir, paths) = test_paths("future-schema-primary");
        let mut future = serde_json::to_value(ProgressEnvelope {
            schema_version: SCHEMA_VERSION,
            state: output_fixture_state(),
        })
        .unwrap();
        future["schema_version"] = (SCHEMA_VERSION + 1).into();
        let primary = serde_json::to_vec(&future).unwrap();
        let backup = encode_state(&state_with_rank(7)).unwrap();
        let temporary = b"temporary must not be replaced";
        std::fs::write(&paths.primary, &primary).unwrap();
        std::fs::write(&paths.backup, &backup).unwrap();
        std::fs::write(&paths.temporary, temporary).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), primary);
        assert_eq!(std::fs::read(&paths.backup).unwrap(), backup);
        assert_eq!(std::fs::read(&paths.temporary).unwrap(), temporary);
        assert!(!paths.pre_migration_v1.exists());
        assert!(!paths.pre_migration_v2.exists());
    }

    #[test]
    fn future_schema_primary_fails_closed_before_every_recovery_candidate() {
        for candidate in [
            "backup",
            "pre-migration-v2",
            "pre-migration-v1",
            "temporary",
        ] {
            let (_dir, paths) = test_paths(&format!("future-schema-{candidate}"));
            let mut future = serde_json::to_value(ProgressEnvelope {
                schema_version: SCHEMA_VERSION,
                state: output_fixture_state(),
            })
            .unwrap();
            future["schema_version"] = (SCHEMA_VERSION + 1).into();
            let primary = serde_json::to_vec(&future).unwrap();
            std::fs::write(&paths.primary, &primary).unwrap();

            let (path, bytes) = match candidate {
                "backup" => (&paths.backup, encode_state(&state_with_rank(7)).unwrap()),
                "pre-migration-v2" => (
                    &paths.pre_migration_v2,
                    include_bytes!("../tests/fixtures/progress_v2.json").to_vec(),
                ),
                "pre-migration-v1" => (
                    &paths.pre_migration_v1,
                    include_bytes!("../tests/fixtures/progress_v1.json").to_vec(),
                ),
                "temporary" => (&paths.temporary, encode_state(&state_with_rank(6)).unwrap()),
                _ => unreachable!(),
            };
            std::fs::write(path, &bytes).unwrap();

            let error = load_state(&paths).unwrap_err();

            assert_eq!(error.kind(), std::io::ErrorKind::Unsupported, "{candidate}");
            assert_eq!(
                std::fs::read(&paths.primary).unwrap(),
                primary,
                "{candidate}"
            );
            assert_eq!(std::fs::read(path).unwrap(), bytes, "{candidate}");
        }
    }

    #[test]
    fn invalid_existing_v2_recovery_fails_closed() {
        let (_dir, paths) = test_paths("invalid-v2-recovery");
        let v2 = include_bytes!("../tests/fixtures/progress_v2.json");
        let invalid = b"{";
        std::fs::write(&paths.primary, v2).unwrap();
        std::fs::write(&paths.pre_migration_v2, invalid).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&paths.pre_migration_v2).unwrap(), invalid);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), v2);
    }

    #[cfg(unix)]
    #[test]
    fn unreadable_existing_v2_recovery_fails_closed() {
        use std::os::unix::fs::PermissionsExt;

        let (_dir, paths) = test_paths("unreadable-v2-recovery");
        let v2 = include_bytes!("../tests/fixtures/progress_v2.json");
        std::fs::write(&paths.primary, v2).unwrap();
        std::fs::write(&paths.pre_migration_v2, v2).unwrap();

        let error = {
            let _permissions = RestoredPermissions::new(&paths.pre_migration_v2);
            let mut unreadable = std::fs::metadata(&paths.pre_migration_v2)
                .unwrap()
                .permissions();
            unreadable.set_mode(0);
            std::fs::set_permissions(&paths.pre_migration_v2, unreadable).unwrap();
            load_state(&paths).unwrap_err()
        };

        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), v2);
    }

    #[cfg(unix)]
    #[test]
    fn symlink_existing_v2_recovery_fails_closed_without_changing_target() {
        use std::os::unix::fs::symlink;

        let (dir, paths) = test_paths("symlink-v2-recovery");
        let v2 = include_bytes!("../tests/fixtures/progress_v2.json");
        let target = dir.0.join("v2-recovery-target.json");
        std::fs::write(&paths.primary, v2).unwrap();
        std::fs::write(&target, v2).unwrap();
        symlink(&target, &paths.pre_migration_v2).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&target).unwrap(), v2);
        assert!(std::fs::symlink_metadata(&paths.pre_migration_v2)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn legacy_snapshot_is_written_once_and_never_overwritten() {
        let (_dir, paths) = test_paths("immutable-legacy");
        let first_legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        let second_legacy = include_bytes!("../tests/fixtures/progress_v1_early.json");

        std::fs::write(&paths.primary, first_legacy).unwrap();
        let first_load = load_state(&paths).unwrap();
        assert_eq!(first_load.source, RecoverySource::Primary);
        assert_eq!(
            std::fs::read(&paths.pre_migration_v1).unwrap(),
            first_legacy
        );

        std::fs::write(&paths.primary, second_legacy).unwrap();
        let second_load = load_state(&paths).unwrap();
        assert_eq!(second_load.source, RecoverySource::Primary);
        assert_eq!(
            std::fs::read(&paths.pre_migration_v1).unwrap(),
            first_legacy
        );
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
            std::fs::write(&paths.pre_migration_v1, invalid_snapshot).unwrap();

            let error = load_state(&paths).unwrap_err();

            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
            assert_eq!(
                std::fs::read(&paths.pre_migration_v1).unwrap(),
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
        std::fs::write(&paths.pre_migration_v1, &versioned_snapshot).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(
            std::fs::read(&paths.pre_migration_v1).unwrap(),
            versioned_snapshot
        );
    }

    #[test]
    fn versioned_pre_migration_snapshot_is_not_a_recovery_source() {
        let (_dir, paths) = test_paths("versioned-snapshot-recovery");
        let versioned_snapshot = encode_state(&state_with_rank(3)).unwrap();
        std::fs::write(&paths.pre_migration_v1, &versioned_snapshot).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(
            std::fs::read(&paths.pre_migration_v1).unwrap(),
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
        assert_eq!(std::fs::read(&paths.pre_migration_v1).unwrap(), legacy);
    }

    #[test]
    fn malformed_snapshot_input_is_not_published() {
        let (_dir, paths) = test_paths("malformed-snapshot-input");

        let error = super::write_immutable_snapshot(
            &paths.pre_migration_v1,
            b"{",
            super::SnapshotSchema::V1,
        )
        .unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert!(!paths.pre_migration_v1.exists());
    }

    #[test]
    fn primary_takes_precedence_over_backup_and_snapshot() {
        let (_dir, paths) = test_paths("primary-precedence");
        std::fs::write(&paths.primary, encode_state(&state_with_rank(3)).unwrap()).unwrap();
        std::fs::write(&paths.backup, encode_state(&state_with_rank(4)).unwrap()).unwrap();
        std::fs::write(
            &paths.pre_migration_v1,
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
            &paths.pre_migration_v1,
            encode_state(&state_with_rank(5)).unwrap(),
        )
        .unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state.rank, 4);
    }

    #[test]
    fn legacy_backup_is_snapshotted_without_publishing_v3() {
        let (_dir, paths) = test_paths("legacy-backup");
        let legacy = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.primary, b"invalid primary").unwrap();
        std::fs::write(&paths.backup, legacy).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state, ProgressState::default());
        assert!(loaded.needs_output_rebuild);
        assert_eq!(std::fs::read(&paths.pre_migration_v1).unwrap(), legacy);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), b"invalid primary");
        assert_eq!(std::fs::read(&paths.backup).unwrap(), legacy);
    }

    #[test]
    fn invalid_primary_and_backup_recover_from_pre_migration_snapshot() {
        let (_dir, paths) = test_paths("snapshot-recovery");
        std::fs::write(&paths.primary, b"invalid primary").unwrap();
        std::fs::write(&paths.backup, b"invalid backup").unwrap();
        let snapshot = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.pre_migration_v1, snapshot).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::PreMigration);
        assert_eq!(loaded.state, ProgressState::default());
        assert!(loaded.needs_output_rebuild);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), b"invalid primary");
        assert_eq!(std::fs::read(&paths.backup).unwrap(), b"invalid backup");
    }

    #[test]
    fn snapshot_recovery_preserves_all_candidates_until_rebuild() {
        let (_dir, paths) = test_paths("candidate-preservation");
        let invalid_backup = b"{\"schema_version\":2}";
        let snapshot = include_bytes!("../tests/fixtures/progress_v1.json");
        std::fs::write(&paths.primary, b"invalid primary").unwrap();
        std::fs::write(&paths.backup, invalid_backup).unwrap();
        std::fs::write(&paths.pre_migration_v1, snapshot).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert!(loaded.needs_output_rebuild);
        assert_eq!(std::fs::read(&paths.primary).unwrap(), b"invalid primary");
        assert_eq!(std::fs::read(&paths.backup).unwrap(), invalid_backup);
        assert_eq!(std::fs::read(&paths.pre_migration_v1).unwrap(), snapshot);
    }

    #[test]
    fn incomplete_v2_primary_recovers_complete_backup_without_snapshot_or_backup_rotation() {
        let (_dir, paths) = test_paths("incomplete-v2-primary");
        let mut damaged = serde_json::from_slice::<serde_json::Value>(include_bytes!(
            "../tests/fixtures/progress_v2.json"
        ))
        .unwrap();
        damaged["state"]["tally"]
            .as_object_mut()
            .unwrap()
            .remove("total_tokens");
        std::fs::write(&paths.primary, serde_json::to_vec(&damaged).unwrap()).unwrap();
        let backup = encode_state(&state_with_rank(7)).unwrap();
        std::fs::write(&paths.backup, &backup).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state, state_with_rank(7));
        assert!(!paths.pre_migration_v2.exists());
        assert_eq!(std::fs::read(&paths.backup).unwrap(), backup);
    }

    #[test]
    fn incomplete_legacy_primary_recovers_complete_backup_without_snapshot_or_backup_rotation() {
        let (_dir, paths) = test_paths("incomplete-legacy-primary");
        let mut damaged = serde_json::from_slice::<serde_json::Value>(include_bytes!(
            "../tests/fixtures/progress_v1.json"
        ))
        .unwrap();
        damaged["tally"]
            .as_object_mut()
            .unwrap()
            .remove("total_tokens");
        std::fs::write(&paths.primary, serde_json::to_vec(&damaged).unwrap()).unwrap();
        let backup = encode_state(&state_with_rank(7)).unwrap();
        std::fs::write(&paths.backup, &backup).unwrap();

        let loaded = load_state(&paths).unwrap();

        assert_eq!(loaded.source, RecoverySource::Backup);
        assert_eq!(loaded.state, state_with_rank(7));
        assert!(!paths.pre_migration_v1.exists());
        assert_eq!(std::fs::read(&paths.backup).unwrap(), backup);
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
        symlink(&target, &paths.pre_migration_v1).unwrap();

        let error = load_state(&paths).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
        assert_eq!(std::fs::read(&target).unwrap(), legacy);
        assert!(std::fs::symlink_metadata(&paths.pre_migration_v1)
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn valid_unversioned_state_requires_output_rebuild() {
        let bytes = include_bytes!("../tests/fixtures/progress_v1.json");
        assert_eq!(
            decode_state(bytes).unwrap(),
            DecodedState::NeedsOutputRebuild { source_schema: 1 }
        );
    }

    #[test]
    fn early_unversioned_state_requires_output_rebuild() {
        let bytes = include_bytes!("../tests/fixtures/progress_v1_early.json");
        assert_eq!(
            decode_state(bytes).unwrap(),
            DecodedState::NeedsOutputRebuild { source_schema: 1 }
        );
    }

    #[test]
    fn unversioned_initialized_false_state_requires_output_rebuild() {
        let bytes = include_bytes!("../tests/fixtures/progress_v1_initialized_false.json");
        assert_eq!(
            decode_state(bytes).unwrap(),
            DecodedState::NeedsOutputRebuild { source_schema: 1 }
        );
    }

    #[test]
    fn version_three_roundtrips_output_fields() {
        let original = output_fixture_state();
        let bytes = encode_state(&original).unwrap();
        let decoded = decode_state(&bytes).unwrap();
        assert_eq!(decoded, DecodedState::Current(original));
        let document = serde_json::from_slice::<serde_json::Value>(&bytes).unwrap();
        assert_eq!(document["schema_version"], 3);
        assert!(document["state"]["tally"].get("output_tokens").is_some());
        assert!(document["state"]["tally"]
            .get("codex_output_totals")
            .is_some());
        assert!(document["state"]["tally"].get("total_tokens").is_none());
        assert!(document["state"]["tally"].get("codex_totals").is_none());
    }

    #[test]
    fn rejects_v3_without_initialized() {
        let mut document = serde_json::to_value(ProgressEnvelope {
            schema_version: SCHEMA_VERSION,
            state: fixture_state(),
        })
        .unwrap();
        document["state"]
            .as_object_mut()
            .unwrap()
            .remove("initialized");

        let error = decode_state(&serde_json::to_vec(&document).unwrap()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_v3_without_output_tokens() {
        let mut document = serde_json::to_value(ProgressEnvelope {
            schema_version: SCHEMA_VERSION,
            state: fixture_state(),
        })
        .unwrap();
        document["state"]["tally"]
            .as_object_mut()
            .unwrap()
            .remove("output_tokens");

        let error = decode_state(&serde_json::to_vec(&document).unwrap()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_v3_without_claude_offsets() {
        let mut document = serde_json::to_value(ProgressEnvelope {
            schema_version: SCHEMA_VERSION,
            state: fixture_state(),
        })
        .unwrap();
        document["state"]["tally"]
            .as_object_mut()
            .unwrap()
            .remove("claude_offsets");

        let error = decode_state(&serde_json::to_vec(&document).unwrap()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_v3_without_codex_offsets() {
        let mut document = serde_json::to_value(ProgressEnvelope {
            schema_version: SCHEMA_VERSION,
            state: fixture_state(),
        })
        .unwrap();
        document["state"]["tally"]
            .as_object_mut()
            .unwrap()
            .remove("codex_offsets");

        let error = decode_state(&serde_json::to_vec(&document).unwrap()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_v3_without_codex_output_totals() {
        let mut document = serde_json::to_value(ProgressEnvelope {
            schema_version: SCHEMA_VERSION,
            state: fixture_state(),
        })
        .unwrap();
        document["state"]["tally"]
            .as_object_mut()
            .unwrap()
            .remove("codex_output_totals");

        let error = decode_state(&serde_json::to_vec(&document).unwrap()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_v2_with_any_required_field_missing() {
        for path in [
            &["state", "initialized"][..],
            &["state", "tally", "total_tokens"][..],
            &["state", "tally", "claude_offsets"][..],
            &["state", "tally", "codex_offsets"][..],
            &["state", "tally", "codex_totals"][..],
        ] {
            let mut document = serde_json::from_slice::<serde_json::Value>(include_bytes!(
                "../tests/fixtures/progress_v2.json"
            ))
            .unwrap();
            let (field, parents) = path.split_last().unwrap();
            let mut parent = &mut document;
            for key in parents {
                parent = &mut parent[*key];
            }
            parent.as_object_mut().unwrap().remove(*field);

            let error = decode_state(&serde_json::to_vec(&document).unwrap()).unwrap_err();

            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData, "{path:?}");
        }
    }

    #[test]
    fn rejects_legacy_without_total_tokens() {
        let mut document = serde_json::from_slice::<serde_json::Value>(include_bytes!(
            "../tests/fixtures/progress_v1.json"
        ))
        .unwrap();
        document["tally"]
            .as_object_mut()
            .unwrap()
            .remove("total_tokens");

        let error = decode_state(&serde_json::to_vec(&document).unwrap()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_legacy_without_claude_offsets() {
        let mut document = serde_json::from_slice::<serde_json::Value>(include_bytes!(
            "../tests/fixtures/progress_v1.json"
        ))
        .unwrap();
        document["tally"]
            .as_object_mut()
            .unwrap()
            .remove("claude_offsets");

        let error = decode_state(&serde_json::to_vec(&document).unwrap()).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    }

    #[test]
    fn rejects_unknown_future_schema() {
        let error = decode_state(br#"{"schema_version":99,"state":{}}"#).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    }

    #[test]
    fn rejects_flattened_document_with_future_schema_version() {
        let error = decode_state(
            br#"{"schema_version":99,"rank":8,"prestige":7,"prestige_token_floor":123456,"initialized":true,"tally":{}}"#,
        )
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    }

    #[test]
    fn rejects_unversioned_shape_with_non_numeric_schema_version() {
        let mut document = serde_json::from_slice::<serde_json::Value>(include_bytes!(
            "../tests/fixtures/progress_v1.json"
        ))
        .unwrap();
        document["schema_version"] = "future".into();

        let error = decode_state(&serde_json::to_vec(&document).unwrap()).unwrap_err();

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
    fn external_fixture_rejects_live_progress_filename() {
        let (dir, _paths) = test_paths("external-fixture-live-name");
        let source = dir.0.join("progress.json");
        std::fs::write(&source, b"fixture").unwrap();

        let error = validate_external_v1_fixture(&source, None).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn external_fixture_rejects_wrong_manual_copy_filename_without_reading_it() {
        let (dir, _paths) = test_paths("external-fixture-wrong-name");
        let source = dir.0.join("progress.manual-before-v2-.json");
        std::fs::write(&source, b"fixture content must remain unread").unwrap();

        let error = validate_external_v1_fixture(&source, None).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(source.is_file());
    }

    #[test]
    fn external_fixture_rejects_canonical_live_primary_alias_without_reading_it() {
        let (dir, _paths) = test_paths("external-fixture-canonical-alias");
        let source = dir.0.join("progress.manual-before-v2-20260722-195941.json");
        let live_primary = dir
            .0
            .join(".")
            .join("progress.manual-before-v2-20260722-195941.json");
        std::fs::write(&source, b"fixture content must remain unread").unwrap();

        let error = validate_external_v1_fixture(&source, Some(&live_primary)).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(source.is_file());
    }

    #[cfg(unix)]
    #[test]
    fn external_fixture_rejects_symlink() {
        use std::os::unix::fs::symlink;

        let (dir, _paths) = test_paths("external-fixture-symlink");
        let target = dir.0.join("progress.manual-before-v2-target.json");
        let source = dir.0.join("progress.manual-before-v2-link.json");
        std::fs::write(&target, b"fixture").unwrap();
        symlink(&target, &source).unwrap();

        let error = validate_external_v1_fixture(&source, None).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn external_fixture_rejects_directory() {
        let (dir, _paths) = test_paths("external-fixture-directory");
        let source = dir.0.join("progress.manual-before-v2-directory.json");
        std::fs::create_dir(&source).unwrap();

        let error = validate_external_v1_fixture(&source, None).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    }

    #[test]
    fn external_fixture_accepts_regular_manual_copy() {
        let (dir, _paths) = test_paths("external-fixture-manual-copy");
        let source = dir.0.join("progress.manual-before-v2-20260722-195941.json");
        std::fs::write(&source, b"fixture").unwrap();

        validate_external_v1_fixture(&source, None).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn external_fixture_rejects_live_primary_hard_link_without_reading_it() {
        use std::os::unix::fs::MetadataExt;

        let (dir, _paths) = test_paths("external-fixture-hard-link");
        let live_primary = dir.0.join("progress.json");
        let source = dir.0.join("progress.manual-before-v2-20260722-195941.json");
        std::fs::write(&live_primary, b"fixture content must remain unread").unwrap();
        std::fs::hard_link(&live_primary, &source).unwrap();

        let error = validate_external_v1_fixture(&source, Some(&live_primary)).unwrap_err();

        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(live_primary.is_file());
        assert!(source.is_file());
        let live_metadata = std::fs::metadata(&live_primary).unwrap();
        let source_metadata = std::fs::metadata(&source).unwrap();
        assert_eq!(source_metadata.dev(), live_metadata.dev());
        assert_eq!(source_metadata.ino(), live_metadata.ino());
    }

    #[test]
    #[ignore = "requires MANA_PROGRESS_V1_FIXTURE"]
    fn external_v1_fixture_preserves_all_progress_invariants() {
        let source = std::path::PathBuf::from(
            std::env::var_os("MANA_PROGRESS_V1_FIXTURE").expect("fixture path"),
        );
        let live_primary = super::live_primary_path();
        let source = validate_external_v1_fixture(&source, live_primary.as_deref()).unwrap();
        let bytes = std::fs::read(source).unwrap();
        assert_eq!(
            decode_state(&bytes).unwrap(),
            DecodedState::NeedsOutputRebuild { source_schema: 1 }
        );
        let (_dir, paths) = test_paths("external-migration");
        std::fs::write(&paths.primary, bytes).unwrap();
        let loaded = load_state(&paths).unwrap();
        assert_eq!(loaded.state, ProgressState::default());
        assert!(loaded.needs_output_rebuild);
        assert!(paths.pre_migration_v1.exists());
    }
}
