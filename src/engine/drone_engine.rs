//! Resource flow and automation logic

use super::GridPos;
use serde::{Deserialize, Serialize};

/// Resource types that can be gathered and transported
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Minerals,
    Energy,
    Data,
    Biomass,
}

/// Drone states for the automation system
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DroneState {
    Idle,          // Waiting at drill
    MovingToCore,  // Carrying resources to Core
    MovingToDrill, // Returning to drill
    Delivering,    // Unloading at Core
    Error,         // Path blocked or other issue
}

/// Represents a drone carrying resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Drone {
    pub id: u32,
    pub position: GridPos,
    pub target: GridPos,
    pub home_drill: GridPos,
    pub state: DroneState,
    pub resource_type: ResourceType,
    pub carrying: f32,
    pub capacity: f32,
    pub speed: f32,
    pub progress: f32, // 0.0 to 1.0 for smooth movement
    pub path: Vec<GridPos>,
    pub path_index: usize,
}

impl Drone {
    pub fn new(id: u32, drill_pos: GridPos, capacity: f32, speed: f32) -> Self {
        Self {
            id,
            position: drill_pos,
            target: drill_pos,
            home_drill: drill_pos,
            state: DroneState::Idle,
            resource_type: ResourceType::Minerals,
            carrying: 0.0,
            capacity,
            speed,
            progress: 0.0,
            path: Vec::new(),
            path_index: 0,
        }
    }

    /// Start moving to Core with resources
    pub fn dispatch_to_core(&mut self, core_pos: GridPos, path: Vec<GridPos>, amount: f32) {
        self.target = core_pos;
        self.carrying = amount.min(self.capacity);
        self.state = DroneState::MovingToCore;
        self.path = path;
        self.path_index = 0;
        self.progress = 0.0;
    }

    /// Start returning to drill
    pub fn return_to_drill(&mut self, path: Vec<GridPos>) {
        self.target = self.home_drill;
        self.carrying = 0.0;
        self.state = DroneState::MovingToDrill;
        self.path = path;
        self.path_index = 0;
        self.progress = 0.0;
    }

    /// Stop where the drone stands and wave an error flag. Cargo is kept: the
    /// route is broken, not the drone.
    pub fn block(&mut self) -> DroneEvent {
        self.state = DroneState::Error;
        self.target = self.position;
        self.path.clear();
        self.path_index = 0;
        self.progress = 0.0;
        DroneEvent::PathBlocked { drone_id: self.id }
    }

    /// Update drone position and state
    pub fn update(&mut self, delta_time: f32) -> Option<DroneEvent> {
        match self.state {
            DroneState::Idle => None,
            DroneState::MovingToCore | DroneState::MovingToDrill => {
                self.progress += self.speed * delta_time;

                // Carry the overflow: a long step crosses several tiles rather
                // than capping the drone at one tile per call.
                while self.progress >= 1.0 {
                    self.progress -= 1.0;
                    self.path_index += 1;

                    if self.path_index >= self.path.len() {
                        // Reached destination
                        self.progress = 0.0;
                        self.position = self.target;
                        if self.state == DroneState::MovingToCore {
                            self.state = DroneState::Delivering;
                            return Some(DroneEvent::ReachedCore {
                                drone_id: self.id,
                                amount: self.carrying,
                            });
                        }
                        self.state = DroneState::Idle;
                        return Some(DroneEvent::ReachedDrill { drone_id: self.id });
                    }

                    // `path_index` is the tile being moved toward, so the tile
                    // just reached is the one before it.
                    self.position = self.path[self.path_index - 1];
                }
                None
            }
            DroneState::Delivering => {
                // Delivery happens instantly for now
                self.state = DroneState::Idle;
                None
            }
            DroneState::Error => None,
        }
    }

    /// Get interpolated visual position for smooth rendering
    pub fn visual_position(&self) -> (f32, f32) {
        let Some(to) = self.path.get(self.path_index) else {
            return (self.position.x as f32, self.position.y as f32);
        };
        // On the first hop the drone is still leaving its current tile.
        let from = if self.path_index > 0 {
            self.path[self.path_index - 1]
        } else {
            self.position
        };
        let interp_x = from.x as f32 + (to.x - from.x) as f32 * self.progress;
        let interp_y = from.y as f32 + (to.y - from.y) as f32 * self.progress;
        (interp_x, interp_y)
    }
}

/// Events generated by drone actions
#[derive(Debug, Clone)]
pub enum DroneEvent {
    ReachedCore { drone_id: u32, amount: f32 },
    ReachedDrill { drone_id: u32 },
    PathBlocked { drone_id: u32 },
}

