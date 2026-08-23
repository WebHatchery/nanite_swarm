//! Terrain types affecting buildability and harvesting

use crate::data::GameData;
use serde::{Deserialize, Serialize};

/// Terrain types that affect gameplay
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TerrainType {
    #[default]
    Empty, // Buildable ground
    Mountain, // Can harvest for iron or place wind turbine
    Forest,   // Can harvest for biomass or keep as pollution buffer
    Water,    // Cannot build, may provide cooling
    Rough,    // Difficult to build on (result of harvesting)
    Void,     // Unbuildable gap (volcanic terrain)
}

impl TerrainType {
    pub fn id(&self) -> &'static str {
        match self {
            TerrainType::Empty => "empty",
            TerrainType::Mountain => "mountain",
            TerrainType::Forest => "forest",
            TerrainType::Water => "water",
            TerrainType::Rough => "rough",
            TerrainType::Void => "void",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "empty" => Some(TerrainType::Empty),
            "mountain" => Some(TerrainType::Mountain),
            "forest" => Some(TerrainType::Forest),
            "water" => Some(TerrainType::Water),
            "rough" => Some(TerrainType::Rough),
            "void" => Some(TerrainType::Void),
            _ => None,
        }
    }

    /// Whether buildings can be placed on this terrain
    pub fn is_buildable(&self, data: &GameData) -> bool {
        data.terrain(self.id()).buildable
    }

    /// Whether this terrain can be harvested
    pub fn is_harvestable(&self, data: &GameData) -> bool {
        data.terrain(self.id()).harvestable
    }

    /// Get harvest rewards (minerals, biomass)
    pub fn harvest_rewards(&self, data: &GameData) -> (f32, f32) {
        let def = data.terrain(self.id());
        (def.harvest_rewards.minerals, def.harvest_rewards.biomass)
    }

    /// Get terrain after harvesting
    pub fn harvested(&self, data: &GameData) -> TerrainType {
        let def = data.terrain(self.id());
        TerrainType::from_id(&def.harvested_to).unwrap_or(*self)
    }

    /// Get preservation bonus description
    pub fn preservation_bonus<'a>(&self, data: &'a GameData) -> Option<&'a str> {
        data.terrain(self.id()).preservation_bonus.as_deref()
    }

    /// Display name
    pub fn name<'a>(&self, data: &'a GameData) -> &'a str {
        data.terrain(self.id()).name.as_str()
    }
}

#[cfg(test)]
mod tests;
