//! Power grid simulation: flood-fill connectivity and generation/consumption totals

use crate::data;

use super::building_type::BuildingType;
use super::grid::Grid;
use super::grid_pos::GridPos;

impl Grid {
    /// Update power grid connectivity using flood fill from Core
    pub fn update_power_grid(&mut self) {
        // First, reset all buildings to unpowered
        for (_, tile) in self.iter_tiles_mut() {
            if let Some(ref mut building) = tile.building {
                let is_core = building.building_type == BuildingType::Core;
                let stalled = building.is_dust_stalled();
                building.powered = is_core && !stalled;
                building.connected_to_core = is_core && !stalled;
            }
        }

        // Find Core position
        let core_pos = match self.find_core() {
            Some(pos) => pos,
            None => return,
        };

        // Flood fill from Core through power-transmitting buildings with repeater range
        let mut best_distance: std::collections::HashMap<GridPos, u32> =
            std::collections::HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((core_pos, 0u32));
        best_distance.insert(core_pos, 0u32);

        while let Some((pos, distance_since_repeater)) = queue.pop_front() {
            // Mark this building as connected and powered
            if let Some(tile) = self.get_mut(pos) {
                if let Some(ref mut building) = tile.building {
                    if !building.is_dust_stalled() {
                        building.connected_to_core = true;
                        building.powered = true;
                    }
                }
            }

            let next_distance = if let Some(tile) = self.get(pos) {
                if let Some(ref building) = tile.building {
                    if building.is_dust_stalled() {
                        distance_since_repeater + 1
                    } else if matches!(
                        building.building_type,
                        BuildingType::Core | BuildingType::PowerNode
                    ) {
                        0
                    } else {
                        distance_since_repeater + 1
                    }
                } else {
                    distance_since_repeater + 1
                }
            } else {
                distance_since_repeater + 1
            };

            // Check neighbors
            for neighbor in pos.neighbors() {
                if !neighbor.in_bounds(self.width, self.height) {
                    continue;
                }

                // Check if neighbor has a power-transmitting building
                if let Some(tile) = self.get(neighbor) {
                    if let Some(ref building) = tile.building {
                        if building.transmits_power()
                            && !building.is_dust_stalled()
                            && next_distance <= self.repeater_range
                        {
                            let should_visit = match best_distance.get(&neighbor) {
                                Some(existing) => next_distance < *existing,
                                None => true,
                            };
                            if should_visit {
                                best_distance.insert(neighbor, next_distance);
                                queue.push_back((neighbor, next_distance));
                            }
                        }
                    }
                }
            }
        }

        // Now mark buildings adjacent to powered conduits/nodes as powered
        let powered_positions: Vec<GridPos> = self
            .iter_tiles()
            .filter_map(|(pos, tile)| {
                tile.building
                    .as_ref()
                    .filter(|b| b.powered && b.transmits_power() && !b.is_dust_stalled())
                    .map(|_| pos)
            })
            .collect();

        for powered_pos in powered_positions {
            for neighbor in powered_pos.neighbors() {
                if let Some(tile) = self.get_mut(neighbor) {
                    if let Some(ref mut building) = tile.building {
                        if !building.transmits_power() && !building.is_dust_stalled() {
                            building.powered = true;
                            building.connected_to_core = true;
                        }
                    }
                }
            }
        }
    }

    /// Check if position is adjacent to a powered building
    pub fn is_adjacent_to_power(&self, pos: GridPos) -> bool {
        for neighbor in pos.neighbors() {
            if let Some(tile) = self.get(neighbor) {
                if let Some(ref building) = tile.building {
                    if building.powered && building.transmits_power() {
                        return true;
                    }
                }
            }
        }
        false
    }

    /// Get total power generation
    pub fn total_power_generation(&self) -> f32 {
        self.iter_tiles()
            .filter_map(|(_, tile)| tile.building.as_ref())
            .filter(|b| b.powered && b.generates_power())
            .map(|b| {
                let def = data::game_data().building(b.building_type.id());
                let mut generation = def.power_generation;
                if def.uses_efficiency {
                    generation *= b.efficiency;
                }
                generation * b.dust_power_generation_multiplier()
            })
            .sum()
    }

    /// Get total power consumption
    pub fn total_power_consumption(&self) -> f32 {
        self.iter_tiles()
            .filter_map(|(_, tile)| tile.building.as_ref())
            .filter(|b| b.powered)
            .map(|b| {
                let def = data::game_data().building(b.building_type.id());
                let base = def.power_consumption;
                let leak = b.dust_power_leak();
                (base * b.dust_power_consumption_multiplier() * b.power_demand_multiplier()) + leak
            })
            .sum()
    }
}

#[cfg(test)]
mod tests;
