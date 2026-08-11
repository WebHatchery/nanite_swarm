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
    Alloy,
}

impl ResourceType {
    pub fn id(self) -> &'static str {
        match self {
            ResourceType::Minerals => "minerals",
            ResourceType::Energy => "energy",
            ResourceType::Data => "data",
            ResourceType::Biomass => "biomass",
            ResourceType::Alloy => "alloy",
        }
    }

    pub const ALL: [ResourceType; 5] = [
        ResourceType::Minerals,
        ResourceType::Energy,
        ResourceType::Data,
        ResourceType::Biomass,
        ResourceType::Alloy,
    ];

    pub fn from_id(id: &str) -> Option<Self> {
        ResourceType::ALL.into_iter().find(|kind| kind.id() == id)
    }

    /// Whether a drone can pick this up and walk it somewhere. Data is
    /// information and Energy is in the wires; neither rides a drone.
    pub fn is_physical(self) -> bool {
        matches!(
            self,
            ResourceType::Minerals | ResourceType::Biomass | ResourceType::Alloy
        )
    }
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
    pub home: GridPos,
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
            home: drill_pos,
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

    /// Send this drone somewhere with a load: the Core, or a building that
    /// wants what it is carrying.
    pub fn dispatch(
        &mut self,
        destination: GridPos,
        path: Vec<GridPos>,
        amount: f32,
        resource: ResourceType,
    ) {
        self.target = destination;
        self.resource_type = resource;
        self.carrying = amount.min(self.capacity);
        self.state = DroneState::MovingToCore;
        self.path = path;
        self.path_index = 0;
        self.progress = 0.0;
    }

    /// Start returning to drill
    pub fn return_to_drill(&mut self, path: Vec<GridPos>) {
        self.target = self.home;
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
                            return Some(DroneEvent::Delivered {
                                drone_id: self.id,
                                amount: self.carrying,
                                at: self.target,
                                resource: self.resource_type,
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
    /// A load arrived somewhere. Where matters now that a drone can be sent to
    /// a processing building instead of the Core.
    Delivered {
        drone_id: u32,
        amount: f32,
        at: GridPos,
        resource: ResourceType,
    },
    ReachedDrill {
        drone_id: u32,
    },
    PathBlocked {
        drone_id: u32,
    },
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
    pub fn drones_at(&self, drill_pos: GridPos) -> Vec<&Drone> {
        self.drones.iter().filter(|d| d.home == drill_pos).collect()
    }

    /// Remove all drones assigned to a specific drill
    pub fn remove_drones_at(&mut self, drill_pos: GridPos) {
        self.drones.retain(|drone| drone.home != drill_pos);
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
mod tests;
