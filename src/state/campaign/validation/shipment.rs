use crate::engine::GridPos;
use crate::state::shipping::Shipment;

use super::super::{Campaign, PLANET_COUNT};

impl Campaign {
    pub(super) fn validate_shipment(
        &self,
        shipment: &Shipment,
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
        if !shipment.cargo.is_physical() {
            return Err(format!("save {label} carries non-physical cargo"));
        }
        if shipment.priority >= 4 {
            return Err(format!("save {label} has an invalid priority"));
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
        if let Some(target_pad) = shipment.target_pad {
            let destination = self.planets[shipment.to].as_ref().expect("checked above");
            if !target_pad.in_bounds(destination.grid.width, destination.grid.height) {
                return Err(format!(
                    "save {label} points at an out-of-bounds landing pad"
                ));
            }
        }
        Ok(())
    }
}
