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
    #[serde(default)]
    pub logistics: LogisticsConfig,
    #[serde(default)]
    pub offline: OfflineConfig,
}

impl GameConfig {
    /// Validate the balance contract before it is copied into every planet.
    ///
    /// This is intentionally kept beside the configuration types instead of
    /// scattered through simulation systems. A bad package should fall back
    /// as one coherent set, not get independently clamped by each consumer.
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.grid.initial_width == 0 || self.grid.initial_height == 0 {
            return Err("grid initial dimensions must be positive".to_string());
        }
        if self.grid.max_width < self.grid.initial_width
            || self.grid.max_height < self.grid.initial_height
        {
            return Err("grid maximum dimensions must include the initial grid".to_string());
        }
        require_positive(self.grid.tile_size, "grid.tile_size")?;
        require_unit(self.grid.min_start_region, "grid.min_start_region")?;

        for (field, value) in [
            ("resources.starting_energy", self.resources.starting_energy),
            (
                "resources.starting_minerals",
                self.resources.starting_minerals,
            ),
            ("resources.storage_bonus", self.resources.storage_bonus),
            ("resources.core_data_rate", self.resources.core_data_rate),
            (
                "resources.server_data_rate",
                self.resources.server_data_rate,
            ),
            ("resources.research_rate", self.resources.research_rate),
            ("resources.forest_biomass", self.resources.forest_biomass),
            (
                "resources.biomass_power_output",
                self.resources.biomass_power_output,
            ),
            (
                "resources.biomass_consumption_rate",
                self.resources.biomass_consumption_rate,
            ),
        ] {
            require_nonnegative(value, field)?;
        }
        for (field, value) in [
            (
                "resources.drone_carry_capacity",
                self.resources.drone_carry_capacity,
            ),
            (
                "resources.drones_per_drill",
                self.resources.drones_per_drill,
            ),
            ("resources.drone_speed", self.resources.drone_speed),
            ("resources.max_energy", self.resources.max_energy),
            (
                "resources.base_mineral_cap",
                self.resources.base_mineral_cap,
            ),
        ] {
            require_positive(value, field)?;
        }

        require_positive(
            self.buildings.drill_output_rate,
            "buildings.drill_output_rate",
        )?;
        require_positive(
            self.buildings.conduit_capacity,
            "buildings.conduit_capacity",
        )?;
        for (field, value) in [
            (
                "buildings.congestion_route_penalty",
                self.buildings.congestion_route_penalty,
            ),
            (
                "buildings.partial_load_min_route",
                self.buildings.partial_load_min_route,
            ),
            (
                "buildings.server_bank_heat_per_second",
                self.buildings.server_bank_heat_per_second,
            ),
            (
                "buildings.water_cooling_rate",
                self.buildings.water_cooling_rate,
            ),
        ] {
            require_nonnegative(value, field)?;
        }
        require_unit(
            self.buildings.partial_load_min_share,
            "buildings.partial_load_min_share",
        )?;
        require_positive(
            self.buildings.server_bank_heat_capacity,
            "buildings.server_bank_heat_capacity",
        )?;
        require_unit(
            self.buildings.overheat_penalty,
            "buildings.overheat_penalty",
        )?;

        for (field, value) in [
            (
                "collapse.negative_power_seconds",
                self.collapse.negative_power_seconds,
            ),
            ("collapse.cooldown_seconds", self.collapse.cooldown_seconds),
            (
                "collapse.full_scale_structures",
                self.collapse.full_scale_structures,
            ),
            (
                "collapse.min_shutdown_seconds",
                self.collapse.min_shutdown_seconds,
            ),
            (
                "collapse.max_shutdown_seconds",
                self.collapse.max_shutdown_seconds,
            ),
            (
                "collapse.research_lock_ratio",
                self.collapse.research_lock_ratio,
            ),
            ("collapse.notice_seconds", self.collapse.notice_seconds),
        ] {
            require_nonnegative(value, field)?;
        }
        if self.collapse.min_shutdown_seconds > self.collapse.max_shutdown_seconds {
            return Err("collapse shutdown bounds are inverted".to_string());
        }
        require_unit(self.collapse.min_data_loss, "collapse.min_data_loss")?;
        require_unit(self.collapse.max_data_loss, "collapse.max_data_loss")?;
        if self.collapse.min_data_loss > self.collapse.max_data_loss {
            return Err("collapse data-loss bounds are inverted".to_string());
        }

