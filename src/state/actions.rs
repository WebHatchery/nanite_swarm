//! Player-initiated actions: placing, selling, and harvesting

use crate::engine::{BuildingType, GridPos, TerrainType};

use super::game_state::PlacementAnim;
use super::game_state::PlanetState;

mod processor;

impl PlanetState {
    /// Try to place a building at position
    pub fn try_place_building(&mut self, pos: GridPos) -> bool {
        if let Some(building_type) = self.selected_building {
            if !self.is_building_unlocked(building_type) {
                return false;
            }
            if !self.constraints_allow_building(building_type) {
                self.notifications.warning("Planet constraint not met");
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
                self.network_revision = self.network_revision.wrapping_add(1);
                self.power_balance = self.net_power();
                self.update_achievements();
                if matches!(
                    building_type,
                    BuildingType::Drill
                        | BuildingType::Smelter
                        | BuildingType::Assembler
                        | BuildingType::MassDriver
                ) && self
                    .grid
                    .find_core()
                    .and_then(|core| crate::engine::route_over_network(&self.grid, pos, core))
                    .is_none()
                {
                    self.notifications
                        .warning("Producer placed on open ground - connect a conduit route");
                }

                self.placement_anims.push(PlacementAnim {
                    position: pos,
                    timer: 0.3,
                });
                self.spawn_place_burst(pos);
                self.emit_audio(super::audio::AudioEvent::Placement);
                self.undo_history
                    .push(super::game_state::UndoEntry::Placed(pos));

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
                self.undo_history
                    .push(super::game_state::UndoEntry::Placed(pos));
                placed_any = true;
            }
        }

        if placed_any {
            self.grid.update_power_grid();
            self.power_balance = self.net_power();
            self.network_revision = self.network_revision.wrapping_add(1);
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
            self.input_hoppers.remove(&(pos.x, pos.y));
            self.pod_loads.remove(&(pos.x, pos.y));
            self.pad_cargo.remove(&(pos.x, pos.y));
            self.drone_queues.retain(|_, queue| {
                queue.retain(|id| self.drones.drones().iter().any(|drone| drone.id == *id));
                true
            });
            self.route_reservations.clear();

            self.resources.minerals += mineral_cost * refund_ratio;
            self.resources.energy += energy_cost * refund_ratio;

            self.grid.update_power_grid();
            self.power_balance = self.net_power();
            self.network_revision = self.network_revision.wrapping_add(1);
            self.emit_audio(super::audio::AudioEvent::Demolition);
            self.undo_history
                .push(super::game_state::UndoEntry::Removed(
                    building_type,
                    pos,
                    removed.overclocked,
                    removed.input_priority,
                    removed.standby,
                ));
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
            self.spawn_harvest_burst(pos);
            self.emit_audio(super::audio::AudioEvent::Harvest);

            self.notifications.info(format!(
                "Harvested {:.0} minerals and {:.0} biomass",
                minerals, biomass
            ));

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

    pub fn save_blueprint(&mut self, anchor: GridPos, positions: &[GridPos]) -> bool {
        let entries: Vec<_> = positions
            .iter()
            .filter_map(|pos| {
                self.grid
                    .get(*pos)
                    .and_then(|tile| tile.building.as_ref())
                    .map(|building| super::game_state::BlueprintEntry {
                        offset: (pos.x - anchor.x, pos.y - anchor.y),
                        building_type: building.building_type,
                        overclocked: building.overclocked,
                        input_priority: building.input_priority,
                        standby: building.standby,
                    })
            })
            .collect();
        if entries.is_empty() {
            return false;
        }
        self.blueprint = entries;
        self.notifications.info("Blueprint saved");
        true
    }

    pub fn begin_box_select(&mut self) {
        self.box_select_mode = true;
        self.box_select_start = None;
        self.box_selected.clear();
        self.bulk_purge_armed = false;
        self.notifications
            .info("Drag across the grid, or tap two opposite corners");
    }

    pub fn finish_box_select(&mut self, end: GridPos) {
        let Some(start) = self.box_select_start.take() else {
            self.box_select_start = Some(end);
            self.notifications.info("Tap the opposite corner");
            return;
        };
        let min_x = start.x.min(end.x);
        let max_x = start.x.max(end.x);
        let min_y = start.y.min(end.y);
        let max_y = start.y.max(end.y);
        self.box_selected = self
            .grid
            .iter_tiles()
            .filter_map(|(pos, tile)| {
                (pos.x >= min_x
                    && pos.x <= max_x
                    && pos.y >= min_y
                    && pos.y <= max_y
                    && tile.building.is_some())
                .then_some(pos)
            })
            .collect();
        self.box_select_mode = false;
        self.bulk_purge_armed = false;
        self.selected_tile = self.box_selected.first().copied();
        self.notifications
            .info(format!("Selected {} buildings", self.box_selected.len()));
    }

    pub fn set_selected_overclock(&mut self, enabled: bool) -> usize {
        if !self
            .research
            .unlocked_techs
            .iter()
            .any(|tech| tech == "adaptive_clocking")
        {
            self.notifications
                .warning("Research Adaptive Clocking to boost processors");
            return 0;
        }
        let selected = self.box_selected.clone();
        self.auto_clocking = false;
        let mut changed = 0;
        for pos in selected {
            let Some(building) = self
                .grid
                .get_mut(pos)
                .and_then(|tile| tile.building.as_mut())
            else {
                continue;
            };
            if building.supports_overclock() && building.overclocked != enabled {
                building.overclocked = enabled;
                changed += 1;
            }
        }
        self.power_balance = self.net_power();
        if changed > 0 {
            self.notifications.info(if enabled {
                format!("Boosted {} selected processors", changed)
            } else {
                format!("Normalized {} selected processors", changed)
            });
        } else {
            self.notifications.warning("No selected processors changed");
        }
        changed
    }

    pub fn place_blueprint(&mut self, anchor: GridPos) -> usize {
        let blueprint = self.blueprint.clone();
        let mut placed = 0;
        for entry in blueprint {
            let pos = GridPos::new(anchor.x + entry.offset.0, anchor.y + entry.offset.1);
            self.select_building(entry.building_type);
            if self.try_place_building(pos) {
                if entry.overclocked {
                    if let Some(building) = self
                        .grid
                        .get_mut(pos)
                        .and_then(|tile| tile.building.as_mut())
                    {
                        building.overclocked = building.supports_overclock();
                    }
                }
                if entry.input_priority {
                    if let Some(building) = self
                        .grid
                        .get_mut(pos)
                        .and_then(|tile| tile.building.as_mut())
                    {
                        building.input_priority = building.supports_overclock();
                    }
                }
                if entry.standby {
                    if let Some(building) = self
                        .grid
                        .get_mut(pos)
                        .and_then(|tile| tile.building.as_mut())
                    {
                        building.standby = building.supports_overclock();
                    }
                }
                placed += 1;
            }
        }
        self.power_balance = self.net_power();
        if placed < self.blueprint.len() {
            self.notifications.warning(format!(
                "Blueprint placed {}/{}; invalid or unaffordable entries skipped",
                placed,
                self.blueprint.len()
            ));
        }
        placed
    }

    pub fn begin_relocation(&mut self, source: GridPos) -> bool {
        if self
            .grid
            .get(source)
            .and_then(|tile| tile.building.as_ref())
            .is_none()
        {
            return false;
        }
        self.relocation_source = Some(source);
        self.notifications
            .info("Relocation armed - tap a destination tile");
        true
    }

    pub fn relocate_building(&mut self, source: GridPos, destination: GridPos) -> bool {
        let Some((building_type, overclocked, input_priority, standby)) = self
            .grid
            .get(source)
            .and_then(|tile| tile.building.as_ref())
            .map(|building| {
                (
                    building.building_type,
                    building.overclocked,
                    building.input_priority,
                    building.standby,
                )
            })
        else {
            return false;
        };
        if !self.grid.can_place_building(destination, building_type) {
            self.notifications
                .warning("Relocation destination is invalid");
            return false;
        }
        if !self.try_sell_building(source) {
            return false;
        }
        self.select_building(building_type);
        let placed = self.try_place_building(destination);
        if placed && overclocked {
            if let Some(building) = self
                .grid
                .get_mut(destination)
                .and_then(|tile| tile.building.as_mut())
            {
                building.overclocked = building.supports_overclock();
            }
            self.power_balance = self.net_power();
        }
        if placed && input_priority {
            if let Some(building) = self
                .grid
                .get_mut(destination)
                .and_then(|tile| tile.building.as_mut())
            {
                building.input_priority = building.supports_overclock();
            }
        }
        if placed && standby {
            if let Some(building) = self
                .grid
                .get_mut(destination)
                .and_then(|tile| tile.building.as_mut())
            {
                building.standby = building.supports_overclock();
            }
        }
        placed
    }

    pub fn undo_last_action(&mut self) -> bool {
        let Some(action) = self.undo_history.pop() else {
            return false;
        };
        let success = match action {
            super::game_state::UndoEntry::Placed(pos) => self.try_sell_building(pos),
            super::game_state::UndoEntry::Removed(
                kind,
                pos,
                overclocked,
                input_priority,
                standby,
            ) => {
                self.select_building(kind);
                let restored = self.try_place_building(pos);
                if restored && overclocked {
                    if let Some(building) = self
                        .grid
                        .get_mut(pos)
                        .and_then(|tile| tile.building.as_mut())
                    {
                        building.overclocked = building.supports_overclock();
                    }
                    self.power_balance = self.net_power();
                }
                if restored && input_priority {
                    if let Some(building) = self
                        .grid
                        .get_mut(pos)
                        .and_then(|tile| tile.building.as_mut())
                    {
                        building.input_priority = building.supports_overclock();
                    }
                }
                if restored && standby {
                    if let Some(building) = self
                        .grid
                        .get_mut(pos)
                        .and_then(|tile| tile.building.as_mut())
                    {
                        building.standby = building.supports_overclock();
                    }
                }
                restored
            }
        };
        if success {
            self.undo_history.pop();
            self.notifications.info("Last placement action undone");
        }
        success
    }

    pub fn toggle_building_overclock(&mut self, pos: GridPos) -> bool {
        let supports_overclock = self
            .grid
            .get(pos)
            .and_then(|tile| tile.building.as_ref())
            .is_some_and(|building| building.supports_overclock());
        if !supports_overclock {
            return false;
        }
        if !self
            .research
            .unlocked_techs
            .iter()
            .any(|tech| tech == "adaptive_clocking")
        {
            self.notifications
                .warning("Research Adaptive Clocking to boost processors");
            return false;
        }
        let Some(building) = self
            .grid
            .get_mut(pos)
            .and_then(|tile| tile.building.as_mut())
        else {
            return false;
        };
        let manual_override = std::mem::replace(&mut self.auto_clocking, false);
        building.overclocked = !building.overclocked;
        let enabled = building.overclocked;
        let name = building.building_type.name();
        self.power_balance = self.net_power();
        self.notifications.info(if enabled {
            format!(
                "{}{} boost: 1.5x work, 1.75x power",
                if manual_override {
                    "Auto Clock OFF; "
                } else {
                    ""
                },
                name
            )
        } else {
            format!(
                "{}{} returned to normal output",
                if manual_override {
                    "Auto Clock OFF; "
                } else {
                    ""
                },
                name
            )
        });
        true
    }
}

#[cfg(test)]
mod tests;
