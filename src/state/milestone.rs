//! Things about a world the game knows how to measure.
//!
//! One vocabulary, shared: an achievement fires on one of these, and a Core
//! stage is reached by meeting several at once. Adding either is a line of
//! JSON as long as it asks about something already in here; asking about
//! something new is the only part that is code.

use crate::engine::BuildingType;

use super::game_state::PlanetState;

/// Something measurable about the world in front of the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Milestone {
    /// Announced by code at the moment it happens, not measured from state.
    Manual,
    Drills,
    ServerBanks,
    Structures,
    NetworkTiles,
    Drones,
    MineralsHeld,
    DataHeld,
    AlloyHeld,
    Technologies,
    ForestsHarvested,
    PowerSurplus,
    SeedShipStages,
}

impl Milestone {
    pub fn id(self) -> &'static str {
        match self {
            Milestone::Manual => "manual",
            Milestone::Drills => "drills",
            Milestone::ServerBanks => "server_banks",
            Milestone::Structures => "structures",
            Milestone::NetworkTiles => "network_tiles",
            Milestone::Drones => "drones",
            Milestone::MineralsHeld => "minerals_held",
            Milestone::DataHeld => "data_held",
            Milestone::AlloyHeld => "alloy_held",
            Milestone::Technologies => "technologies",
            Milestone::ForestsHarvested => "forests_harvested",
            Milestone::PowerSurplus => "power_surplus",
            Milestone::SeedShipStages => "seed_ship_stages",
        }
    }

    pub const ALL: [Milestone; 13] = [
        Milestone::Manual,
        Milestone::Drills,
        Milestone::ServerBanks,
        Milestone::Structures,
        Milestone::NetworkTiles,
        Milestone::Drones,
        Milestone::MineralsHeld,
        Milestone::DataHeld,
        Milestone::AlloyHeld,
        Milestone::Technologies,
        Milestone::ForestsHarvested,
        Milestone::PowerSurplus,
        Milestone::SeedShipStages,
    ];

    pub fn from_id(id: &str) -> Option<Self> {
        Milestone::ALL
            .into_iter()
            .find(|condition| condition.id() == id)
    }
}

impl PlanetState {
    /// Where this world currently stands against a milestone, in whatever it
    /// is counted in.
    ///
    /// Separate from [`PlanetState::meets`] because "how far along is this"
    /// is a different question from "is it done", and a records screen with
    /// only the second one is a list of padlocks.
    pub fn measure(&self, milestone: Milestone) -> f32 {
        match milestone {
            // Announced by code at the moment it happens; never read off state.
            Milestone::Manual => 0.0,
            Milestone::Drills => self.count_of(BuildingType::Drill),
            Milestone::ServerBanks => self.count_of(BuildingType::ServerBank),
            Milestone::Structures => self.grid.total_buildings() as f32,
            Milestone::NetworkTiles => self.network_tile_count() as f32,
            Milestone::Drones => self.drones.total_count() as f32,
            Milestone::MineralsHeld => self.resources.minerals,
            Milestone::DataHeld => self.resources.data,
            Milestone::AlloyHeld => self.resources.alloy,
            Milestone::Technologies => self.research.unlocked_techs.len() as f32,
            Milestone::ForestsHarvested => self.forest_harvested_count as f32,
            Milestone::PowerSurplus => self.power_balance.max(0.0),
            Milestone::SeedShipStages => self.seed_ship.stage_index() as f32,
        }
    }

    /// Whether this world currently meets a milestone.
    pub fn meets(&self, milestone: Milestone, target: f32) -> bool {
        match milestone {
            Milestone::Manual => false,
            _ => self.measure(milestone) >= target,
        }
    }

    fn count_of(&self, building_type: BuildingType) -> f32 {
        self.grid.find_buildings(building_type).len() as f32
    }

    /// Tiles that carry drones, which is the size of the network the player
    /// has actually laid.
    fn network_tile_count(&self) -> usize {
        self.grid
            .iter_tiles()
            .filter(|(_, tile)| {
                tile.building
                    .as_ref()
                    .is_some_and(|building| building.carries_traffic())
            })
            .count()
    }
}
