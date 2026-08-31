//! Validation for the loaded balance contract.

use super::GameConfig;
use crate::engine::ResourceType;

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
        for resource in &self.mass_driver.cargo {
            let Some(resource_type) = ResourceType::from_id(resource) else {
                return Err(format!(
                    "mass_driver.cargo contains unknown resource \"{resource}\""
                ));
            };
            if !resource_type.is_physical() {
                return Err(format!(
                    "mass_driver.cargo resource \"{resource}\" is not physical"
                ));
            }
        }

        for (field, value) in [
            ("upkeep.dust_rate", self.upkeep.dust_rate),
            ("upkeep.sweeper_rate", self.upkeep.sweeper_rate),
            (
                "upkeep.pollution_multiplier",
                self.upkeep.pollution_multiplier,
            ),
            ("upkeep.acid_multiplier", self.upkeep.acid_multiplier),
        ] {
            require_nonnegative(value, field)?;
        }
        for (field, value) in [
            ("upkeep.filter_multiplier", self.upkeep.filter_multiplier),
            (
                "upkeep.forest_buffer_multiplier",
                self.upkeep.forest_buffer_multiplier,
            ),
            (
                "upkeep.hazard_counter_strength",
                self.upkeep.hazard_counter_strength,
            ),
        ] {
            require_unit(value, field)?;
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
