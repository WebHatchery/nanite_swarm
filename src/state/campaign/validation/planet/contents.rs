//! Validation for persisted planet contents and derived observations.

use crate::engine::GridPos;
use crate::state::camera::{MAX_ZOOM, MIN_ZOOM};
use crate::state::game_state::PlanetState;

use super::super::super::Campaign;

impl Campaign {
    pub(super) fn validate_planet_scalars(
        &self,
        planet: &PlanetState,
        source: usize,
    ) -> Result<(), String> {
        for (label, value) in [
            ("power balance", planet.power_balance),
            ("battery", planet.battery_seconds),
            ("export cooldown", planet.export_cooldown),
        ] {
            if !value.is_finite() || (label != "power balance" && value < 0.0) {
                return Err(format!("save world slot {source} has invalid {label}"));
            }
        }
        if !planet.time_played.is_finite() || planet.time_played < 0.0 {
            return Err(format!("save world slot {source} has invalid time played"));
        }
        if planet.last_saved_unix < 0 {
            return Err(format!(
                "save world slot {source} has an invalid save timestamp"
            ));
        }
        if !planet.camera.pan_x.is_finite()
            || !planet.camera.pan_y.is_finite()
            || !planet.camera.zoom.is_finite()
            || !(MIN_ZOOM..=MAX_ZOOM).contains(&planet.camera.zoom)
        {
            return Err(format!("save world slot {source} has invalid camera state"));
        }
        if planet.core_stages().is_empty()
            || planet.core_stage as usize >= planet.core_stages().len()
        {
            return Err(format!(
                "save world slot {source} has an invalid Core stage"
            ));
        }
        if planet.hazards.acid_rain.is_sign_negative()
            || !planet.hazards.acid_rain.is_finite()
            || planet.hazards.freeze.is_sign_negative()
            || !planet.hazards.freeze.is_finite()
        {
            return Err(format!(
                "save world slot {source} has invalid hazard values"
            ));
        }
        for (index, feature) in planet.features.iter().enumerate() {
            let [x, y, width, height] = [
                feature.bounds.0,
                feature.bounds.1,
                feature.bounds.2,
                feature.bounds.3,
            ];
            if ![x, y, width, height]
                .into_iter()
                .all(|value| value.is_finite())
                || !(0.0..=1.0).contains(&x)
                || !(0.0..=1.0).contains(&y)
                || !(0.0..=1.0).contains(&width)
                || !(0.0..=1.0).contains(&height)
                || x + width > 1.0
                || y + height > 1.0
            {
                return Err(format!(
                    "save world slot {source} feature {index} has invalid bounds"
                ));
            }
        }
        for (index, sample) in planet.graph_samples.iter().enumerate() {
            if [
                sample.power_produced,
                sample.power_consumed,
                sample.minerals_consumed,
                sample.alloy_produced,
                sample.alloy_consumed,
                sample.components_produced,
                sample.components_consumed,
                sample.data_produced,
                sample.data_consumed,
            ]
            .into_iter()
            .any(|value| !value.is_finite() || value < 0.0)
            {
                return Err(format!(
                    "save world slot {source} graph sample {index} has invalid values"
                ));
            }
        }
        for (index, collapse) in planet.collapse_history.iter().enumerate() {
            if !collapse.world_time.is_finite() || collapse.world_time < 0.0 {
                return Err(format!(
                    "save world slot {source} collapse record {index} has invalid time"
                ));
            }
            if collapse
                .position
                .is_some_and(|position| !position.in_bounds(planet.grid.width, planet.grid.height))
            {
                return Err(format!(
                    "save world slot {source} collapse record {index} has an out-of-bounds position"
                ));
            }
        }
        Ok(())
    }

