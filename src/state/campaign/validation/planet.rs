use std::collections::HashSet;

use crate::state::game_state::{PlanetState, ResearchProgress};

use super::super::Campaign;

mod contents;

impl Campaign {
    pub(super) fn validate_planet(
        &self,
        planet: &PlanetState,
        source: usize,
    ) -> Result<(), String> {
        planet.config.validate().map_err(|error| {
            format!("save world slot {source} has invalid configuration: {error}")
        })?;
        self.validate_planet_scalars(planet, source)?;
        self.validate_resources(planet, source)?;
        self.validate_research(&planet.research, &format!("world {source} research"))?;
        self.validate_grid_contents(planet, source)?;
        self.validate_buffers(planet, source)?;
        planet
            .seed_ship
            .validate_loaded_state()
            .map_err(|error| format!("save world slot {source} {error}"))?;
        Ok(())
    }

    pub(super) fn validate_research(
        &self,
        research: &ResearchProgress,
        label: &str,
    ) -> Result<(), String> {
        let nodes = &self.data.data().research.nodes;
        for (index, id) in research.unlocked_techs.iter().enumerate() {
            let known = nodes.iter().any(|node| node.id == *id);
            if !known || research.unlocked_techs[..index].contains(id) {
                return Err(format!("save {label} has an invalid unlocked research id"));
            }
        }
        if !research.research_progress.is_finite() || research.research_progress < 0.0 {
            return Err(format!("save {label} has invalid active progress"));
        }
        match research.current_research.as_deref() {
            Some(id) => {
                let Some(node) = nodes.iter().find(|node| node.id == id) else {
                    return Err(format!("save {label} points at unknown active research"));
                };
                if research
                    .unlocked_techs
                    .iter()
                    .any(|unlocked| unlocked == id)
                    || research.research_progress > node.data_cost
                {
                    return Err(format!("save {label} has invalid active research progress"));
                }
            }
            None if research.research_progress != 0.0 => {
                return Err(format!("save {label} has progress without active research"));
            }
            None => {}
        }
        for (id, progress) in &research.parked_progress {
            let Some(node) = nodes.iter().find(|node| node.id == *id) else {
                return Err(format!("save {label} has unknown parked research"));
            };
            if !progress.is_finite()
                || *progress < 0.0
                || *progress > node.data_cost
                || research.current_research.as_deref() == Some(id)
                || research
                    .unlocked_techs
                    .iter()
                    .any(|unlocked| unlocked == id)
            {
                return Err(format!("save {label} has invalid parked research progress"));
            }
        }
        Ok(())
    }

    pub(super) fn validate_drones(
        &self,
        planet: &PlanetState,
        source: usize,
    ) -> Result<(), String> {
        let width = planet.grid.width;
        let height = planet.grid.height;
        planet
            .drones
            .validate_loaded_ids()
            .map_err(|error| format!("save world slot {source} {error}"))?;
        let mut ids = HashSet::new();
        for (index, drone) in planet.drones.drones().iter().enumerate() {
            if !ids.insert(drone.id) {
                return Err(format!(
                    "save world slot {source} has duplicate drone id {}",
                    drone.id
                ));
            }
            if !drone.resource_type.is_physical() {
                return Err(format!(
                    "save world slot {source} drone {index} carries a non-physical resource"
                ));
            }
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
                || drone.capacity <= 0.0
                || !drone.speed.is_finite()
                || drone.speed <= 0.0
                || !drone.progress.is_finite()
                || !(0.0..1.0).contains(&drone.progress)
                || drone.carrying > drone.capacity
            {
                return Err(format!(
                    "save world slot {source} drone {index} has invalid flight data"
                ));
            }
        }
        Ok(())
    }

    pub(super) fn validate_grid_shape(
        &self,
        planet: &PlanetState,
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
}
