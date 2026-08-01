//! Current planetary state

use super::camera::Camera;
use super::seed_ship::SeedShip;
use crate::data::{GameConfig, PlanetHazards};
use crate::engine::{BuildingType, DroneManager, Grid, GridPos, Stats};
use macroquad::miniquad;
use macroquad_toolkit::achievements::{Achievement, Achievements};
use macroquad_toolkit::fx::ParticleSystem;
use macroquad_toolkit::ui::ScrollArea;
use serde::{Deserialize, Serialize};

/// Saves written before worlds had identities were all the starting world.
fn default_planet_index() -> usize {
    2
}

fn unix_seconds_now() -> i64 {
    (miniquad::date::now() as i64).max(0)
}

/// Placement animation for newly placed buildings
#[derive(Debug, Clone)]
pub struct PlacementAnim {
    pub position: GridPos,
    pub timer: f32,
}
/// Resources held by the player
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Resources {
    pub energy: f32,
    pub minerals: f32,
    pub data: f32,
    pub biomass: f32,
    /// Refined from minerals by processing buildings. The first product the
    /// swarm cannot dig straight out of the ground.
    #[serde(default)]
    pub alloy: f32,
}

impl Resources {
    /// Check if player can afford a cost
    pub fn can_afford(&self, minerals: f32, energy: f32) -> bool {
        self.minerals >= minerals && self.energy >= energy
    }

    /// Deduct cost from resources
    pub fn spend(&mut self, minerals: f32, energy: f32) -> bool {
        if self.can_afford(minerals, energy) {
            self.minerals -= minerals;
            self.energy -= energy;
            true
        } else {
            false
        }
    }
}

/// Research node status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchProgress {
    pub unlocked_techs: Vec<String>,
    pub current_research: Option<String>,
    pub research_progress: f32,
}

impl Default for ResearchProgress {
    fn default() -> Self {
        Self {
            unlocked_techs: crate::data::game_data().research.starting_unlocked.clone(),
            current_research: None,
            research_progress: 0.0,
        }
    }
}

/// Current game state for a planet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetState {
    pub name: String,
    /// Which campaign slot this world is, so it can find its own definition.
    #[serde(default = "default_planet_index")]
    pub planet_index: usize,
    pub resources: Resources,
    pub grid: Grid,
    pub drones: DroneManager,
    pub research: ResearchProgress,
    pub config: GameConfig,
    pub time_played: f64,
    pub selected_building: Option<BuildingType>,
    pub power_balance: f32,
    #[serde(skip, default)]
    pub biomass_power_bonus: f32,
    /// Unspent real time between fixed simulation steps.
    #[serde(skip, default)]
    pub sim_accumulator: f32,
    pub battery_seconds: f32,
    pub last_saved_unix: i64,
    pub achievements: Achievements,
    pub unlocked_buildings: Vec<BuildingType>,
    /// What this world will not let the swarm build at all: no wind where
    /// there is no wind, no harvesters where nothing grows.
    #[serde(default)]
    pub banned_buildings: Vec<BuildingType>,
    /// What this world does to the machinery standing on it.
    #[serde(default)]
    pub hazards: PlanetHazards,
    /// The megastructure this world is being converted into.
    #[serde(default)]
    pub seed_ship: SeedShip,
    /// Everything unlocked research does to the simulation, folded into one
    /// sheet. Derived from `research.unlocked_techs` — never edited directly,
    /// always rebuilt by [`PlanetState::refresh_stats`].
    #[serde(skip, default)]
    pub stats: Stats,
    #[serde(skip, default)]
    pub power_negative_seconds: f32,
    #[serde(skip, default)]
    pub power_collapse_cooldown: f32,
    #[serde(skip, default)]
    pub power_collapse_shutdown: f32,
    #[serde(skip, default)]
    pub research_lock_timer: f32,
    #[serde(skip, default)]
    pub collapse_notice_timer: f32,
    /// Counts down while the arrival line for this world is on screen.
    #[serde(skip, default)]
    pub arrival_notice_timer: f32,
    #[serde(skip, default)]
    pub forest_harvested_count: i32,
    #[serde(skip, default)]
    pub tutorial_step: u8,
    #[serde(skip, default)]
    pub tutorial_hidden: bool,
    #[serde(skip, default)]
    pub tutorial_done: bool,
    #[serde(skip, default)]
    pub last_offline_seconds: f32,
    #[serde(skip, default)]
    pub last_offline_simulated: f32,
    #[serde(skip, default)]
    pub offline_notice_timer: f32,
    #[serde(skip, default)]
    pub drag_last_pos: Option<GridPos>,
    #[serde(skip, default)]
    pub selected_tile: Option<GridPos>,
    /// Where the player is looking at this world from. Kept per planet, so a
    /// world is framed as it was left.
    #[serde(default)]
    pub camera: Camera,
    /// Cursor position the current middle-drag was last seen at.
    #[serde(skip, default)]
    pub camera_drag_anchor: Option<(f32, f32)>,
    #[serde(skip, default)]
    pub show_help: bool,
    #[serde(skip, default)]
    pub build_palette_scroll: ScrollArea,
    #[serde(skip, default)]
    pub particles: ParticleSystem,
    #[serde(skip, default)]
    pub particle_timer: f32,
    #[serde(skip, default)]
    pub placement_anims: Vec<PlacementAnim>,
    // Minerals a drill has cut but not yet handed to a drone
    #[serde(skip)]
    pub output_buffers: std::collections::HashMap<(i32, i32), f32>,
    // Ore delivered to a processing building and not yet consumed
    #[serde(default)]
    pub input_buffers: std::collections::HashMap<(i32, i32), f32>,
    // Drones currently crossing each network tile, rebuilt every tick
    #[serde(skip)]
    pub traffic: std::collections::HashMap<(i32, i32), u32>,
}

