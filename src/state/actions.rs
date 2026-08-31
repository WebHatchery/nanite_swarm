//! Player-initiated actions: placing, selling, and harvesting

use crate::engine::{BuildingType, GridPos, TerrainType};

use super::game_state::PlacementAnim;
use super::game_state::PlanetState;

mod placement;
mod processor;
mod relocation;

pub use placement::*;

impl PlanetState {
    /// Try to place a building at position.
    ///
    /// Placement is also reached by drag, blueprint, and undo paths, so this
    /// method owns the same first-actionable failure explanation as the map
    /// preview. Keeping the guard here prevents one input path from silently
    /// failing while another reports the same invalid tile.
    pub fn try_place_building(&mut self, pos: GridPos) -> bool {
        let Some(building_type) = self.selected_building else {
            return false;
        };
        if let Some(reason) = self.placement_block_reason(pos, building_type) {
            self.notifications
                .warning(reason.message(self, building_type));
            return false;
        }
        let (mineral_cost, energy_cost) = building_type.cost(self.data());

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
            self.refresh_power_grid();
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
                .and_then(|core| {
                    let dust_response = self.resolved_dust_response();
                    crate::engine::route_over_network_with(&self.grid, pos, core, &dust_response)
                })
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
        false
    }

    pub fn try_place_conduit_path(&mut self, from: GridPos, to: GridPos) -> bool {
        if !self.is_building_unlocked(BuildingType::Conduit) {
            return false;
        }
        let Some(mut path) = self.grid.find_conduit_path(from, to) else {
            let should_warn = self.route_path_blocked_at != Some(to);
            self.route_path_blocked_at = Some(to);
            if should_warn {
                self.notifications
                    .warning(self.conduit_route_block_message(from, to));
            }
            return false;
        };
        self.route_path_blocked_at = None;

        // Touch route gestures can begin on empty ground. Include that first
        // tile here so the route action has one owner and cannot warn once for
        // a failed first placement and again for the next segment.
        if self
            .grid
            .get(from)
            .is_some_and(|tile| tile.building.is_none())
        {
            path.insert(0, from);
        }

        let mut placed_any = false;
        let mut resource_blocked_at = None;
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

            let (mineral_cost, energy_cost) = BuildingType::Conduit.cost(self.data());
            if !self.resources.can_afford(mineral_cost, energy_cost) {
                resource_blocked_at = Some(pos);
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
            self.refresh_power_grid();
            self.power_balance = self.net_power();
            self.network_revision = self.network_revision.wrapping_add(1);
        }

        if let Some(blocked_pos) = resource_blocked_at {
            let (mineral_cost, energy_cost) = BuildingType::Conduit.cost(self.data());
            let should_warn = self.route_resource_blocked_at != Some(blocked_pos);
            self.route_resource_blocked_at = Some(blocked_pos);
            if should_warn {
                self.notifications.warning(format!(
                    "Route paused: need M {:.0} and E {:.0} per conduit",
                    mineral_cost, energy_cost
                ));
            }
        } else if placed_any {
            self.route_resource_blocked_at = None;
        }

        placed_any
    }

