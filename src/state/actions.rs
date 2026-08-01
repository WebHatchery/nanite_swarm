//! Player-initiated actions: placing, selling, and harvesting

use crate::engine::{BuildingType, GridPos, TerrainType};

use super::game_state::PlacementAnim;
use super::game_state::PlanetState;

impl PlanetState {
    /// Try to place a building at position
    pub fn try_place_building(&mut self, pos: GridPos) -> bool {
        if let Some(building_type) = self.selected_building {
            if !self.is_building_unlocked(building_type) {
                return false;
            }
            let (mineral_cost, energy_cost) = building_type.cost();

            if !self.resources.can_afford(mineral_cost, energy_cost) {
                return false;
            }

            if self.grid.place_building(pos, building_type) {
                self.resources.spend(mineral_cost, energy_cost);

                // Spawn initial drone for drills
                if building_type == BuildingType::Drill {
                    self.drones.spawn_drone(pos);
                    self.drill_buffers.insert((pos.x, pos.y), 0.0);
                }

                // Reveal area around new building
                self.grid.reveal_around(pos, 3);

                // Update power grid
                self.grid.update_power_grid();
                self.power_balance = self.net_power();
                self.update_achievements();

                self.placement_anims.push(PlacementAnim {
                    position: pos,
                    timer: 0.3,
                });
                self.spawn_place_burst(pos);

                return true;
            }
        }
        false
    }

    pub fn try_place_conduit_path(&mut self, from: GridPos, to: GridPos) -> bool {
        if !self.is_building_unlocked(BuildingType::Conduit) {
            return false;
        }
        let Some(path) = self.grid.find_conduit_path(from, to) else {
            return false;
        };

        let mut placed_any = false;
        for pos in path {
            let Some(tile) = self.grid.get(pos) else {
                continue;
            };
            if tile
                .building
                .as_ref()
                .map(|b| b.building_type == BuildingType::Conduit)
                .unwrap_or(false)
            {
                continue;
            }

            let (mineral_cost, energy_cost) = BuildingType::Conduit.cost();
            if !self.resources.can_afford(mineral_cost, energy_cost) {
                break;
            }

            if self.grid.place_building(pos, BuildingType::Conduit) {
                self.resources.spend(mineral_cost, energy_cost);
                self.grid.reveal_around(pos, 3);
                placed_any = true;
            }
        }

        if placed_any {
            self.grid.update_power_grid();
            self.power_balance = self.net_power();
        }

        placed_any
    }

    pub fn try_sell_building(&mut self, pos: GridPos) -> bool {
        let Some(tile) = self.grid.get(pos) else {
            return false;
        };
        let Some(building) = tile.building.as_ref() else {
            return false;
        };
        if building.building_type == BuildingType::Core {
            return false;
        }

        let building_type = building.building_type;
        let (mineral_cost, energy_cost) = building_type.cost();
        let refund_ratio = 0.5;

        if let Some(removed) = self.grid.remove_building(pos) {
            if removed.building_type == BuildingType::Drill {
                self.drill_buffers.remove(&(pos.x, pos.y));
                self.drones.remove_drones_at_drill(pos);
            }

            self.resources.minerals += mineral_cost * refund_ratio;
            self.resources.energy += energy_cost * refund_ratio;

            self.grid.update_power_grid();
            self.power_balance = self.net_power();
            return true;
        }

        false
    }

    /// Try to harvest terrain at position
    pub fn try_harvest_terrain(&mut self, pos: GridPos) -> bool {
        if let Some(tile) = self.grid.get(pos) {
            if !tile.revealed || !tile.terrain.is_harvestable() || tile.building.is_some() {
                return false;
            }

            let terrain = tile.terrain;
            let (minerals, biomass) = terrain.harvest_rewards();

            // Apply harvest
            if let Some(tile) = self.grid.get_mut(pos) {
                tile.terrain = terrain.harvested();
                match terrain {
                    TerrainType::Mountain => {
                        tile.mountain_harvested = true;
                    }
                    TerrainType::Forest => {
                        tile.forest_cleared = true;
                        tile.biomass_amount = 0.0;
                        self.forest_harvested_count += 1;
                    }
                    _ => {}
                }
            }

            self.resources.minerals += minerals;
            self.resources.biomass += biomass;

            true
        } else {
            false
        }
    }