impl PlanetState {
    /// Build the world at campaign slot `planet_index`, from its entry in
    /// `assets/planets.json`.
    pub fn new(planet_index: usize, seed: u64, config: GameConfig) -> Self {
        let def = crate::data::game_data().planet(planet_index);
        let (name, width, height) = (def.name.as_str(), def.width, def.height);
        let mut grid = Grid::new_with_terrain(width, height, seed, &def.terrain);
        grid.initialize_forest_biomass(config.resources.forest_biomass);

        // Place Core at center
        let center = GridPos::new(width as i32 / 2, height as i32 / 2);
        grid.place_building(center, BuildingType::Core);
        grid.update_power_grid();

        let mut unlocked_buildings = Vec::new();
        for def in &crate::data::game_data().buildings {
            if def.start_unlocked {
                if let Some(building_type) = BuildingType::from_id(&def.id) {
                    unlocked_buildings.push(building_type);
                }
            }
        }

        let mut state = Self {
            name: name.to_string(),
            planet_index,
            resources: Resources {
                energy: config.resources.starting_energy,
                minerals: config.resources.starting_minerals,
                data: 0.0,
                biomass: 0.0,
                alloy: 0.0,
            },
            grid,
            drones: DroneManager::new(
                config.resources.drone_carry_capacity,
                config.resources.drone_speed,
            ),
            research: ResearchProgress::default(),
            config,
            time_played: 0.0,
            selected_building: Some(BuildingType::Drill),
            power_balance: 10.0,
            biomass_power_bonus: 0.0,
            sim_accumulator: 0.0,
            battery_seconds: 4.0 * 60.0 * 60.0,
            last_saved_unix: unix_seconds_now(),
            achievements: Achievements::from_definitions(achievement_definitions()),
            stats: Stats::default(),
            power_negative_seconds: 0.0,
            power_collapse_cooldown: 0.0,
            power_collapse_shutdown: 0.0,
            research_lock_timer: 0.0,
            collapse_notice_timer: 0.0,
            arrival_notice_timer: 0.0,
            forest_harvested_count: 0,
            tutorial_step: 0,
            tutorial_hidden: false,
            tutorial_done: false,
            unlocked_buildings,
            banned_buildings: def
                .banned_buildings
                .iter()
                .filter_map(|id| BuildingType::from_id(id))
                .collect(),
            hazards: def.hazards,
            seed_ship: SeedShip::default(),
            last_offline_seconds: 0.0,
            last_offline_simulated: 0.0,
            offline_notice_timer: 0.0,
            drag_last_pos: None,
            selected_tile: None,
            camera: Camera::default(),
            camera_drag_anchor: None,
            show_help: false,
            build_palette_scroll: ScrollArea::new(),
            particles: ParticleSystem::new(),
            particle_timer: 0.0,
            placement_anims: Vec::new(),
            output_buffers: std::collections::HashMap::new(),
            input_buffers: std::collections::HashMap::new(),
            traffic: std::collections::HashMap::new(),
        };
        state.refresh_stats();
        state
    }
}

impl PlanetState {
    /// Rebuild the stat sheet from what research has unlocked, and push the
    /// values that live outside it into place. Call this after anything that
    /// changes `research.unlocked_techs`.
    pub fn refresh_stats(&mut self) {
        self.stats = Stats::from_unlocked(&self.research.unlocked_techs);

        let capacity = self.stats.apply(
            crate::engine::StatId::DroneCapacity,
            self.config.resources.drone_carry_capacity,
        );
        self.drones.drone_capacity = capacity;
        for drone in self.drones.drones_mut() {
            drone.capacity = capacity;
        }
    }
}

impl Default for PlanetState {
    fn default() -> Self {
        Self::new(2, 42, GameConfig::default())
    }
}

/// Canonical achievement definitions. Used both to seed a new [`PlanetState`]
/// and to reconcile loaded saves via [`Achievements::sync_definitions`].
pub(crate) fn achievement_definitions() -> Vec<Achievement> {
    vec![
        Achievement::new("first_drill", "First Drill", "Place your first drill."),
        Achievement::new(
            "power_surplus",
            "Power Surplus",
            "Reach positive net power.",
        ),
        Achievement::new("data_miner", "Data Miner", "Accumulate 25 data."),
        Achievement::new("builder", "Builder", "Place 10 buildings."),
        Achievement::new(
            "seed_ship",
            "Seed Ship",
            "Finish a Seed Ship and leave a world spent.",
        ),
    ]
}
