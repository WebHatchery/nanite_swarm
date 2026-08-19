//! Building catalog: identity, cost, and static definitions

use crate::data;
use serde::{Deserialize, Serialize};

/// Building types that can be placed on the grid
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingType {
    Core,             // Central AI structure - receives resources
    Drill,            // Extracts minerals, spawns drones
    Conduit,          // Connects buildings for resource flow
    Bridge,           // Allows conduit crossings (overlay)
    PowerNode,        // Extends power grid
    WindTurbine,      // Generates power (bonus on mountains)
    ServerBank,       // Generates data, consumes power
    Sweeper,          // Cleans dust buildup in nearby buildings
    Storage,          // Increases mineral storage capacity
    BiomassHarvester, // Consumes forest biomass for power
    Smelter,          // Refines minerals into alloy
    Assembler,        // Turns routed ore and alloy into precision components
    HeaterNode,       // Thaws the network nearby on frozen worlds
    ShieldGenerator,  // Holds acid off everything nearby
    MassDriver,       // Throws cargo at another world
    LandingPad,       // Catches what another world threw
}

impl BuildingType {
    pub fn id(&self) -> &'static str {
        match self {
            BuildingType::Core => "core",
            BuildingType::Drill => "drill",
            BuildingType::Conduit => "conduit",
            BuildingType::Bridge => "bridge",
            BuildingType::PowerNode => "power_node",
            BuildingType::WindTurbine => "wind_turbine",
            BuildingType::ServerBank => "server_bank",
            BuildingType::Sweeper => "sweeper",
            BuildingType::Storage => "storage",
            BuildingType::BiomassHarvester => "biomass_harvester",
            BuildingType::Smelter => "smelter",
            BuildingType::Assembler => "assembler",
            BuildingType::HeaterNode => "heater_node",
            BuildingType::ShieldGenerator => "shield_generator",
            BuildingType::MassDriver => "mass_driver",
            BuildingType::LandingPad => "landing_pad",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "core" => Some(BuildingType::Core),
            "drill" => Some(BuildingType::Drill),
            "conduit" => Some(BuildingType::Conduit),
            "bridge" => Some(BuildingType::Bridge),
            "power_node" => Some(BuildingType::PowerNode),
            "wind_turbine" => Some(BuildingType::WindTurbine),
            "server_bank" => Some(BuildingType::ServerBank),
            "sweeper" => Some(BuildingType::Sweeper),
            "storage" => Some(BuildingType::Storage),
            "biomass_harvester" => Some(BuildingType::BiomassHarvester),
            "smelter" => Some(BuildingType::Smelter),
            "assembler" => Some(BuildingType::Assembler),
            "heater_node" => Some(BuildingType::HeaterNode),
            "shield_generator" => Some(BuildingType::ShieldGenerator),
            "mass_driver" => Some(BuildingType::MassDriver),
            "landing_pad" => Some(BuildingType::LandingPad),
            _ => None,
        }
    }

    fn def(&self) -> &'static data::BuildingDef {
        data::game_data().building(self.id())
    }

    /// Resource cost to build
    pub fn cost(&self) -> (f32, f32) {
        let def = self.def();
        (def.cost.minerals, def.cost.energy)
    }

    /// Display name for UI
    pub fn name(&self) -> &'static str {
        self.def().name.as_str()
    }

    /// Keyboard shortcut for quick selection
    pub fn hotkey(&self) -> Option<char> {
        self.def()
            .hotkey
            .as_ref()
            .and_then(|key| key.chars().next())
    }

    /// Short description for UI
    pub fn description(&self) -> &'static str {
        self.def().description.as_str()
    }

    /// Net power per second (positive = generation, negative = consumption)
    pub fn power_delta(&self) -> f32 {
        let def = self.def();
        def.power_generation - def.power_consumption
    }
}

#[cfg(test)]
mod tests;
