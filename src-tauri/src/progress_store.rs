use crate::progress::{ProgressState, TallyState, TIERS};

pub const SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProgressEnvelope {
    pub schema_version: u32,
    pub state: ProgressState,
}

#[derive(serde::Deserialize)]
#[serde(untagged)]
enum ProgressDocument {
    Versioned(ProgressEnvelope),
    Legacy(LegacyProgressState),
}

fn legacy_initialized() -> bool {
    true
}

#[derive(serde::Deserialize)]
struct LegacyProgressState {
    rank: usize,
    prestige: u32,
    prestige_token_floor: u64,
    #[serde(default = "legacy_initialized")]
    initialized: bool,
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
    let document = serde_json::from_slice(bytes).map_err(invalid_data)?;
    match document {
        ProgressDocument::Versioned(envelope) => {
            if envelope.schema_version != SCHEMA_VERSION {
                return Err(invalid_data("unsupported progress schema version"));
            }
            Ok((validate_rank(envelope.state)?, false))
        }
        ProgressDocument::Legacy(legacy) => {
            let state = ProgressState {
                rank: legacy.rank,
                prestige: legacy.prestige,
                prestige_token_floor: legacy.prestige_token_floor,
                initialized: legacy.initialized,
                tally: legacy.tally,
            };
            Ok((validate_rank(state)?, true))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{decode_state, encode_state, ProgressEnvelope, SCHEMA_VERSION};
    use crate::progress::ProgressState;

    fn fixture_state() -> ProgressState {
        serde_json::from_slice(include_bytes!("../tests/fixtures/progress_v1.json")).unwrap()
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
