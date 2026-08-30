//! Relocation and reversible placement actions.

use crate::engine::{BuildingType, DroneState, GridPos};

use super::super::game_state::{PlanetState, UndoEntry};

impl PlanetState {
    pub fn begin_relocation(&mut self, source: GridPos) -> bool {
        if self
            .grid
            .get(source)
            .and_then(|tile| tile.building.as_ref())
            .is_none()
        {
            self.notifications
                .warning("Select a structure before relocating");
            return false;
        }
        if self
            .grid
            .get(source)
            .and_then(|tile| tile.building.as_ref())
            .is_some_and(|building| building.building_type == BuildingType::Core)
        {
            self.notifications.warning("The Core cannot be relocated");
            return false;
        }
        self.selected_building = None;
        self.demolish_mode = false;
        self.box_select_mode = false;
        self.box_select_start = None;
        self.box_selected.clear();
        self.bulk_purge_armed = false;
        self.purge_armed = None;
        self.drag_last_pos = None;
        self.smart_drag_source = None;
        self.relocation_source = Some(source);
        self.notifications
            .info("Relocation armed - tap a destination tile");
        true
    }

    pub fn relocate_building(&mut self, source: GridPos, destination: GridPos) -> bool {
        let placed = self.relocate_building_internal(source, destination);
        if placed {
            self.undo_history
                .push(UndoEntry::Relocated(source, destination));
        }
        placed
    }

    fn relocate_building_internal(&mut self, source: GridPos, destination: GridPos) -> bool {
        let Some(building_type) = self
            .grid
            .get(source)
            .and_then(|tile| tile.building.as_ref())
            .map(|building| building.building_type)
        else {
            return false;
        };
        if building_type == BuildingType::Core {
            return false;
        }
        if !self.grid.can_place_building(destination, building_type) {
            self.notifications
                .warning("Relocation destination is invalid");
            return false;
        }

        let Some(mut building) = self.grid.remove_building(source) else {
            return false;
        };
        building.position = destination;
        let Some(tile) = self.grid.get_mut(destination) else {
            if let Some(source_tile) = self.grid.get_mut(source) {
                source_tile.building = Some(building);
            }
            return false;
        };
        tile.building = Some(building);

        move_coordinate_entry(
            &mut self.output_buffers,
            (source.x, source.y),
            (destination.x, destination.y),
        );
        move_coordinate_entry(
            &mut self.input_buffers,
            (source.x, source.y),
            (destination.x, destination.y),
        );
        move_coordinate_entry(
            &mut self.input_hoppers,
            (source.x, source.y),
            (destination.x, destination.y),
        );
        move_coordinate_entry(
            &mut self.pod_loads,
            (source.x, source.y),
            (destination.x, destination.y),
        );
        move_coordinate_entry(
            &mut self.pad_cargo,
            (source.x, source.y),
            (destination.x, destination.y),
        );

        for drone in self.drones.drones_mut() {
            if drone.home != source {
                continue;
            }
            drone.home = destination;
            match drone.state {
                DroneState::Idle => {
                    drone.position = destination;
                    drone.target = destination;
                    drone.path.clear();
                    drone.path_index = 0;
                    drone.progress = 0.0;
                }
                DroneState::MovingToDrill => {
                    // The old return path ends at the source tile. Let the
                    // next network pass find a route to the new home.
                    drone.target = destination;
                    drone.state = DroneState::Error;
                    drone.path.clear();
                    drone.path_index = 0;
                    drone.progress = 0.0;
                }
                DroneState::Error => {
                    drone.target = destination;
                    drone.path.clear();
                    drone.path_index = 0;
                    drone.progress = 0.0;
                }
                DroneState::MovingToCore | DroneState::Delivering => {}
            }
        }

        self.drone_queues.clear();
        self.route_reservations.clear();
        self.cancel_interaction_modes();
        self.grid.reveal_around(destination, 3);
        self.refresh_power_grid();
        self.network_revision = self.network_revision.wrapping_add(1);
        self.power_balance = self.net_power();
        self.placement_anims
            .push(super::super::game_state::PlacementAnim {
                position: destination,
                timer: 0.3,
            });
        self.spawn_place_burst(destination);
        self.emit_audio(super::super::audio::AudioEvent::Placement);
        true
    }

    pub fn undo_last_action(&mut self) -> bool {
        let Some(action) = self.undo_history.pop() else {
            self.notifications.warning("No action to undo");
            return false;
        };
        let creates_inverse = !matches!(action, UndoEntry::Relocated(_, _));
        let success = match action {
            UndoEntry::Placed(pos) => self.try_sell_building(pos),
            UndoEntry::Removed(kind, pos, overclocked, input_priority, standby) => {
                self.selected_building = Some(kind);
                let restored = self.try_place_building(pos);
                self.cancel_interaction_modes();
                if restored {
                    if let Some(building) = self
                        .grid
                        .get_mut(pos)
                        .and_then(|tile| tile.building.as_mut())
                    {
                        let supports_policy = building.supports_overclock();
                        building.overclocked = overclocked && supports_policy && !standby;
                        building.input_priority = input_priority && supports_policy;
                        building.standby = standby && supports_policy;
                    }
                    self.power_balance = self.net_power();
                }
                restored
            }
            UndoEntry::Relocated(source, destination) => {
                self.relocate_building_internal(destination, source)
            }
        };
        if success && creates_inverse {
            self.undo_history.pop();
        }
        if success {
            self.cancel_interaction_modes();
            self.notifications.info("Last placement action undone");
        }
        success
    }
}

fn move_coordinate_entry<T>(
    entries: &mut std::collections::HashMap<(i32, i32), T>,
    source: (i32, i32),
    destination: (i32, i32),
) {
    if let Some(value) = entries.remove(&source) {
        entries.insert(destination, value);
    }
}