    fn conduit_route_block_message(&self, from: GridPos, to: GridPos) -> &'static str {
        let Some(tile) = self.grid.get(to) else {
            return "Route blocked: destination is outside the surveyed grid";
        };
        if !tile.revealed {
            return "Route blocked: survey the destination first";
        }
        if tile.filter {
            return "Route blocked: route around the dust filter";
        }
        if matches!(tile.terrain, TerrainType::Water | TerrainType::Void)
            && !tile
                .building
                .as_ref()
                .is_some_and(|building| building.building_type == BuildingType::Bridge)
        {
            return "Route blocked: place a Bridge on the water or void gap first";
        }
        if let Some(tile) = self.grid.get(from) {
            if !tile.revealed {
                return "Route blocked: survey the starting tile first";
            }
            if tile.filter {
                return "Route blocked: route around the dust filter";
            }
        }
        "Route blocked: no open conduit path to destination"
    }

    /// Lay a conduit route that starts at an existing building.
    ///
    /// Smart Drag is deliberately stricter than ordinary conduit placement:
    /// the gesture must begin on a building, so dragging across empty ground
    /// cannot accidentally spend resources on a route.
    pub fn try_place_smart_conduit_path(&mut self, from: GridPos, to: GridPos) -> bool {
        if self
            .grid
            .get(from)
            .and_then(|tile| tile.building.as_ref())
            .is_none()
        {
            return false;
        }

        self.try_place_conduit_path(from, to)
    }

    pub fn try_sell_building(&mut self, pos: GridPos) -> bool {
        let Some(tile) = self.grid.get(pos) else {
            return false;
        };
        let Some(building) = tile.building.as_ref() else {
            return false;
        };
        if building.building_type == BuildingType::Core {
            self.notifications.warning("The Core cannot be demolished");
            return false;
        }

        let building_type = building.building_type;
        let (mineral_cost, energy_cost) = building_type.cost(self.data());
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

            self.refresh_power_grid();
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
        let data_context = self.data.clone();
        let data = data_context.data();
        let Some(tile) = self.grid.get(pos) else {
            return false;
        };
        let terrain = tile.terrain;
        let Some((minerals, biomass)) = self.harvest_rewards_at(pos) else {
            return false;
        };

        // Apply harvest
        if let Some(tile) = self.grid.get_mut(pos) {
            tile.terrain = terrain.harvested(data);
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
    }

    /// Resolve the reward shown by the terrain inspector and paid by harvest.
    ///
    /// Research and surveyed feature effects modify the terrain's ordinary
    /// reward without changing the tile's eventual harvested state.
    pub fn harvest_rewards_at(&self, pos: GridPos) -> Option<(f32, f32)> {
        let data = self.data();
        let tile = self.grid.get(pos)?;
        if !tile.revealed || !tile.terrain.is_harvestable(data) || tile.building.is_some() {
            return None;
        }
        let (minerals, biomass) = tile.terrain.harvest_rewards(data);
        let yield_multiplier = self.stats.apply(crate::engine::StatId::HarvestYield, 1.0)
            * self.feature_harvest_multiplier_at(pos).unwrap_or(1.0);
        Some((minerals * yield_multiplier, biomass * yield_multiplier))
    }

    /// Check if terrain at position can be harvested
    pub fn can_harvest(&self, pos: GridPos) -> bool {
        if let Some(tile) = self.grid.get(pos) {
            tile.revealed && tile.terrain.is_harvestable(self.data()) && tile.building.is_none()
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
            let reduction = (1.0 - self.config.upkeep.filter_multiplier.clamp(0.0, 1.0)) * 100.0;
            self.emit_audio(super::audio::AudioEvent::UiConfirm);
            self.notifications.info(format!(
                "Forest converted: dust filter -{reduction:.0}% nearby"
            ));
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
            self.notifications
                .warning("Select at least one building before saving a blueprint");
            return false;
        }
        self.blueprint = entries;
        self.notifications.info("Blueprint saved");
        true
    }

    pub fn begin_box_select(&mut self) {
        self.selected_building = None;
        self.demolish_mode = false;
        self.relocation_source = None;
        self.box_select_start = None;
        self.bulk_purge_armed = false;
        self.purge_armed = None;
        self.drag_last_pos = None;
        self.smart_drag_source = None;
        self.box_select_mode = true;
        self.box_selected.clear();
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
        let mut standby_count = 0;
        for pos in selected {
            let Some(building) = self
                .grid
                .get_mut(pos)
                .and_then(|tile| tile.building.as_mut())
            else {
                continue;
            };
            if !building.supports_overclock() {
                continue;
            }
            if building.standby {
                standby_count += 1;
                continue;
            }
            if building.overclocked != enabled {
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
        } else if enabled && standby_count > 0 {
            self.notifications.warning(format!(
                "Resume {} paused processor{} before boosting",
                standby_count,
                if standby_count == 1 { "" } else { "s" }
            ));
        } else {
            self.notifications.warning("No selected processors changed");
        }
        changed
    }

    pub fn place_blueprint(&mut self, anchor: GridPos) -> usize {
        if self.blueprint.is_empty() {
            self.notifications
                .warning("Save a blueprint before stamping");
            return 0;
        }
        self.cancel_interaction_modes();
        let blueprint = self.blueprint.clone();
        let mut placed = 0;
        for entry in blueprint {
            let Some(x) = anchor.x.checked_add(entry.offset.0) else {
                continue;
            };
            let Some(y) = anchor.y.checked_add(entry.offset.1) else {
                continue;
            };
            let pos = GridPos::new(x, y);
            self.selected_building = Some(entry.building_type);
            if self.try_place_building(pos) {
                if let Some(building) = self
                    .grid
                    .get_mut(pos)
                    .and_then(|tile| tile.building.as_mut())
                {
                    let supports_policy = building.supports_overclock();
                    building.overclocked = entry.overclocked && supports_policy && !entry.standby;
                    building.input_priority = entry.input_priority && supports_policy;
                    building.standby = entry.standby && supports_policy;
                }
                placed += 1;
            }
        }
        self.cancel_interaction_modes();
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

    pub fn toggle_building_overclock(&mut self, pos: GridPos) -> bool {
        let data_context = self.data.clone();
        let data = data_context.data();
        let (supports_overclock, standby) = self
            .grid
            .get(pos)
            .and_then(|tile| tile.building.as_ref())
            .map(|building| (building.supports_overclock(), building.standby))
            .unwrap_or((false, false));
        if !supports_overclock {
            return false;
        }
        if standby {
            self.notifications
                .warning("Resume the processor before boosting it");
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
        let name = building.building_type.name(data).to_string();
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
