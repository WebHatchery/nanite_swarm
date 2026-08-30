//! Save-state invariants that must hold before the campaign is simulated.

use super::{Campaign, PLANET_COUNT};

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
