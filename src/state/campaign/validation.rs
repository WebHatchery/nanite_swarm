//! Save-state invariants that must hold before the campaign is simulated.

use super::{Campaign, PLANET_COUNT};
use crate::engine::BuildingType;

impl Campaign {
    /// Reject references that a hand-edited or partially written save could
    /// otherwise turn into a panic during the first frame after loading.
    pub(crate) fn validate_loaded_state(&self) -> Result<(), String> {
        if self.current >= PLANET_COUNT || self.planets[self.current].is_none() {
            return Err(format!(
                "save points at missing current world {}",
                self.current
            ));
        }

        for (source, planet) in self.planets.iter().enumerate() {
            let Some(planet) = planet else {
                continue;
            };
            if planet.planet_index != source {
                return Err(format!(
                    "save world slot {source} contains world {}",
                    planet.planet_index
                ));
            }
            self.validate_grid_shape(planet, source)?;
            let core_count = planet.grid.find_buildings(BuildingType::Core).len();
            if core_count != 1 {
                return Err(format!(
                    "save world slot {source} has {core_count} Cores; exactly one is required"
                ));
            }
            self.validate_drones(planet, source)?;
            for (index, shipment) in planet.launched_pods.iter().enumerate() {
                self.validate_shipment(
                    shipment,
                    &format!("queued shipment {source}:{index}"),
                    Some(source),
                )?;
            }
            let Some(order) = planet.export else {
                continue;
            };
            if order.target >= PLANET_COUNT {
                return Err(format!(
                    "save freight order from world {source} points outside the system"
                ));
            }
            if order.target == source {
                return Err(format!(
                    "save freight order from world {source} points at itself"
                ));
            }
            if self.planets[order.target].is_none() {
                return Err(format!(
                    "save freight order from world {source} points at untouched world {}",
                    order.target
                ));
            }
            if !order.schedule_seconds.is_finite() || order.schedule_seconds < 0.0 {
                return Err(format!(
                    "save freight order from world {source} has an invalid cooldown"
                ));
            }
            if !order.reserve_source.is_finite() || order.reserve_source < 0.0 {
                return Err(format!(
                    "save freight order from world {source} has an invalid reserve"
                ));
            }
        }

        for (index, shipment) in self.shipments.iter().enumerate() {
            self.validate_shipment(shipment, &format!("shipment {index}"), None)?;
        }

        Ok(())
    }

    fn validate_drones(
        &self,
        planet: &super::super::PlanetState,
        source: usize,
    ) -> Result<(), String> {
        let width = planet.grid.width;
        let height = planet.grid.height;
        for (index, drone) in planet.drones.drones().iter().enumerate() {
            for (label, position) in [
                ("position", drone.position),
                ("target", drone.target),
                ("home", drone.home),
            ] {
                if !position.in_bounds(width, height) {
                    return Err(format!(
                        "save world slot {source} drone {index} has an out-of-bounds {label}"
                    ));
                }
            }
            if drone.path_index > drone.path.len() {
                return Err(format!(
                    "save world slot {source} drone {index} points past its route"
                ));
            }
            if drone
                .path
                .iter()
                .any(|position| !position.in_bounds(width, height))
            {
                return Err(format!(
                    "save world slot {source} drone {index} has an out-of-bounds route"
                ));
            }
            if !drone.carrying.is_finite()
                || drone.carrying < 0.0
                || !drone.capacity.is_finite()
                || drone.capacity < 0.0
                || !drone.speed.is_finite()
                || drone.speed < 0.0
                || !drone.progress.is_finite()
                || !(0.0..1.0).contains(&drone.progress)
            {
                return Err(format!(
                    "save world slot {source} drone {index} has invalid flight data"
                ));
            }
        }
        Ok(())
    }

    fn validate_grid_shape(
        &self,
        planet: &super::super::PlanetState,
        source: usize,
    ) -> Result<(), String> {
        let width = planet.grid.width;
        let height = planet.grid.height;
        if width == 0 || height == 0 {
            return Err(format!("save world slot {source} has an empty grid"));
        }
        if width > i32::MAX as u32 || height > i32::MAX as u32 {
            return Err(format!(
                "save world slot {source} has grid dimensions too large"
            ));
        }
        let expected_tiles = width as u64 * height as u64;
        let actual_tiles = planet.grid.iter_tiles().count() as u64;
        if expected_tiles != actual_tiles {
            return Err(format!(
                "save world slot {source} has {actual_tiles} tiles for a {width}x{height} grid"
            ));
        }
        Ok(())
    }

    fn validate_shipment(
        &self,
        shipment: &super::super::shipping::Shipment,
        label: &str,
        expected_source: Option<usize>,
    ) -> Result<(), String> {
        if shipment.from >= PLANET_COUNT || shipment.to >= PLANET_COUNT {
            return Err(format!("save {label} points outside the system"));
        }
        if expected_source.is_some_and(|source| source != shipment.from) {
            return Err(format!("save {label} has the wrong source world"));
        }
        if shipment.from == shipment.to {
            return Err(format!("save {label} points at its source world"));
        }
        if self.planets[shipment.from].is_none() || self.planets[shipment.to].is_none() {
            return Err(format!("save {label} references an untouched world"));
        }
        if !shipment.amount.is_finite()
            || shipment.amount < 0.0
            || !shipment.remaining.is_finite()
            || shipment.remaining < 0.0
            || !shipment.transit.is_finite()
            || shipment.transit < 0.0
        {
            return Err(format!("save {label} has invalid flight data"));
        }
        Ok(())
    }
}
