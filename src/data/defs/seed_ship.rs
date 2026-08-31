//! Seed Ship construction and launch sequence records.

use serde::Deserialize;

use super::research::ModifierDef;

/// What one Seed Ship stage swallows before it is done.
#[derive(Debug, Clone, Copy, Default, Deserialize)]
pub struct SeedShipCost {
    pub minerals: f32,
    pub data: f32,
    pub biomass: f32,
    #[serde(default)]
    pub alloy: f32,
    #[serde(default)]
    pub components: f32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedShipStageDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cost: SeedShipCost,
    /// Research that has to be done before this stage can be worked on. The
    /// tree and the megastructure have to meet somewhere.
    #[serde(default)]
    pub requires: Option<String>,
    /// One line on what standing this stage up does for the world it is being
    /// built on. Empty on the last stage, which does nothing but leave.
    #[serde(default)]
    pub boon: String,
    /// What that boon actually is, in the same declared form research uses.
    #[serde(default)]
    pub modifiers: Vec<ModifierDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SeedShipData {
    /// Ceiling on how fast the yard can swallow each resource, per second.
    pub intake_per_second: SeedShipCost,
    pub stages: Vec<SeedShipStageDef>,
    #[serde(default)]
    pub launch: LaunchSequenceDef,
}

/// The beats of a launch, and what the swarm says over each of them.
///
/// All four durations default to zero, so a missing or unreadable block gives
/// a sequence that is over before it starts rather than one that hangs.
/// `{origin}` in any line is replaced with the name of the world being spent.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LaunchSequenceDef {
    pub countdown_seconds: f32,
    pub ascent_seconds: f32,
    pub transit_seconds: f32,
    pub arrival_seconds: f32,
    pub countdown_line: String,
    pub ascent_line: String,
    pub transit_line: String,
    pub skip_hint: String,
}
