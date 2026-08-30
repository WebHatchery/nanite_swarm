//! One answer to the question "why will this tile not take that building?".
//!
//! The map preview and the touch action used to ask slightly different
//! questions.  Keeping the reason on the state action layer makes a red
//! preview mean the same thing as the warning after a tap, and leaves the
//! renderer free of a second copy of the placement rules.

use crate::engine::{BuildingType, GridPos};

use super::super::game_state::PlanetState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementBlockReason {
    OutOfBounds,
    Unrevealed,
    Filter,
    Occupied,
    Banned,
    Research,
    Terrain,
    Constraint,
    Resources,
}

impl PlacementBlockReason {
    /// A compact label suitable for a map preview or a small inspector row.
    pub fn preview_label(self) -> &'static str {
        match self {
            Self::OutOfBounds => "OUTSIDE GRID",
            Self::Unrevealed => "SURVEY REQUIRED",
            Self::Filter => "FILTER TILE",
            Self::Occupied => "OCCUPIED",
            Self::Banned => "WORLD BAN",
            Self::Research => "RESEARCH REQUIRED",
            Self::Terrain => "WRONG TERRAIN",
            Self::Constraint => "WORLD RULE",
            Self::Resources => "RESOURCES LOW",
        }
    }

    /// The actionable warning shown after a primary map tap.
    pub fn message(self, state: &PlanetState, building_type: BuildingType) -> String {
        match self {
            Self::OutOfBounds => "That tile is outside the surveyed grid".to_string(),
            Self::Unrevealed => "Survey this tile before building".to_string(),
            Self::Filter => "This tile is already a dust filter".to_string(),
            Self::Occupied => "This tile is already occupied".to_string(),
            Self::Banned => "This world bans that building".to_string(),
            Self::Research => "Research this building before placing it".to_string(),
            Self::Terrain => terrain_message(building_type).to_string(),
            Self::Constraint => {
                let (_, details) = state.planet_constraint_status();
                if details.is_empty() {
                    "Meet this world's construction requirements first".to_string()
                } else {
                    format!("World rule: {details}")
                }
            }
            Self::Resources => {
                let (mineral_cost, energy_cost) = building_type.cost(state.data());
                format!(
                    "Need M {:.0} and E {:.0} to build",
                    mineral_cost, energy_cost
                )
            }
        }
    }
}

fn terrain_message(building_type: BuildingType) -> &'static str {
    match building_type {
        BuildingType::Bridge => "Bridges need a water or void gap",
        BuildingType::BiomassHarvester => "Biomass harvesters need a forest",
        BuildingType::WindTurbine => "Wind turbines need open ground or a mountain",
        BuildingType::Conduit => "Conduits need buildable ground",
        _ => "That terrain cannot hold this building",
    }
}

impl PlanetState {
    /// Return the first reason a selected building cannot be placed.
    ///
    /// The order follows what a player can fix: find a tile, reveal it, clear
    /// the tile, unlock the building, satisfy the world's rule, then pay for
    /// it.  It also preserves the most useful reason when several conditions
    /// are false at once.
    pub fn placement_block_reason(
        &self,
        pos: GridPos,
        building_type: BuildingType,
    ) -> Option<PlacementBlockReason> {
        let Some(tile) = self.grid.get(pos) else {
            return Some(PlacementBlockReason::OutOfBounds);
        };
        if !tile.revealed {
            return Some(PlacementBlockReason::Unrevealed);
        }
        if tile.filter {
            return Some(PlacementBlockReason::Filter);
        }
        if tile.building.is_some() {
            return Some(PlacementBlockReason::Occupied);
        }
        if self.is_building_banned(building_type) {
            return Some(PlacementBlockReason::Banned);
        }
        if !self.is_building_researched(building_type) {
            return Some(PlacementBlockReason::Research);
        }
        if !self.grid.can_place_building(pos, building_type) {
            return Some(PlacementBlockReason::Terrain);
        }
        if !self.constraints_allow_building(building_type) {
            return Some(PlacementBlockReason::Constraint);
        }
        let (mineral_cost, energy_cost) = building_type.cost(self.data());
        if !self.resources.can_afford(mineral_cost, energy_cost) {
            return Some(PlacementBlockReason::Resources);
        }
        None
    }
}

#[cfg(test)]
mod tests;
