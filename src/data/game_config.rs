//! Game balance values and settings

use serde::{Deserialize, Serialize};

/// Core game configuration loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub grid: GridConfig,
    pub resources: ResourceConfig,
    pub buildings: BuildingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub initial_width: u32,
    pub initial_height: u32,
    pub tile_size: f32,
    pub max_width: u32,
    pub max_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub starting_energy: f32,
    pub starting_minerals: f32,
    pub drone_carry_capacity: f32,
    pub drone_speed: f32,
    pub max_energy: f32,
    pub base_mineral_cap: f32,
    pub storage_bonus: f32,
    pub storage_tech_bonus: f32,
    pub core_data_rate: f32,
    pub forest_biomass: f32,
    pub biomass_power_output: f32,
    pub biomass_consumption_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingConfig {
    pub core_power_consumption: f32,
    pub drill_output_rate: f32,
    pub conduit_throughput: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            grid: GridConfig {
                initial_width: 16,
                initial_height: 16,
                tile_size: 32.0,
                max_width: 64,
                max_height: 64,
            },
            resources: ResourceConfig {
                starting_energy: 50.0,
                starting_minerals: 50.0,
                drone_carry_capacity: 10.0,
                // Tiles per second. Keep in step with assets/game_config.json.
                drone_speed: 8.0,
                max_energy: 50.0,
                base_mineral_cap: 100.0,
                storage_bonus: 100.0,
                storage_tech_bonus: 50.0,
                core_data_rate: 0.25,
                forest_biomass: 60.0,
                biomass_power_output: 8.0,
                biomass_consumption_rate: 1.0,
            },
            buildings: BuildingConfig {
                core_power_consumption: 5.0,
                drill_output_rate: 2.0,
                conduit_throughput: 10.0,
            },
        }
    }
}
