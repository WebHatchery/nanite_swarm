//! Research progress attached to a planetary state.

use crate::data::GameData;
use crate::engine::{BuildingType, Stats};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::PlanetState;

/// Research node status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProgress {
    pub unlocked_techs: Vec<String>,
    pub current_research: Option<String>,
    pub research_progress: f32,
    /// Partial work on nodes not currently selected for research.
    #[serde(default)]
    pub parked_progress: HashMap<String, f32>,
}

impl Default for ResearchProgress {
    fn default() -> Self {
        Self {
            unlocked_techs: Vec::new(),
            current_research: None,
            research_progress: 0.0,
            parked_progress: HashMap::new(),
        }
    }
}

impl ResearchProgress {
    pub fn from_data(data: &GameData) -> Self {
        Self {
            unlocked_techs: data.research.starting_unlocked.clone(),
            ..Self::default()
        }
    }
}

impl PlanetState {
    /// Take the campaign's research and rebuild everything it drives here:
    /// the stat sheet and which buildings this world will accept.
    ///
    /// Every world needs its own answer even when the research is shared,
    /// because a world can refuse a building the swarm has researched.
    pub fn adopt_research(&mut self, research: &ResearchProgress) {
        self.research = research.clone();
        self.refresh_stats();
        self.refresh_building_unlocks();
    }

    /// Open up every building whose prerequisite research is done.
    pub fn refresh_building_unlocks(&mut self) {
        // This is deliberately a rebuild, not an additive sync. Removing a
        // research unlock in a migrated/debug campaign must not leave a stale
        // building selectable on every world forever.
        self.unlocked_buildings.clear();
        let building_unlocks: Vec<_> = self
            .data()
            .buildings
            .iter()
            .map(|def| (def.id.clone(), def.start_unlocked, def.unlocked_by.clone()))
            .collect();
        for (id, start_unlocked, unlocked_by) in building_unlocks {
            let Some(building_type) = BuildingType::from_id(&id) else {
                continue;
            };
            let unlocked = start_unlocked
                || unlocked_by
                    .as_deref()
                    .is_some_and(|tech| self.research.unlocked_techs.iter().any(|id| id == tech));
            if unlocked {
                self.unlock_building(building_type);
            }
        }
    }

    /// Rebuild the stat sheet from what research has unlocked, and push the
    /// values that live outside it into place. Call this after anything that
    /// changes `research.unlocked_techs`.
    pub fn refresh_stats(&mut self) {
        let mut stats = Stats::from_unlocked(self.data(), &self.research.unlocked_techs);
        // A Seed Ship stage that is standing works for the world it stands on,
        // until the ship takes it away.
        for stage in self.seed_ship.standing_stages() {
            stats.add_declared(&stage.modifiers);
        }
        // Everything the Core has grown into, which unlike the ship's stages
        // is never taken away again.
        for stage in self.core_stages_reached() {
            stats.add_declared(&stage.modifiers);
        }
        self.stats = stats;

        let capacity = self.stats.apply(
            crate::engine::StatId::DroneCapacity,
            self.config.resources.drone_carry_capacity,
        );
        self.drones.drone_capacity = capacity;
        for drone in self.drones.drones_mut() {
            drone.capacity = capacity;
        }

        // Both of these live outside the sheet - one on the drone manager, one
        // on the grid - so they have to be pushed rather than read.
        self.drones.drone_speed = self.stats.apply(
            crate::engine::StatId::DroneSpeed,
            self.config.resources.drone_speed,
        );
        self.grid.repeater_range = self
            .stats
            .apply(
                crate::engine::StatId::RepeaterRange,
                self.config.buildings.repeater_range as f32,
            )
            .max(1.0)
            .round() as u32;
        self.refresh_power_grid();
    }
}
