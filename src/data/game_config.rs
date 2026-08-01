//! Game balance values and settings

use serde::{Deserialize, Serialize};

/// Core game configuration loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    pub grid: GridConfig,
    pub resources: ResourceConfig,
    pub buildings: BuildingConfig,
    #[serde(default = "CollapseConfig::default")]
    pub collapse: CollapseConfig,
    #[serde(default = "OreConfig::default")]
    pub ore: OreConfig,
    #[serde(default = "MassDriverConfig::default")]
    pub mass_driver: MassDriverConfig,
    #[serde(default = "UpkeepConfig::default")]
    pub upkeep: UpkeepConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub initial_width: u32,
    pub initial_height: u32,
    pub tile_size: f32,
    pub max_width: u32,
    pub max_height: u32,
    /// The least share of a world that has to be buildable ground connected to
    /// the landing site. Terrain is laid in regions now, so without a floor a
    /// chasm can wall the Core in before the swarm can research Bridges.
    #[serde(default = "default_min_start_region")]
    pub min_start_region: f32,
}

fn default_min_start_region() -> f32 {
    0.3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceConfig {
    pub starting_energy: f32,
    pub starting_minerals: f32,
    pub drone_carry_capacity: f32,
    pub drones_per_drill: f32,
    pub drone_speed: f32,
    pub max_energy: f32,
    pub base_mineral_cap: f32,
    pub storage_bonus: f32,
    pub core_data_rate: f32,
    pub server_data_rate: f32,
    /// Data spent per second on whatever is being researched.
    pub research_rate: f32,
    pub forest_biomass: f32,
    pub biomass_power_output: f32,
    pub biomass_consumption_rate: f32,
}

/// What a power collapse costs, and how that grows with the size of the swarm
/// that collapsed.
///
/// A flat penalty is wrong at both ends: twenty seconds off a base of four
/// buildings is brutal, and the same twenty seconds off a base of sixty is a
/// pause. Everything here is interpolated between the small-swarm figure and
/// the full-scale one.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollapseConfig {
    /// Seconds of negative power before the grid gives out.
    pub negative_power_seconds: f32,
    /// Seconds before another collapse can happen at all.
    pub cooldown_seconds: f32,
    /// Structures standing at which the penalty is at its worst.
    pub full_scale_structures: f32,
    /// Shutdown for the smallest swarm, and for one at full scale.
    pub min_shutdown_seconds: f32,
    pub max_shutdown_seconds: f32,
    /// Research stays locked this many times as long as the shutdown.
    pub research_lock_ratio: f32,
    /// Share of stored Data and research progress lost, at each end.
    pub min_data_loss: f32,
    pub max_data_loss: f32,
    /// How long the banner stays up.
    pub notice_seconds: f32,
}

/// How ore is spread through the ground, and what happens to a rich patch as
/// it is cut. Only the bonus above ordinary ground depletes (gdd.md §3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OreConfig {
    /// Share of ground that is a rich deposit, and how rich it can be.
    pub rich_chance: f32,
    pub rich_min: f32,
    pub rich_max: f32,
    /// Share that is poorer than ordinary, and how poor.
    pub lean_chance: f32,
    pub lean_min: f32,
    pub lean_max: f32,
    /// Richness lost per unit of ore cut out of the tile.
    pub depletion_per_unit: f32,
}

/// What a Mass Driver throws, how fast it loads, and how long the throw takes.
///
/// Transit is measured off the orbits in `planets.json` rather than a flat
/// number, so shipping to the next world over is cheap and shipping across the
/// system is a commitment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassDriverConfig {
    /// Resource a driver pulls out of its hopper and into the pod per second.
    pub load_rate: f32,
    /// How much has to be loaded before the pod is thrown.
    pub pod_capacity: f32,
    /// Flight seconds per unit of difference between two orbit radii.
    pub seconds_per_orbit_unit: f32,
    /// However close two worlds are, a throw takes at least this long.
    pub min_transit_seconds: f32,
    /// How much cargo one Landing Pad can hold waiting for collection. A pod
    /// that finds every pad full holds in orbit until one has room.
    pub pad_capacity: f32,
    /// Resource ids a driver will accept, in the order the map cycles them.
    pub cargo: Vec<String>,
}