        require_unit(self.ore.rich_chance, "ore.rich_chance")?;
        require_unit(self.ore.lean_chance, "ore.lean_chance")?;
        for (field, value) in [
            ("ore.rich_min", self.ore.rich_min),
            ("ore.rich_max", self.ore.rich_max),
            ("ore.lean_min", self.ore.lean_min),
            ("ore.lean_max", self.ore.lean_max),
            ("ore.depletion_per_unit", self.ore.depletion_per_unit),
        ] {
            require_nonnegative(value, field)?;
        }
        if self.ore.rich_min > self.ore.rich_max || self.ore.lean_min > self.ore.lean_max {
            return Err("ore richness bounds are inverted".to_string());
        }

        require_positive(self.mass_driver.load_rate, "mass_driver.load_rate")?;
        require_positive(self.mass_driver.pod_capacity, "mass_driver.pod_capacity")?;
        require_nonnegative(
            self.mass_driver.seconds_per_orbit_unit,
            "mass_driver.seconds_per_orbit_unit",
        )?;
        require_positive(
            self.mass_driver.min_transit_seconds,
            "mass_driver.min_transit_seconds",
        )?;
        require_positive(self.mass_driver.pad_capacity, "mass_driver.pad_capacity")?;
        if self.mass_driver.cargo.is_empty() {
            return Err("mass_driver.cargo must contain at least one resource".to_string());
        }

        for (field, value) in [
            ("upkeep.dust_rate", self.upkeep.dust_rate),
            ("upkeep.sweeper_rate", self.upkeep.sweeper_rate),
            ("upkeep.filter_multiplier", self.upkeep.filter_multiplier),
            (
                "upkeep.forest_buffer_multiplier",
                self.upkeep.forest_buffer_multiplier,
            ),
            (
                "upkeep.pollution_multiplier",
                self.upkeep.pollution_multiplier,
            ),
            ("upkeep.acid_multiplier", self.upkeep.acid_multiplier),
            (
                "upkeep.hazard_counter_strength",
                self.upkeep.hazard_counter_strength,
            ),
        ] {
            require_nonnegative(value, field)?;
        }
        for (field, radius) in [
            ("upkeep.sweeper_radius", self.upkeep.sweeper_radius),
            ("upkeep.filter_radius", self.upkeep.filter_radius),
            (
                "upkeep.forest_buffer_radius",
                self.upkeep.forest_buffer_radius,
            ),
            ("upkeep.pollution_radius", self.upkeep.pollution_radius),
            (
                "upkeep.hazard_counter_radius",
                self.upkeep.hazard_counter_radius,
            ),
        ] {
            if radius < 0 {
                return Err(format!("{field} must be nonnegative"));
            }
        }
        for (field, value) in [
            (
                "upkeep.dust_response.efficiency_threshold",
                self.upkeep.dust_response.efficiency_threshold,
            ),
            (
                "upkeep.dust_response.speed_threshold",
                self.upkeep.dust_response.speed_threshold,
            ),
            (
                "upkeep.dust_response.leak_threshold",
                self.upkeep.dust_response.leak_threshold,
            ),
            (
                "upkeep.dust_response.stall_threshold",
                self.upkeep.dust_response.stall_threshold,
            ),
        ] {
            require_nonnegative(value, field)?;
        }
        require_unit(
            self.upkeep.dust_response.efficiency,
            "upkeep.dust_response.efficiency",
        )?;
        require_unit(
            self.upkeep.dust_response.speed_multiplier,
            "upkeep.dust_response.speed_multiplier",
        )?;
        require_unit(self.upkeep.dust_response.leak, "upkeep.dust_response.leak")?;

        for (field, value) in [
            ("logistics.pad_depth", self.logistics.pad_depth),
            ("logistics.hopper_depth", self.logistics.hopper_depth),
            (
                "logistics.reservation_seconds",
                self.logistics.reservation_seconds,
            ),
            (
                "logistics.dispatch_lookahead",
                self.logistics.dispatch_lookahead,
            ),
            (
                "logistics.path_validation_budget_ms",
                self.logistics.path_validation_budget_ms,
            ),
        ] {
            require_nonnegative(value, field)?;
        }
        if self.logistics.max_queue_depth == 0 {
            return Err("logistics.max_queue_depth must be positive".to_string());
        }

