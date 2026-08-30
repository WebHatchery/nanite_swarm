use crate::data::GameData;

/// Validate cross-file references once, with the source asset and identifier
/// in every failure. Runtime lookups remain compact because a malformed
/// package cannot reach them.
pub fn validate_game_data(data: &GameData) -> Result<(), String> {
    crate::engine::validate_game_data(data)?;
    crate::directives::validate_game_data(data)?;
    crate::state::validate_game_data(data)?;
    crate::data::validate_game_data(data)?;
    Ok(())
}
