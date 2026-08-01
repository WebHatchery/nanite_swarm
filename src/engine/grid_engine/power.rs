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
                (base * b.dust_power_consumption_multiplier()) + leak
            })
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::super::building_type::BuildingType;
    use super::super::grid::Grid;
    use super::super::grid_pos::GridPos;

    fn grid_with_core(width: u32, height: u32) -> (Grid, GridPos) {
        let mut grid = Grid::new(width, height);
        let core_pos = GridPos::new(width as i32 / 2, height as i32 / 2);
        // Reveal the whole grid so placement in these tests is never blocked by fog of war.
        grid.reveal_around(core_pos, width + height);
        grid.place_building(core_pos, BuildingType::Core);
        (grid, core_pos)
    }

    #[test]
    fn conduit_adjacent_to_core_is_powered() {
        let (mut grid, core_pos) = grid_with_core(6, 6);
        let conduit_pos = GridPos::new(core_pos.x + 1, core_pos.y);
        grid.place_building(conduit_pos, BuildingType::Conduit);
        grid.update_power_grid();
        assert!(
            grid.get(conduit_pos)
                .unwrap()
                .building
                .as_ref()
                .unwrap()
                .powered
        );
    }

    #[test]
    fn building_beyond_repeater_range_is_unpowered_without_a_node() {
        let (mut grid, core_pos) = grid_with_core(20, 4);
        // Lay a chain of conduits far past the repeater range with no PowerNode.
        let far_x = core_pos.x + grid.repeater_range as i32 + 3;
        for x in (core_pos.x + 1)..=far_x {
            grid.place_building(GridPos::new(x, core_pos.y), BuildingType::Conduit);
        }
        grid.update_power_grid();
        let far_pos = GridPos::new(far_x, core_pos.y);
        assert!(
            !grid
                .get(far_pos)
                .unwrap()
                .building
                .as_ref()
                .unwrap()
                .powered
        );
    }

    #[test]
    fn power_node_repeater_extends_range() {
        let (mut grid, core_pos) = grid_with_core(30, 4);
        let range = grid.repeater_range as i32;
        for x in (core_pos.x + 1)..core_pos.x + range {
            grid.place_building(GridPos::new(x, core_pos.y), BuildingType::Conduit);
        }
        let node_pos = GridPos::new(core_pos.x + range, core_pos.y);
        grid.place_building(node_pos, BuildingType::PowerNode);
        let far_x = node_pos.x + range;
        for x in (node_pos.x + 1)..=far_x {
            grid.place_building(GridPos::new(x, core_pos.y), BuildingType::Conduit);
        }
        grid.update_power_grid();
        let far_pos = GridPos::new(far_x, core_pos.y);
        assert!(
            grid.get(far_pos)
                .unwrap()
                .building
                .as_ref()
                .unwrap()
                .powered
        );
    }

    #[test]
    fn dust_stalled_building_does_not_transmit_power() {
        let (mut grid, core_pos) = grid_with_core(6, 6);
        let conduit_pos = GridPos::new(core_pos.x + 1, core_pos.y);
        let beyond_pos = GridPos::new(core_pos.x + 2, core_pos.y);
        grid.place_building(conduit_pos, BuildingType::Conduit);
        grid.place_building(beyond_pos, BuildingType::Conduit);
        grid.get_mut(conduit_pos)
            .unwrap()
            .building
            .as_mut()
            .unwrap()
            .dust = 100.0;
        grid.update_power_grid();
        assert!(
            !grid
                .get(beyond_pos)
                .unwrap()
                .building
                .as_ref()
                .unwrap()
                .powered
        );
    }

    #[test]
    fn the_core_generates_and_a_connected_drill_consumes() {
        let (mut grid, core_pos) = grid_with_core(6, 6);
        let drill_pos = GridPos::new(core_pos.x + 1, core_pos.y);
        grid.place_building(drill_pos, BuildingType::Drill);
        grid.update_power_grid();
        // Core generates power, drill consumes it once connected.
        assert!(grid.total_power_generation() > 0.0);
        assert!(grid.total_power_consumption() > 0.0);
    }
}
