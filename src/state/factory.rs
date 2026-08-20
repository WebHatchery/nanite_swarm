//! Factory depth and operating focus.
//!
//! The map already contains the pieces of a multi-stage factory, but the
//! player had to infer its depth from individual building cards. This module
//! gives that growth a compact vocabulary and a deliberate operating profile.

use crate::engine::BuildingType;
use serde::{Deserialize, Serialize};

use super::game_state::PlanetState;

/// Which production deck receives the swarm's spare scheduling attention.
///
/// Focus is intentionally a trade: a selected deck works faster, while its
/// powered machines draw a little more energy. Balanced remains the safe
/// default for a newly landed world and for older saves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FactoryFocus {
    #[default]
    Balanced,
    Extraction,
    Refining,
    Assembly,
}

impl FactoryFocus {
    pub const ALL: [Self; 4] = [
        Self::Balanced,
        Self::Extraction,
        Self::Refining,
        Self::Assembly,
    ];

    pub fn next(self) -> Self {
        match self {
            Self::Balanced => Self::Extraction,
            Self::Extraction => Self::Refining,
            Self::Refining => Self::Assembly,
            Self::Assembly => Self::Balanced,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Balanced => "BALANCED",
            Self::Extraction => "EXTRACTION",
            Self::Refining => "REFINING",
            Self::Assembly => "ASSEMBLY",
        }
    }

    pub fn short_label(self) -> &'static str {
        match self {
            Self::Balanced => "BAL",
            Self::Extraction => "ORE",
            Self::Refining => "ALLOY",
            Self::Assembly => "PARTS",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Balanced => "All decks share the grid evenly.",
            Self::Extraction => "Drills cut 25% faster; drill power draw rises 15%.",
            Self::Refining => "Smelters refine 25% faster; smelter draw rises 15%.",
            Self::Assembly => "Servers and assemblers work 25% faster; draw rises 15%.",
        }
    }

    pub fn affects(self, building_type: BuildingType) -> bool {
        match self {
            Self::Balanced => false,
            Self::Extraction => building_type == BuildingType::Drill,
            Self::Refining => building_type == BuildingType::Smelter,
            Self::Assembly => {
                matches!(
                    building_type,
                    BuildingType::ServerBank | BuildingType::Assembler
                )
            }
        }
    }

    pub fn work_multiplier(self, building_type: BuildingType) -> f32 {
        if self.affects(building_type) {
            1.25
        } else {
            1.0
        }
    }
}

impl PlanetState {
    /// The deepest physical deck the current base has opened.
    ///
    /// This is derived from what is standing rather than saved as a second
    /// progression counter, so demolishing an optional deck never corrupts a
    /// save and old worlds receive the feature automatically.
    pub fn factory_depth(&self) -> u8 {
        if !self.grid.find_buildings(BuildingType::Smelter).is_empty()
            || !self
                .grid
                .find_buildings(BuildingType::ServerBank)
                .is_empty()
        {
            if !self.grid.find_buildings(BuildingType::Assembler).is_empty() {
                if !self
                    .grid
                    .find_buildings(BuildingType::MassDriver)
                    .is_empty()
                    || self.core_stage >= 3
                {
                    return 3;
                }
                return 2;
            }
            return 1;
        }
        0
    }

    pub fn factory_depth_label(&self) -> &'static str {
        match self.factory_depth() {
            0 => "SURFACE",
            1 => "FOUNDRY",
            2 => "ASSEMBLY",
            _ => "ORBITAL",
        }
    }

    /// Returns the normalized progress toward the next deck and its action.
    pub fn factory_depth_progress(&self) -> (f32, &'static str) {
        match self.factory_depth() {
            0 => (
                self.grid.find_buildings(BuildingType::Smelter).len() as f32 / 1.0,
                "PLACE SMELTER",
            ),
            1 => (
                self.grid.find_buildings(BuildingType::Assembler).len() as f32 / 1.0,
                "PLACE ASSEMBLER",
            ),
            2 => (
                self.grid.find_buildings(BuildingType::MassDriver).len() as f32 / 1.0,
                "PLACE MASS DRIVER",
            ),
            _ => (1.0, "ALL DECKS ONLINE"),
        }
    }

    pub fn factory_focus_multiplier(&self, building_type: BuildingType) -> f32 {
        self.factory_focus.work_multiplier(building_type)
    }

    /// Extra draw from a focused production deck. This is kept separate from
    /// the grid's connection calculation: focus cannot make an unpowered
    /// building appear powered, it only changes the balance once connected.
    pub fn factory_focus_power_tax(&self) -> f32 {
        if self.factory_focus == FactoryFocus::Balanced {
            return 0.0;
        }
        self.grid
            .iter_tiles()
            .filter_map(|(_, tile)| tile.building.as_ref())
            .filter(|building| {
                building.powered && self.factory_focus.affects(building.building_type)
            })
            .map(|building| {
                let def = crate::data::game_data().building(building.building_type.id());
                def.power_consumption
                    * 0.15
                    * building.dust_power_consumption_multiplier()
                    * building.power_demand_multiplier()
            })
            .sum()
    }

    pub fn set_factory_focus(&mut self, focus: FactoryFocus) {
        if self.factory_focus == focus {
            return;
        }
        self.factory_focus = focus;
        self.grid.update_power_grid();
        self.power_balance = self.net_power();
        self.notifications
            .info(format!("Factory focus: {}", focus.label()));
    }

    pub fn cycle_factory_focus(&mut self) {
        self.set_factory_focus(self.factory_focus.next());
    }
}

#[cfg(test)]
mod tests;
