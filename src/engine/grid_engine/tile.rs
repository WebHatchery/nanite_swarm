//! A single grid tile: terrain, occupying building, and permanent scars

use super::building::Building;
use super::terrain::TerrainType;
use serde::{Deserialize, Serialize};

/// A single tile on the grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tile {
    pub terrain: TerrainType,
    pub building: Option<Building>,
    pub revealed: bool, // For fog of war / expansion
    #[serde(default)]
    pub filter: bool, // Forest filter tile for dust reduction
    #[serde(default)]
    pub mountain_harvested: bool, // Permanent scar for turbine bonuses
    #[serde(default)]
    pub forest_cleared: bool, // Permanent pollution penalty
    #[serde(default)]
    pub biomass_amount: f32,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            terrain: TerrainType::Empty,
            building: None,
            revealed: false,
            filter: false,
            mountain_harvested: false,
            forest_cleared: false,
            biomass_amount: 0.0,
        }
    }
}