    /// Check if terrain at position can be harvested
    pub fn can_harvest(&self, pos: GridPos) -> bool {
        if let Some(tile) = self.grid.get(pos) {
            tile.revealed && tile.terrain.is_harvestable() && tile.building.is_none()
        } else {
            false
        }
    }

    pub fn try_convert_forest_to_filter(&mut self, pos: GridPos) -> bool {
        if let Some(tile) = self.grid.get(pos) {
            if !tile.revealed || tile.terrain != TerrainType::Forest || tile.building.is_some() {
                return false;
            }
        } else {
            return false;
        }

        if let Some(tile) = self.grid.get_mut(pos) {
            tile.terrain = TerrainType::Rough;
            tile.filter = true;
            tile.forest_cleared = true;
            tile.biomass_amount = 0.0;
            self.forest_harvested_count += 1;
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::GameConfig;

    fn state() -> PlanetState {
        PlanetState::new("Test", 24, 24, 42, GameConfig::default())
    }

    #[test]
    fn try_place_building_spends_resources_and_spawns_drone() {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        let before_minerals = state.resources.minerals;

        assert!(state.try_place_building(pos));
        assert!(state.resources.minerals < before_minerals);
        assert_eq!(state.drones.total_count(), 1);
        assert!(state.drill_buffers.contains_key(&(pos.x, pos.y)));
    }

    #[test]
    fn try_place_building_fails_when_unaffordable() {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.resources.minerals = 0.0;

        assert!(!state.try_place_building(pos));
        assert_eq!(state.drones.total_count(), 0);
    }

    #[test]
    fn try_sell_building_refunds_half_cost_and_cannot_sell_core() {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        assert!(!state.try_sell_building(core));

        let pos = GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);
        let minerals_after_build = state.resources.minerals;

        assert!(state.try_sell_building(pos));
        assert!(state.resources.minerals > minerals_after_build);
        assert!(state.grid.get(pos).unwrap().building.is_none());
        assert_eq!(state.drones.total_count(), 0);
    }

    #[test]
    fn try_harvest_terrain_converts_mountain_and_grants_minerals() {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 5, core.y);
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Mountain;
        state.grid.reveal_around(pos, 1);
        let before = state.resources.minerals;

        assert!(state.can_harvest(pos));
        assert!(state.try_harvest_terrain(pos));
        assert!(state.resources.minerals > before);
        let tile = state.grid.get(pos).unwrap();
        assert_eq!(tile.terrain, TerrainType::Rough);
        assert!(tile.mountain_harvested);
        assert!(!state.can_harvest(pos));
    }

    #[test]
    fn try_harvest_terrain_fails_on_unrevealed_tile() {
        let mut state = state();
        let far_pos = GridPos::new(0, 0);
        state.grid.get_mut(far_pos).unwrap().terrain = TerrainType::Mountain;
        assert!(!state.can_harvest(far_pos));
        assert!(!state.try_harvest_terrain(far_pos));
    }

    #[test]
    fn try_convert_forest_to_filter_requires_forest_terrain() {
        let mut state = state();
        let core = state.grid.find_core().unwrap();
        let pos = GridPos::new(core.x + 5, core.y);
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Forest;
        state.grid.reveal_around(pos, 1);

        assert!(state.try_convert_forest_to_filter(pos));
        let tile = state.grid.get(pos).unwrap();
        assert!(tile.filter);
        assert!(tile.forest_cleared);
        assert_eq!(tile.terrain, TerrainType::Rough);

        // Already converted: no longer forest, so a second attempt fails.
        assert!(!state.try_convert_forest_to_filter(pos));
    }
}
