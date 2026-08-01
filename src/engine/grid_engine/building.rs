//! A building instance placed on the grid

use crate::data;

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

    pub fn dust_drone_speed_multiplier(&self) -> f32 {
        if self.dust >= 50.0 {
            0.7
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

    pub fn is_dust_stalled(&self) -> bool {
        self.dust >= 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drill_at_origin() -> Building {
        Building::new(BuildingType::Drill, GridPos::new(0, 0))
    }

    #[test]
    fn new_building_starts_unpowered_unless_core() {
        let drill = drill_at_origin();
        assert!(!drill.powered);
        assert!(!drill.connected_to_core);

        let core = Building::new(BuildingType::Core, GridPos::new(1, 1));
        assert!(core.powered);
        assert!(core.connected_to_core);
    }

    #[test]
    fn dust_efficiency_degrades_in_steps() {
        let mut building = drill_at_origin();
        building.dust = 0.0;
        assert_eq!(building.dust_efficiency(), 1.0);
        building.dust = 25.0;
        assert_eq!(building.dust_efficiency(), 0.9);
        building.dust = 100.0;
        assert_eq!(building.dust_efficiency(), 0.0);
    }

    #[test]
    fn dust_drone_speed_multiplier_slows_at_50() {
        let mut building = drill_at_origin();
        building.dust = 49.0;
        assert_eq!(building.dust_drone_speed_multiplier(), 1.0);
        building.dust = 50.0;
        assert_eq!(building.dust_drone_speed_multiplier(), 0.7);
    }

    #[test]
    fn dust_stalled_only_at_100() {
        let mut building = drill_at_origin();
        building.dust = 99.9;
        assert!(!building.is_dust_stalled());
        building.dust = 100.0;
        assert!(building.is_dust_stalled());
    }

    #[test]
    fn dust_power_leak_only_for_transmitters_over_75() {
        let mut conduit = Building::new(BuildingType::Conduit, GridPos::new(0, 0));
        conduit.dust = 80.0;
        assert!(conduit.transmits_power());
        assert_eq!(conduit.dust_power_leak(), 0.5);

        let mut drill = drill_at_origin();
        drill.dust = 80.0;
        assert!(!drill.transmits_power());
        assert_eq!(drill.dust_power_leak(), 0.0);
    }
}
