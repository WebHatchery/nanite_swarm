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
                    self.output_buffers.insert((pos.x, pos.y), 0.0);
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
                self.drones.remove_drones_at(pos);
            }
            // Whatever was on the pad, in the hopper or half-loaded into a pod
            // goes down with the building. Leaving it behind means the next
            // thing built here inherits a stranger's cargo.
            self.output_buffers.remove(&(pos.x, pos.y));
            self.input_buffers.remove(&(pos.x, pos.y));
            self.pod_loads.remove(&(pos.x, pos.y));
            self.pad_cargo.remove(&(pos.x, pos.y));

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
            // What the ground is worth is terrain data; what the swarm gets out
            // of it is a stat, so a tech can pay off on ground already surveyed.
            let yield_multiplier = self.stats.apply(crate::engine::StatId::HarvestYield, 1.0);
            let (minerals, biomass) = (minerals * yield_multiplier, biomass * yield_multiplier);

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
mod tests;
