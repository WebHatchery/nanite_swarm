//! Short-term directives for pacing and rewards.

use crate::engine::BuildingType;
use crate::state::PlanetState;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DirectiveKind {
    PowerSurplus,
    DrillCount,
    ServerBanks,
    HarvestForest,
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
}

impl Directive {
    pub fn new(kind: DirectiveKind, target: i32, duration: f32, reward_data: f32) -> Self {
        let description = match kind {
            DirectiveKind::PowerSurplus => format!("Sustain +{} power for {}s", target, target),
            DirectiveKind::DrillCount => format!("Operate {} drills", target),
            DirectiveKind::ServerBanks => format!("Run {} server banks", target),
            DirectiveKind::HarvestForest => format!("Harvest {} forests", target),
        };
        Self {
            kind,
            description,
            target,
            progress: 0,
            duration,
            reward_data,
            completed: false,
            sustained: 0.0,
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
                self.sustained = (self.sustained + step).clamp(0.0, self.target as f32);
                self.progress = self.sustained as i32;
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
        }
        if self.progress >= self.target {
            self.completed = true;
        }
    }
}

pub fn pick_directive(tier: i32) -> Directive {
    match tier % 4 {
        0 => Directive::new(
            DirectiveKind::PowerSurplus,
            20 + tier * 5,
            600.0,
            20.0 + tier as f32 * 5.0,
        ),
        1 => Directive::new(
            DirectiveKind::DrillCount,
            3 + tier,
            600.0,
            15.0 + tier as f32 * 4.0,
        ),
        2 => Directive::new(
            DirectiveKind::ServerBanks,
            2 + tier / 2,
            600.0,
            20.0 + tier as f32 * 5.0,
        ),
        _ => Directive::new(
            DirectiveKind::HarvestForest,
            2 + tier / 2,
            600.0,
            10.0 + tier as f32 * 3.0,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;
    use crate::engine::{BuildingType, GridPos};
    use crate::state::PlanetState;

    #[test]
    fn pick_directive_cycles_through_all_four_kinds() {
        assert_eq!(pick_directive(0).kind, DirectiveKind::PowerSurplus);
        assert_eq!(pick_directive(1).kind, DirectiveKind::DrillCount);
        assert_eq!(pick_directive(2).kind, DirectiveKind::ServerBanks);
        assert_eq!(pick_directive(3).kind, DirectiveKind::HarvestForest);
        // Wraps around.
        assert_eq!(pick_directive(4).kind, DirectiveKind::PowerSurplus);
    }

    #[test]
    fn power_surplus_directive_progresses_only_while_target_is_met() {
        let mut directive = Directive::new(DirectiveKind::PowerSurplus, 5, 100.0, 10.0);
        let mut state = PlanetState::new("Test", 8, 8, 1, GameConfig::default());
        state.power_balance = 10.0;

        directive.update(&state, 1.0);
        assert_eq!(directive.progress, 1);

        state.power_balance = -5.0;
        directive.update(&state, 1.0);
        assert_eq!(directive.progress, 0);
    }

    #[test]
    fn drill_count_directive_tracks_placed_drills_and_completes() {
        let mut directive = Directive::new(DirectiveKind::DrillCount, 1, 100.0, 10.0);
        let mut state = PlanetState::new("Test", 8, 8, 1, GameConfig::default());
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);

        directive.update(&state, 0.0);
        assert_eq!(directive.progress, 1);
        assert!(directive.completed);
    }

    #[test]
    fn completed_directive_no_longer_updates() {
        let mut directive = Directive::new(DirectiveKind::PowerSurplus, 1, 100.0, 10.0);
        directive.completed = true;
        directive.progress = 1;
        let mut state = PlanetState::new("Test", 8, 8, 1, GameConfig::default());
        state.power_balance = -50.0;

        directive.update(&state, 1.0);
        // A completed directive is frozen: progress doesn't regress further.
        assert_eq!(directive.progress, 1);
    }

    #[test]
    fn duration_counts_down_but_never_below_zero() {
        let mut directive = Directive::new(DirectiveKind::HarvestForest, 5, 2.0, 10.0);
        let state = PlanetState::new("Test", 8, 8, 1, GameConfig::default());
        directive.update(&state, 1.5);
        assert_eq!(directive.duration, 0.5);
        directive.update(&state, 5.0);
        assert_eq!(directive.duration, 0.0);
    }
}
