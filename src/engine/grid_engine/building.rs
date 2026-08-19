//! A building instance placed on the grid

use crate::data;
use crate::data::DustResponseConfig;

use super::building_type::BuildingType;
use super::grid_pos::GridPos;
use serde::{Deserialize, Serialize};

/// A building placed on the grid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Building {
    pub building_type: BuildingType,
    pub position: GridPos,
    pub powered: bool,
    pub efficiency: f32,         // 0.0 to 1.0+
    pub connected_to_core: bool, // For logistics validation
    #[serde(default)]
    pub dust: f32, // 0.0 to 100.0
    #[serde(default)]
    pub acid_wear: f32,
    #[serde(default)]
    pub heat: f32,
    /// Player-selected high-throughput mode for recipe buildings.
    #[serde(default)]
    pub overclocked: bool,
    /// Player-selected claim on scarce routed recipe inputs.
    #[serde(default)]
    pub input_priority: bool,
}

impl Building {
    pub fn new(building_type: BuildingType, position: GridPos) -> Self {
        let is_core = building_type == BuildingType::Core;
        Self {
            building_type,
            position,
            powered: is_core,
            efficiency: 1.0,
            connected_to_core: is_core,
            dust: 0.0,
            acid_wear: 0.0,
            heat: 0.0,
            overclocked: false,
            input_priority: false,
        }
    }

    /// Check if this building transmits power
    pub fn transmits_power(&self) -> bool {
        data::game_data()
            .building(self.building_type.id())
            .transmits_power
    }

    /// Check if drones may route through this building's tile
    pub fn carries_traffic(&self) -> bool {
        data::game_data()
            .building(self.building_type.id())
            .carries_traffic
    }

    /// Check if this building generates power
    pub fn generates_power(&self) -> bool {
        data::game_data()
            .building(self.building_type.id())
            .generates_power
    }

    /// Check if this building consumes power
    pub fn consumes_power(&self) -> bool {
        data::game_data()
            .building(self.building_type.id())
            .consumes_power
    }

    pub fn dust_efficiency(&self) -> f32 {
        if self.dust >= 100.0 {
            0.0
        } else if self.dust >= 25.0 {
            0.9
        } else {
            1.0
        }
    }

    pub fn dust_efficiency_with(&self, response: &DustResponseConfig) -> f32 {
        if self.dust >= response.stall_threshold {
            0.0
        } else if self.dust >= response.efficiency_threshold {
            response.efficiency
        } else {
            1.0
        }
    }

    pub fn dust_drone_speed_multiplier(&self) -> f32 {
        if self.dust >= 50.0 {
            0.7
        } else {
            1.0
        }
    }

    pub fn dust_drone_speed_multiplier_with(&self, response: &DustResponseConfig) -> f32 {
        if self.dust >= response.speed_threshold {
            response.speed_multiplier
        } else {
            1.0
        }
    }

    pub fn dust_power_generation_multiplier(&self) -> f32 {
        if self.dust >= 100.0 {
            0.0
        } else if self.dust >= 75.0 {
            0.7
        } else {
            1.0
        }
    }

    pub fn dust_power_consumption_multiplier(&self) -> f32 {
        if self.dust >= 100.0 {
            0.0
        } else if self.dust >= 75.0 {
            1.2
        } else {
            1.0
        }
    }

    pub fn dust_power_leak(&self) -> f32 {
        if self.dust >= 75.0 && self.transmits_power() {
            0.5
        } else {
            0.0
        }
    }

    pub fn dust_power_leak_with(&self, response: &DustResponseConfig) -> f32 {
        if self.dust >= response.leak_threshold && self.transmits_power() {
            response.leak
        } else {
            0.0
        }
    }

    pub fn is_dust_stalled(&self) -> bool {
        self.dust >= 100.0
    }

    pub fn is_dust_stalled_with(&self, response: &DustResponseConfig) -> bool {
        self.dust >= response.stall_threshold
    }

    pub fn is_overheated(&self, limit: f32) -> bool {
        self.heat >= limit.max(0.0)
    }

    pub fn supports_overclock(&self) -> bool {
        !data::game_data()
            .building(self.building_type.id())
            .recipe
            .is_empty()
    }

    pub fn work_multiplier(&self) -> f32 {
        if self.overclocked && self.supports_overclock() {
            1.5
        } else {
            1.0
        }
    }

    pub fn power_demand_multiplier(&self) -> f32 {
        if self.overclocked && self.supports_overclock() {
            1.75
        } else {
            1.0
        }
    }

    pub fn dust_accumulation_multiplier(&self) -> f32 {
        if self.overclocked && self.supports_overclock() {
            1.8
        } else {
            1.0
        }
    }
}

#[cfg(test)]
mod tests;
