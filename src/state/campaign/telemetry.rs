//! Read-only snapshots used by the Solar System map and world dossier.

use crate::engine::ResourceType;
use crate::state::{CoreRequirementProgress, UpkeepAlertCause};

use super::Campaign;

/// The small live dossier the Solar System map can show for every world the
/// swarm has already reached. These are copied values rather than references
/// so the map can assemble one stable snapshot while the campaign remains
/// mutable behind the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldAlert {
    Nominal,
    Collapse,
    Power,
    Hibernation,
    Stalled,
    Starved,
    Blocked,
    Storage,
    Maintenance,
}

impl WorldAlert {
    pub fn label(self) -> &'static str {
        match self {
            Self::Nominal => "NOMINAL",
            Self::Collapse => "COLLAPSE",
            Self::Power => "POWER",
            Self::Hibernation => "HIBERNATION",
            Self::Stalled => "STALL",
            Self::Starved => "INPUT",
            Self::Blocked => "BLOCKED",
            Self::Storage => "STORAGE",
            Self::Maintenance => "MAINTENANCE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldTelemetry {
    pub minerals: f32,
    /// The live mineral ceiling, including Storage buildings and research.
    /// Other pools have a fixed shared ceiling; minerals are the remote pool
    /// the player can deliberately expand from the Solar System screen.
    pub mineral_capacity: f32,
    pub data: f32,
    pub biomass: f32,
    pub alloy: f32,
    pub components: f32,
    pub ore_rate: f32,
    pub data_rate: f32,
    pub biomass_rate: f32,
    pub alloy_rate: f32,
    pub components_rate: f32,
    pub power_balance: f32,
    pub structures: usize,
    pub drones: usize,
    /// The world's earned Core evolution, one-based for player-facing
    /// telemetry. The total travels with it so the dossier remains honest if
    /// the data file gains another stage later.
    pub core_stage: usize,
    pub core_stage_count: usize,
    /// The first unmet requirement for the next Core stage, if one exists.
    /// Keeping this as a copy lets the Solar System explain remote growth
    /// without borrowing a planet through the rendering layer.
    pub core_next_requirement: Option<CoreRequirementProgress>,
    pub alerts: usize,
    pub alert_kind: WorldAlert,
    /// The first stored resource at its live ceiling while that resource has
    /// a live producer, if any. Energy is excluded because a full battery is
    /// normal for a healthy remote world, and idle full reserves are not waste.
    /// The alert count still reports saturation as one distinct condition even
    /// when more than one stored resource is full.
    pub storage_full_resource: Option<ResourceType>,
    /// Whether remote maintenance is caused by dust, corrosion, or both.
    pub maintenance_cause: Option<UpkeepAlertCause>,
}

impl Campaign {
    /// Minerals stockpiled on a world, for the map to show what is waiting.
    pub fn stockpile(&self, index: usize) -> Option<f32> {
        self.planet(index).map(|planet| planet.resources.minerals)
    }

    /// Read the operational pulse of a colonized world without travelling to
    /// it. The map uses the same observed production windows as the planetary
    /// HUD, so a background world can say that it is quiet or bottlenecked
    /// rather than pretending its theoretical capacity is live output.
    pub fn world_telemetry(&self, index: usize) -> Option<WorldTelemetry> {
        let planet = self.planet(index)?;
        let power_alert = planet.power_balance < 0.0;
        let hibernating = planet.battery_seconds <= 0.0;
        let collapse = planet.power_collapse_shutdown > 0.0;
        let stalled = planet.stalled_drone_count() > 0;
        let starved = !planet.starved_factories().is_empty();
        let blocked = !planet.blocked_factories().is_empty();
        let storage_full_resource = [
            (ResourceType::Minerals, planet.mineral_output_rate()),
            (ResourceType::Data, planet.data_generation_rate()),
            (ResourceType::Biomass, planet.biomass_harvest_rate()),
            (ResourceType::Alloy, planet.alloy_rate()),
            (ResourceType::Components, planet.components_rate()),
        ]
        .into_iter()
        .find_map(|(resource, rate)| {
            (rate > 0.0 && planet.resource_is_full(resource)).then_some(resource)
        });
        let storage_full = storage_full_resource.is_some();
        let maintenance_cause = planet.upkeep_alert_cause();
        let maintenance = maintenance_cause.is_some();
        let alerts = usize::from(power_alert)
            + usize::from(hibernating)
            + usize::from(collapse)
            + usize::from(stalled)
            + usize::from(starved)
            + usize::from(blocked)
            + usize::from(storage_full)
            + usize::from(maintenance);
        let alert_kind = if collapse {
            WorldAlert::Collapse
        } else if power_alert {
            WorldAlert::Power
        } else if hibernating {
            WorldAlert::Hibernation
        } else if stalled {
            WorldAlert::Stalled
        } else if starved {
            WorldAlert::Starved
        } else if blocked {
            WorldAlert::Blocked
        } else if storage_full {
            WorldAlert::Storage
        } else if maintenance {
            WorldAlert::Maintenance
        } else {
            WorldAlert::Nominal
        };
        Some(WorldTelemetry {
            minerals: planet.resources.minerals,
            mineral_capacity: planet.resource_capacity(ResourceType::Minerals),
            data: planet.resources.data,
            biomass: planet.resources.biomass,
            alloy: planet.resources.alloy,
            components: planet.resources.components,
            ore_rate: planet.throughput.last().unwrap_or(0.0),
            data_rate: planet.data_generation_rate(),
            biomass_rate: planet.biomass_harvest_rate(),
            alloy_rate: planet.observed_alloy_rate(),
            components_rate: planet.observed_components_rate(),
            power_balance: planet.power_balance,
            structures: planet.grid.total_buildings(),
            drones: planet.drones.total_count(),
            core_stage: planet.core_stage_index() + 1,
            core_stage_count: planet.core_stages().len(),
            core_next_requirement: planet.next_core_requirement(),
            alerts,
            alert_kind,
            storage_full_resource,
            maintenance_cause,
        })
    }
}
