//! Player-initiated actions: placing, selling, and harvesting

use crate::engine::{BuildingType, GridPos, TerrainType};

use super::game_state::PlacementAnim;
use super::game_state::PlanetState;

mod batch;
mod harvest;
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
