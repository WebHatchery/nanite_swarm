//! Save-state invariants that must hold before the campaign is simulated.

use super::{Campaign, PLANET_COUNT};
use crate::engine::BuildingType;

mod planet;
mod shipment;

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
        self.validate_research(&self.research, "campaign research")?;
        self.validate_directive()?;
        if !self.directive_timer.is_finite() || self.directive_timer < 0.0 {
            return Err("save has an invalid directive timer".to_string());
        }
        if self.directive_tier < 0 {
            return Err("save has a negative directive tier".to_string());
        }
        for (index, record) in self.directive_history.iter().enumerate() {
            if !record.reward_data.is_finite() || record.reward_data < 0.0 || record.tier < 0 {
                return Err(format!(
                    "save directive history entry {index} has invalid progress"
                ));
            }
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
            self.validate_planet(planet, source)?;
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
            if !order.cargo.is_physical() {
                return Err(format!(
                    "save freight order from world {source} carries non-physical cargo"
                ));
            }
            if order.priority >= 4 {
                return Err(format!(
                    "save freight order from world {source} has an invalid priority"
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
            if let Some(target_pad) = order.target_pad {
                let destination = self.planets[order.target].as_ref().expect("checked above");
                if !target_pad.in_bounds(destination.grid.width, destination.grid.height) {
                    return Err(format!(
                        "save freight order from world {source} points at an out-of-bounds landing pad"
                    ));
                }
            }
        }

        for (index, shipment) in self.shipments.iter().enumerate() {
            self.validate_shipment(shipment, &format!("shipment {index}"), None)?;
        }

        Ok(())
    }

    fn validate_directive(&self) -> Result<(), String> {
        let directive = &self.directive;
        if directive.target <= 0
            || directive.progress < 0
            || directive.progress > directive.target
            || !directive.duration.is_finite()
            || directive.duration < 0.0
            || !directive.reward_data.is_finite()
            || directive.reward_data < 0.0
            || !directive.sustained.is_finite()
            || directive.sustained < 0.0
            || !directive.hold_seconds.is_finite()
            || directive.hold_seconds <= 0.0
            || directive.sustained > directive.hold_seconds
        {
            return Err("save has invalid directive progress".to_string());
        }
        Ok(())
    }
}