/// Manages all drones in the game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DroneManager {
    drones: Vec<Drone>,
    next_id: u32,
    pub drone_capacity: f32,
    pub drone_speed: f32,
}

impl DroneManager {
    pub fn new(capacity: f32, speed: f32) -> Self {
        Self {
            drones: Vec::new(),
            next_id: 1,
            drone_capacity: capacity,
            drone_speed: speed,
        }
    }

    /// Spawn a new drone at a drill position
    pub fn spawn_drone(&mut self, drill_pos: GridPos) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.drones.push(Drone::new(
            id,
            drill_pos,
            self.drone_capacity,
            self.drone_speed,
        ));
        id
    }

    /// Get all drones
    pub fn drones(&self) -> &[Drone] {
        &self.drones
    }

    /// Get mutable access to drones
    pub fn drones_mut(&mut self) -> &mut [Drone] {
        &mut self.drones
    }

    /// Get mutable reference to a drone by ID
    pub fn get_drone_mut(&mut self, id: u32) -> Option<&mut Drone> {
        self.drones.iter_mut().find(|d| d.id == id)
    }

    /// Get drones at a specific drill
    pub fn drones_at_drill(&self, drill_pos: GridPos) -> Vec<&Drone> {
        self.drones
            .iter()
            .filter(|d| d.home_drill == drill_pos)
            .collect()
    }

    /// Remove all drones assigned to a specific drill
    pub fn remove_drones_at_drill(&mut self, drill_pos: GridPos) {
        self.drones.retain(|drone| drone.home_drill != drill_pos);
    }

    /// Get idle drones at a specific drill
    pub fn idle_drones_at_drill(&mut self, drill_pos: GridPos) -> Vec<&mut Drone> {
        self.drones
            .iter_mut()
            .filter(|d| d.home_drill == drill_pos && d.state == DroneState::Idle)
            .collect()
    }

    /// Update all drones and return events
    pub fn update(&mut self, delta_time: f32) -> Vec<DroneEvent> {
        let mut events = Vec::new();
        for drone in &mut self.drones {
            if let Some(event) = drone.update(delta_time) {
                events.push(event);
            }
        }
        events
    }

    /// Count drones by state
    pub fn count_by_state(&self, state: DroneState) -> usize {
        self.drones.iter().filter(|d| d.state == state).count()
    }

    /// Total number of drones
    pub fn total_count(&self) -> usize {
        self.drones.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn straight_path(len: usize) -> Vec<GridPos> {
        (1..=len as i32).map(|x| GridPos::new(x, 0)).collect()
    }

    #[test]
    fn dispatch_to_core_caps_carrying_at_capacity() {
        let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        drone.dispatch_to_core(GridPos::new(3, 0), straight_path(3), 999.0);
        assert_eq!(drone.carrying, 10.0);
        assert_eq!(drone.state, DroneState::MovingToCore);
        assert_eq!(drone.path_index, 0);
    }

    #[test]
    fn update_does_not_arrive_before_crossing_full_progress() {
        let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        drone.dispatch_to_core(GridPos::new(1, 0), straight_path(1), 5.0);

        // speed(5.0) * delta(0.1) = 0.5 progress: not enough to cross one edge yet.
        let event = drone.update(0.1);
        assert!(event.is_none());
        assert_eq!(drone.state, DroneState::MovingToCore);
        assert_eq!(drone.position, GridPos::new(0, 0));
    }

    #[test]
    fn update_reaches_core_after_crossing_a_single_hop_path() {
        let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        drone.dispatch_to_core(GridPos::new(1, 0), straight_path(1), 5.0);

        // speed(5.0) * delta(0.3) = 1.5 progress: crosses the single edge.
        let event = drone.update(0.3);
        assert!(matches!(
            event,
            Some(DroneEvent::ReachedCore { amount, .. }) if amount == 5.0
        ));
        assert_eq!(drone.state, DroneState::Delivering);
        assert_eq!(drone.position, GridPos::new(1, 0));
    }

    #[test]
    fn update_eventually_reaches_core_over_a_multi_hop_path() {
        let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        drone.dispatch_to_core(GridPos::new(3, 0), straight_path(3), 5.0);

        let mut delivered = None;
        for _ in 0..20 {
            if let Some(DroneEvent::ReachedCore { amount, .. }) = drone.update(0.3) {
                delivered = Some(amount);
                break;
            }
        }
        assert_eq!(delivered, Some(5.0));
        assert_eq!(drone.state, DroneState::Delivering);
        assert_eq!(drone.position, GridPos::new(3, 0));
    }

    #[test]
    fn return_to_drill_reaches_drill_and_goes_idle() {
        let mut drone = Drone::new(1, GridPos::new(2, 0), 10.0, 10.0);
        drone.position = GridPos::new(2, 0);
        drone.return_to_drill(straight_path(0));
        // Empty path: the very next update should immediately arrive.
        let event = drone.update(1.0);
        assert!(matches!(event, Some(DroneEvent::ReachedDrill { .. })));
        assert_eq!(drone.state, DroneState::Idle);
    }

    #[test]
    fn idle_and_error_states_do_not_move() {
        let mut idle = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        assert!(idle.update(1.0).is_none());
        assert_eq!(idle.position, GridPos::new(0, 0));

        let mut errored = Drone::new(2, GridPos::new(0, 0), 10.0, 5.0);
        errored.state = DroneState::Error;
        assert!(errored.update(1.0).is_none());
        assert_eq!(errored.state, DroneState::Error);
    }

    #[test]
    fn manager_spawns_and_tracks_drones_per_drill() {
        let mut manager = DroneManager::new(10.0, 5.0);
        let drill_a = GridPos::new(0, 0);
        let drill_b = GridPos::new(5, 5);
        manager.spawn_drone(drill_a);
        manager.spawn_drone(drill_a);
        manager.spawn_drone(drill_b);

        assert_eq!(manager.total_count(), 3);
        assert_eq!(manager.drones_at_drill(drill_a).len(), 2);
        assert_eq!(manager.count_by_state(DroneState::Idle), 3);

        manager.remove_drones_at_drill(drill_a);
        assert_eq!(manager.total_count(), 1);
    }

    #[test]
    fn manager_assigns_unique_ascending_ids() {
        let mut manager = DroneManager::new(10.0, 5.0);
        let id1 = manager.spawn_drone(GridPos::new(0, 0));
        let id2 = manager.spawn_drone(GridPos::new(0, 0));
        assert_ne!(id1, id2);
        assert!(manager.get_drone_mut(id1).is_some());
        assert!(manager.get_drone_mut(id2).is_some());
        assert!(manager.get_drone_mut(id2 + 100).is_none());
    }

    #[test]
    fn a_long_step_crosses_several_tiles_instead_of_capping_at_one() {
        let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        drone.dispatch_to_core(GridPos::new(6, 0), straight_path(6), 5.0);

        // speed(5.0) * delta(0.7) = 3.5 tiles of travel in a single step.
        assert!(drone.update(0.7).is_none());
        assert_eq!(drone.position, GridPos::new(3, 0));
        assert!((drone.progress - 0.5).abs() < 1e-5);
    }

    #[test]
    fn a_step_long_enough_to_overshoot_still_arrives_once() {
        let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        drone.dispatch_to_core(GridPos::new(3, 0), straight_path(3), 5.0);

        // Sixty seconds of travel over a three tile path: arrive, do not wrap.
        let event = drone.update(60.0);
        assert!(matches!(event, Some(DroneEvent::ReachedCore { .. })));
        assert_eq!(drone.position, GridPos::new(3, 0));
        assert_eq!(drone.progress, 0.0);
    }

    #[test]
    fn position_tracks_the_tile_the_drone_last_reached() {
        let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        drone.dispatch_to_core(GridPos::new(3, 0), straight_path(3), 5.0);

        // Half way along the first hop: still standing on the drill tile.
        drone.update(0.1);
        assert_eq!(drone.position, GridPos::new(0, 0));
        let (vx, _) = drone.visual_position();
        assert!(vx > 0.0 && vx < 1.0);

        // Crossing the first hop lands the drone on the first path tile.
        drone.update(0.2);
        assert_eq!(drone.position, GridPos::new(1, 0));
    }

    #[test]
    fn blocking_a_drone_stops_it_where_it_stands_and_keeps_its_cargo() {
        let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        drone.dispatch_to_core(GridPos::new(3, 0), straight_path(3), 7.0);
        drone.update(0.3);

        let event = drone.block();
        assert!(matches!(event, DroneEvent::PathBlocked { drone_id: 1 }));
        assert_eq!(drone.state, DroneState::Error);
        assert_eq!(drone.carrying, 7.0);
        assert_eq!(drone.position, GridPos::new(1, 0));
        assert!(drone.path.is_empty());
        // A blocked drone does not drift onwards.
        assert!(drone.update(1.0).is_none());
        assert_eq!(drone.position, GridPos::new(1, 0));
    }

    #[test]
    fn block_is_idempotent_for_an_already_blocked_drone() {
        let mut drone = Drone::new(1, GridPos::new(0, 0), 10.0, 5.0);
        drone.block();
        drone.block();
        assert_eq!(drone.state, DroneState::Error);
        assert_eq!(drone.position, GridPos::new(0, 0));
    }
}