/// What settles on the machinery, what cleans it off, and how far each of
/// those reaches.
///
/// All of it used to be Rust constants, which meant the one loop the player
/// spends the whole game fighting could not be retuned without a rebuild.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpkeepConfig {
    /// Dust settling on every building per second, before research.
    pub dust_rate: f32,
    /// Dust one Sweeper clears per second, and how far it reaches.
    pub sweeper_rate: f32,
    pub sweeper_radius: i32,
    /// A preserved forest filter's reach, and what it does to the rate inside
    /// it.
    pub filter_radius: i32,
    pub filter_multiplier: f32,
    /// Cleared forest's reach, and what it does to the rate inside it.
    pub pollution_radius: i32,
    pub pollution_multiplier: f32,
    /// Acid at full strength corrodes the network this many times faster than
    /// dust settles on it.
    pub acid_multiplier: f32,
    /// Tiles a Shield Generator or Heater Node protects, and the share of the
    /// hazard it holds off inside that.
    pub hazard_counter_radius: i32,
    pub hazard_counter_strength: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingConfig {
    pub drill_output_rate: f32,
    /// Tiles the grid carries power before it needs a repeater to go further.
    #[serde(default = "default_repeater_range")]
    pub repeater_range: u32,
    /// Drones one network tile passes at full speed before they share it.
    pub conduit_capacity: f32,
    /// What each drone past that limit adds to the tile's routing cost, in
    /// tiles. Zero routes everything down the shortest run regardless.
    pub congestion_route_penalty: f32,
    /// How long a run has to be, in tiles, before a producer will send a drone
    /// out with less than a full load rather than make it wait.
    pub partial_load_min_route: f32,
    /// The least a partial load may be, as a share of a full one.
    pub partial_load_min_share: f32,
}

fn default_repeater_range() -> u32 {
    6
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
                min_start_region: default_min_start_region(),
            },
            resources: ResourceConfig {
                starting_energy: 50.0,
                starting_minerals: 50.0,
                drone_carry_capacity: 10.0,
                drones_per_drill: 1.0,
                // Tiles per second. Keep in step with assets/game_config.json.
                drone_speed: 8.0,
                max_energy: 50.0,
                base_mineral_cap: 100.0,
                storage_bonus: 100.0,
                core_data_rate: 0.25,
                server_data_rate: 1.0,
                research_rate: 5.0,
                forest_biomass: 60.0,
                biomass_power_output: 8.0,
                biomass_consumption_rate: 1.0,
            },
            buildings: BuildingConfig {
                // Minerals cut per second by one drill.
                drill_output_rate: 5.0,
                repeater_range: default_repeater_range(),
                conduit_capacity: 2.0,
                congestion_route_penalty: 1.5,
                partial_load_min_route: 6.0,
                partial_load_min_share: 0.5,
            },
            collapse: CollapseConfig::default(),
            ore: OreConfig::default(),
            mass_driver: MassDriverConfig::default(),
            upkeep: UpkeepConfig::default(),
        }
    }
}

impl Default for UpkeepConfig {
    fn default() -> Self {
        Self {
            dust_rate: 0.12,
            sweeper_rate: 0.6,
            sweeper_radius: 3,
            filter_radius: 3,
            filter_multiplier: 0.6,
            pollution_radius: 3,
            pollution_multiplier: 1.3,
            acid_multiplier: 4.0,
            hazard_counter_radius: 4,
            hazard_counter_strength: 0.9,
        }
    }
}

impl Default for MassDriverConfig {
    fn default() -> Self {
        Self {
            load_rate: 3.0,
            pod_capacity: 60.0,
            seconds_per_orbit_unit: 0.6,
            min_transit_seconds: 20.0,
            pad_capacity: 120.0,
            cargo: vec!["minerals".to_string(), "alloy".to_string()],
        }
    }
}

impl Default for OreConfig {
    fn default() -> Self {
        Self {
            rich_chance: 0.18,
            rich_min: 1.4,
            rich_max: 2.2,
            lean_chance: 0.22,
            lean_min: 0.5,
            lean_max: 0.8,
            depletion_per_unit: 0.0004,
        }
    }
}

impl Default for CollapseConfig {
    fn default() -> Self {
        Self {
            negative_power_seconds: 60.0,
            cooldown_seconds: 120.0,
            full_scale_structures: 60.0,
            min_shutdown_seconds: 6.0,
            max_shutdown_seconds: 45.0,
            research_lock_ratio: 1.5,
            min_data_loss: 0.1,
            max_data_loss: 0.35,
            notice_seconds: 10.0,
        }
    }
}