        for (field, value) in [
            (
                "offline.max_elapsed_seconds",
                self.offline.max_elapsed_seconds,
            ),
            ("offline.max_resource_gain", self.offline.max_resource_gain),
            (
                "offline.clock_tamper_tolerance_seconds",
                self.offline.clock_tamper_tolerance_seconds,
            ),
            (
                "offline.fallback_delta_seconds",
                self.offline.fallback_delta_seconds,
            ),
        ] {
            require_nonnegative(value, field)?;
        }
        Ok(())
    }
}

fn require_nonnegative(value: f32, field: &str) -> Result<(), String> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(format!("{field} must be finite and nonnegative"))
    }
}

fn require_positive(value: f32, field: &str) -> Result<(), String> {
    if value.is_finite() && value > 0.0 {
        Ok(())
    } else {
        Err(format!("{field} must be finite and positive"))
    }
}

fn require_unit(value: f32, field: &str) -> Result<(), String> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(format!("{field} must be finite and between 0 and 1"))
    }
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

fn default_forest_buffer_radius() -> i32 {
    3
}

fn default_forest_buffer_multiplier() -> f32 {
    0.75
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
    /// A converted forest filter's reach, and what it does to the rate inside
    /// it.
    pub filter_radius: i32,
    pub filter_multiplier: f32,
    /// An intact forest's reach, and the dust share that remains inside it.
    #[serde(default = "default_forest_buffer_radius")]
    pub forest_buffer_radius: i32,
    #[serde(default = "default_forest_buffer_multiplier")]
    pub forest_buffer_multiplier: f32,
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
    #[serde(default)]
    pub dust_response: DustResponseConfig,
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
    pub server_bank_heat_per_second: f32,
    pub server_bank_heat_capacity: f32,
    pub water_cooling_rate: f32,
    pub overheat_penalty: f32,
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
                server_bank_heat_per_second: 0.8,
                server_bank_heat_capacity: 100.0,
                water_cooling_rate: 2.0,
                overheat_penalty: 0.5,
            },
            collapse: CollapseConfig::default(),
            ore: OreConfig::default(),
            mass_driver: MassDriverConfig::default(),
            upkeep: UpkeepConfig::default(),
            logistics: LogisticsConfig::default(),
            offline: OfflineConfig::default(),
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
            forest_buffer_radius: 3,
            forest_buffer_multiplier: 0.75,
            pollution_radius: 3,
            pollution_multiplier: 1.3,
            acid_multiplier: 4.0,
            hazard_counter_radius: 4,
            hazard_counter_strength: 0.9,
            dust_response: DustResponseConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DustResponseConfig {
    pub efficiency_threshold: f32,
    pub efficiency: f32,
    pub speed_threshold: f32,
    pub speed_multiplier: f32,
    pub leak_threshold: f32,
    pub leak: f32,
    pub stall_threshold: f32,
}

impl Default for DustResponseConfig {
    fn default() -> Self {
        Self {
            efficiency_threshold: 25.0,
            efficiency: 0.9,
            speed_threshold: 50.0,
            speed_multiplier: 0.7,
            leak_threshold: 75.0,
            leak: 0.5,
            stall_threshold: 100.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogisticsConfig {
    pub pad_depth: f32,
    pub hopper_depth: f32,
    pub reservation_seconds: f32,
    pub dispatch_lookahead: f32,
    pub max_queue_depth: usize,
    pub path_validation_budget_ms: f32,
}

impl Default for LogisticsConfig {
    fn default() -> Self {
        Self {
            pad_depth: 30.0,
            hopper_depth: 30.0,
            reservation_seconds: 1.5,
            dispatch_lookahead: 3.0,
            max_queue_depth: 64,
            path_validation_budget_ms: 4.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OfflineConfig {
    pub max_elapsed_seconds: f32,
    pub max_resource_gain: f32,
    pub clock_tamper_tolerance_seconds: f32,
    pub fallback_delta_seconds: f32,
}

impl Default for OfflineConfig {
    fn default() -> Self {
        Self {
            max_elapsed_seconds: 86_400.0,
            max_resource_gain: 10_000.0,
            clock_tamper_tolerance_seconds: 300.0,
            fallback_delta_seconds: 60.0,
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
            cargo: vec![
                "minerals".to_string(),
                "alloy".to_string(),
                "components".to_string(),
            ],
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

#[cfg(test)]
mod tests;
