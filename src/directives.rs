//! Standing orders: what the swarm is currently being asked to do.
//!
//! The set lives in `assets/directives.json` — wording, targets, how each grows
//! with the tier, and what it pays. Only the *kinds* are code, because a kind
//! is something the simulation has to know how to measure.

use crate::data::DirectiveDef;
use crate::engine::BuildingType;
use crate::state::PlanetState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DirectiveKind {
    PowerSurplus,
    DrillCount,
    ServerBanks,
    HarvestForest,
    StructureCount,
    MineralStock,
    ResearchCount,
}

impl DirectiveKind {
    pub fn id(self) -> &'static str {
        match self {
            DirectiveKind::PowerSurplus => "power_surplus",
            DirectiveKind::DrillCount => "drill_count",
            DirectiveKind::ServerBanks => "server_banks",
            DirectiveKind::HarvestForest => "harvest_forest",
            DirectiveKind::StructureCount => "structure_count",
            DirectiveKind::MineralStock => "mineral_stock",
            DirectiveKind::ResearchCount => "research_count",
        }
    }

    pub const ALL: [DirectiveKind; 7] = [
        DirectiveKind::PowerSurplus,
        DirectiveKind::DrillCount,
        DirectiveKind::ServerBanks,
        DirectiveKind::HarvestForest,
        DirectiveKind::StructureCount,
        DirectiveKind::MineralStock,
        DirectiveKind::ResearchCount,
    ];

    pub fn from_id(id: &str) -> Option<Self> {
        DirectiveKind::ALL.into_iter().find(|kind| kind.id() == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub kind: DirectiveKind,
    pub description: String,
    pub target: i32,
    pub progress: i32,
    pub duration: f32,
    pub reward_data: f32,
    pub completed: bool,
    /// Seconds of sustained surplus, kept as a float so a fractional
    /// simulation step still counts toward the integer `progress`.
    #[serde(default)]
    pub sustained: f32,
    /// How long a "hold this" directive wants it held for. Separate from the
    /// target so the threshold and the endurance can be tuned apart.
    #[serde(default = "default_hold_seconds")]
    pub hold_seconds: f32,
}

fn default_hold_seconds() -> f32 {
    20.0
}

impl Directive {
    /// Build the tier-th instance of a declared directive.
    pub fn from_def(def: &DirectiveDef, tier: i32, duration: f32) -> Self {
        let target = (def.base_target + def.target_per_tier * tier as f32)
            .round()
            .max(1.0) as i32;
        let hold_seconds = if def.hold_seconds > 0.0 {
            def.hold_seconds
        } else {
            default_hold_seconds()
        };
        let description = def
            .text
            .replace("{target}", &target.to_string())
            .replace("{hold}", &format!("{:.0}", hold_seconds));
        Self {
            kind: DirectiveKind::from_id(&def.kind).unwrap_or(DirectiveKind::DrillCount),
            description,
            target,
            progress: 0,
            duration,
            reward_data: def.base_reward + def.reward_per_tier * tier as f32,
            completed: false,
            sustained: 0.0,
            hold_seconds,
        }
    }

    pub fn update(&mut self, state: &PlanetState, delta: f32) {
        if self.completed {
            return;
        }
        self.duration = (self.duration - delta).max(0.0);
        match self.kind {
            DirectiveKind::PowerSurplus => {
                // One second of surplus is one point of progress, and losing
                // the surplus bleeds it back at the same rate. Counting in
                // whole frames instead made this never move at 60fps and run
                // deeply negative whenever power dipped.
                let step = if state.power_balance >= self.target as f32 {
                    delta
                } else {
                    -delta
                };
                self.sustained = (self.sustained + step).clamp(0.0, self.hold_seconds);
                self.progress = self.sustained as i32;
                if self.sustained >= self.hold_seconds {
                    self.completed = true;
                }
                return;
            }
            DirectiveKind::DrillCount => {
                let count = state.grid.find_buildings(BuildingType::Drill).len() as i32;
                self.progress = count.min(self.target);
            }
            DirectiveKind::ServerBanks => {
                let count = state.grid.find_buildings(BuildingType::ServerBank).len() as i32;
                self.progress = count.min(self.target);
            }
            DirectiveKind::HarvestForest => {
                self.progress = state.forest_harvested_count.min(self.target);
            }
            DirectiveKind::StructureCount => {
                self.progress = (state.grid.total_buildings() as i32).min(self.target);
            }
            DirectiveKind::MineralStock => {
                self.progress = (state.resources.minerals as i32).min(self.target);
            }
            DirectiveKind::ResearchCount => {
                self.progress = (state.research.unlocked_techs.len() as i32).min(self.target);
            }
        }
        if self.progress >= self.target {
            self.completed = true;
        }
    }
}

#[cfg(test)]
impl Directive {
    /// A directive built straight from numbers, for tests about how one
    /// behaves rather than about how the set is declared.
    fn for_test(kind: DirectiveKind, target: i32, duration: f32) -> Self {
        Self {
            kind,
            description: String::new(),
            target,
            progress: 0,
            duration,
            reward_data: 10.0,
            completed: false,
            sustained: 0.0,
            hold_seconds: target as f32,
        }
    }
}

/// How long a directive stands before the next one takes its place.
pub fn rotation_seconds() -> f32 {
    crate::data::game_data()
        .directives
        .rotation_seconds
        .max(1.0)
}

/// The directive for this tier: the declared set, in order, cycling.
///
/// The set is data, so a new standing order that reuses an existing kind is a
/// line of JSON rather than another arm of a `match tier % 4`.
pub fn pick_directive(tier: i32) -> Directive {
    let data = crate::data::game_data();
    let set = &data.directives.directives;
    let duration = rotation_seconds();
    if set.is_empty() {
        // Nothing declared: ask for a drill and say so plainly.
        return Directive {
            kind: DirectiveKind::DrillCount,
            description: "Operate 3 drills".to_string(),
            target: 3,
            progress: 0,
            duration,
            reward_data: 15.0,
            completed: false,
            sustained: 0.0,
            hold_seconds: default_hold_seconds(),
        };
    }
    let index = tier.rem_euclid(set.len() as i32) as usize;
    let cycle = tier.div_euclid(set.len() as i32).max(0);
    Directive::from_def(&set[index], cycle, duration)
}

#[cfg(test)]
mod tests;
