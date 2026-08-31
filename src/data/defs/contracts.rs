//! Tutorial, directive, achievement, and Core-stage data contracts.

use serde::Deserialize;

use super::research::ModifierDef;

/// What finishes a tutorial step. `kind` is checked at load, so a typo is a
/// startup failure rather than a step nobody can ever complete.
#[derive(Debug, Clone, Deserialize)]
pub struct TutorialGoalDef {
    pub kind: String,
    pub target: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TutorialStepDef {
    pub id: String,
    pub title: String,
    pub instruction: String,
    pub goal: TutorialGoalDef,
    /// Optional next-action copy shown when this step completes the lesson.
    #[serde(default)]
    pub completion_message: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TutorialDataFile {
    pub steps: Vec<TutorialStepDef>,
}

/// One kind of standing order the swarm can be given, and how it grows as the
/// campaign goes on. `text` may use `{target}` and `{hold}`.
#[derive(Debug, Clone, Deserialize)]
pub struct DirectiveDef {
    pub id: String,
    pub kind: String,
    pub text: String,
    pub base_target: f32,
    #[serde(default)]
    pub target_per_tier: f32,
    /// Only read by directives that ask for something to be held: how long.
    /// It used to double as the target, so raising the power threshold also
    /// made the player hold it for longer.
    #[serde(default)]
    pub hold_seconds: f32,
    pub base_reward: f32,
    #[serde(default)]
    pub reward_per_tier: f32,
}

/// What has to be true for an achievement, in the same declared shape the
/// directives use: something the simulation knows how to measure, and how much
/// of it. `manual` is for the ones code announces at the moment they happen.
#[derive(Debug, Clone, Deserialize)]
pub struct AchievementConditionDef {
    pub kind: String,
    #[serde(default = "one")]
    pub target: f32,
}

fn one() -> f32 {
    1.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct AchievementDef {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AchievementConditionDef,
}

/// One step of the Evolving Core: what it takes to get there and what it does
/// once it is standing.
#[derive(Debug, Clone, Deserialize)]
pub struct CoreStageDef {
    pub id: String,
    pub name: String,
    pub description: String,
    /// Every one of these has to be met at once. Empty for the stage a world
    /// lands with.
    #[serde(default)]
    pub requires: Vec<AchievementConditionDef>,
    #[serde(default)]
    pub modifiers: Vec<ModifierDef>,
    #[serde(default)]
    pub sprite: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoreStageDataFile {
    pub stages: Vec<CoreStageDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AchievementDataFile {
    pub achievements: Vec<AchievementDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DirectiveDataFile {
    /// How long a directive stands before the next one replaces it.
    pub rotation_seconds: f32,
    pub directives: Vec<DirectiveDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlanetDataFile {
    pub planets: Vec<super::planets::PlanetDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BuildingDataFile {
    pub buildings: Vec<super::buildings::BuildingDef>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TerrainDataFile {
    pub terrain: Vec<super::buildings::TerrainDef>,
}