    pub(super) fn validate_resources(
        &self,
        planet: &PlanetState,
        source: usize,
    ) -> Result<(), String> {
        for (label, value) in [
            ("energy", planet.resources.energy),
            ("minerals", planet.resources.minerals),
            ("data", planet.resources.data),
            ("biomass", planet.resources.biomass),
            ("alloy", planet.resources.alloy),
            ("components", planet.resources.components),
            ("consumed minerals", planet.resources.consumed.minerals),
            ("consumed energy", planet.resources.consumed.energy),
            ("consumed data", planet.resources.consumed.data),
            ("consumed biomass", planet.resources.consumed.biomass),
            ("consumed alloy", planet.resources.consumed.alloy),
            ("consumed components", planet.resources.consumed.components),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(format!(
                    "save world slot {source} has invalid {label} resources"
                ));
            }
        }
        Ok(())
    }

    pub(super) fn validate_grid_contents(
        &self,
        planet: &PlanetState,
        source: usize,
    ) -> Result<(), String> {
        for (position, tile) in planet.grid.iter_tiles() {
            if !tile.biomass_amount.is_finite()
                || tile.biomass_amount < 0.0
                || !tile.ore_richness.is_finite()
                || tile.ore_richness < 0.0
            {
                return Err(format!(
                    "save world slot {source} tile ({},{}) has invalid ground values",
                    position.x, position.y
                ));
            }
            let Some(building) = tile.building.as_ref() else {
                continue;
            };
            if building.position != position {
                return Err(format!(
                    "save world slot {source} building at ({},{}) claims position ({},{})",
                    position.x, position.y, building.position.x, building.position.y
                ));
            }
            for (label, value) in [
                ("efficiency", building.efficiency),
                ("dust", building.dust),
                ("acid wear", building.acid_wear),
                ("heat", building.heat),
            ] {
                if !value.is_finite() || value < 0.0 {
                    return Err(format!(
                        "save world slot {source} building at ({},{}) has invalid {label}",
                        position.x, position.y
                    ));
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate_buffers(
        &self,
        planet: &PlanetState,
        source: usize,
    ) -> Result<(), String> {
        for (label, buffers) in [
            ("output buffer", &planet.output_buffers),
            ("input buffer", &planet.input_buffers),
            ("pod load", &planet.pod_loads),
        ] {
            for (&(x, y), value) in buffers {
                self.validate_buffer_entry(planet, source, label, GridPos::new(x, y), *value)?;
            }
        }
        for (&(x, y), hoppers) in &planet.input_hoppers {
            let position = GridPos::new(x, y);
            if !position.in_bounds(planet.grid.width, planet.grid.height) {
                return Err(format!(
                    "save world slot {source} {label} is out of bounds",
                    label = "input hopper"
                ));
            }
            for (&resource, value) in hoppers {
                if !resource.is_physical() {
                    return Err(format!(
                        "save world slot {source} input hopper at ({x},{y}) carries non-physical resource"
                    ));
                }
                self.validate_buffer_entry(planet, source, "input hopper", position, *value)?;
            }
        }
        for (&(x, y), resource) in &planet.pad_cargo {
            let position = GridPos::new(x, y);
            if !position.in_bounds(planet.grid.width, planet.grid.height) {
                return Err(format!(
                    "save world slot {source} pad cargo is out of bounds"
                ));
            }
            if !resource.is_physical() {
                return Err(format!(
                    "save world slot {source} pad cargo carries a non-physical resource"
                ));
            }
        }
        Ok(())
    }

    fn validate_buffer_entry(
        &self,
        planet: &PlanetState,
        source: usize,
        label: &str,
        position: GridPos,
        value: f32,
    ) -> Result<(), String> {
        if !position.in_bounds(planet.grid.width, planet.grid.height) {
            return Err(format!(
                "save world slot {source} {label} at ({},{}) is out of bounds",
                position.x, position.y
            ));
        }
        if !value.is_finite() || value < 0.0 {
            return Err(format!(
                "save world slot {source} {label} at ({},{}) has invalid amount",
                position.x, position.y
            ));
        }
        Ok(())
    }
}
