//! Mass Drivers: what a world throws at another world, and what is in flight.
//!
//! The driver is a logistics consumer like any other. Drones carry ore to its
//! hopper over the conduit network, it pulls that into a pod, and when the pod
//! is full it throws it. Nothing teleports: a world that has not routed
//! anything to its driver exports nothing, however large its stockpile.
//!
//! The order (what to load, where to throw it) belongs to the world, not to
//! one driver, and it survives being left behind — that is the whole point. A
//! world the swarm walked away from keeps feeding the world it walked to.

use serde::{Deserialize, Serialize};

use crate::data::MassDriverConfig;
use crate::engine::{BuildingType, GridPos, ResourceType};

use super::campaign::Campaign;
use super::game_state::PlanetState;

mod campaign;
mod planet;

/// Source-stock thresholds a standing order can protect. Zero means the
/// driver may pull from all available stock; positive values keep that amount
/// on the source world before surplus shipping resumes.
const EXPORT_RESERVE_CHOICES: [f32; 5] = [0.0, 100.0, 250.0, 500.0, 1_000.0];

/// A standing order for a world: what its Mass Drivers load, and where they
/// throw it.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ExportOrder {
    pub target: usize,
    pub cargo: ResourceType,
    #[serde(default)]
    pub target_pad: Option<GridPos>,
    #[serde(default)]
    pub schedule_seconds: f32,
    #[serde(default)]
    pub priority: u8,
    #[serde(default)]
    pub reserve_source: f32,
    #[serde(default)]
    pub surplus_only: bool,
}

impl Default for ExportOrder {
    fn default() -> Self {
        Self {
            target: 0,
            cargo: ResourceType::Minerals,
            target_pad: None,
            schedule_seconds: 0.0,
            priority: 0,
            reserve_source: 0.0,
            surplus_only: false,
        }
    }
}

/// A pod that has left the ground and not yet landed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shipment {
    pub from: usize,
    pub to: usize,
    pub cargo: ResourceType,
    pub amount: f32,
    /// World seconds still to fly, out of the [`Shipment::transit`] it started
    /// with, so the map can draw how far along it is.
    pub remaining: f32,
    pub transit: f32,
    #[serde(default)]
    pub target_pad: Option<GridPos>,
    #[serde(default)]
    pub overflow: bool,
    /// Copied from the standing order so simultaneous arrivals can be
    /// resolved in a stable, player-visible order.
    #[serde(default)]
    pub priority: u8,
}

impl Shipment {
    /// How much of the flight is behind it, from zero at launch to one on
    /// landing.
    pub fn progress(&self) -> f32 {
        if self.transit <= 0.0 {
            return 1.0;
        }
        (1.0 - self.remaining / self.transit).clamp(0.0, 1.0)
    }

    /// Arrived, and circling: the destination has no pad with room on it. The
    /// cargo is not lost — it lands the moment somewhere can take it.
    pub fn is_holding(&self) -> bool {
        self.remaining <= 0.0
    }
}

/// How long a throw between two worlds takes, off their orbits.
pub fn transit_seconds(
    data: &crate::data::GameData,
    from: usize,
    to: usize,
    config: &MassDriverConfig,
) -> f32 {
    let gap = (data.planet(from).orbit_radius - data.planet(to).orbit_radius).abs();
    (gap * config.seconds_per_orbit_unit).max(config.min_transit_seconds)
}

/// The first cargo a driver accepts, for a route that has just been set.
fn default_cargo(config: &MassDriverConfig) -> ResourceType {
    accepted_cargo(config)
        .first()
        .copied()
        .unwrap_or(ResourceType::Minerals)
}

/// What the drivers will throw, in the order the map cycles them. Anything the
/// engine does not recognise, or that no drone could carry there, is dropped.
fn accepted_cargo(config: &MassDriverConfig) -> Vec<ResourceType> {
    config
        .cargo
        .iter()
        .filter_map(|id| ResourceType::from_id(id))
        .filter(|cargo| cargo.is_physical())
        .collect()
}

/// A resource as the player reads it on a shipping label.
pub fn cargo_name(cargo: ResourceType) -> &'static str {
    match cargo {
        ResourceType::Minerals => "Minerals",
        ResourceType::Energy => "Energy",
        ResourceType::Data => "Data",
        ResourceType::Biomass => "Biomass",
        ResourceType::Alloy => "Alloy",
        ResourceType::Components => "Components",
    }
}

#[cfg(test)]
mod tests;
