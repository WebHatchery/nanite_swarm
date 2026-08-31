//! Current planetary state

use super::camera::Camera;
pub use super::consumption::{ConsumptionTotals, Resources};
use super::factory::FactoryFocus;
use super::seed_ship::SeedShip;
mod construction;
mod features;
mod records;
mod research;
use crate::data::{GameConfig, GameData, GameDataContext, PlanetHazards};
use crate::engine::{BuildingType, DroneManager, Grid, GridPos, ResourceType, Stats};
pub(crate) use features::feature_cooling_multiplier;
pub(crate) use features::feature_hazard_multiplier;
pub(crate) use features::FeatureEffects;
pub use features::PlanetFeature;
use macroquad::miniquad;
use macroquad_toolkit::achievements::{Achievement, Achievements};
use macroquad_toolkit::fx::ParticleSystem;
use macroquad_toolkit::input::TouchGesture;
use macroquad_toolkit::notifications::NotificationManager;
use macroquad_toolkit::ui::ScrollArea;
pub use records::{BlueprintEntry, CollapseRecord, GraphMode, GraphSample, UndoEntry};
pub use research::ResearchProgress;
use serde::{Deserialize, Serialize};

/// Real time and world time run at the same rate until the player says
/// otherwise.
fn default_time_scale() -> f32 {
    1.0
}

/// Saves written before worlds had identities were all the starting world.
/// How many buckets the throughput graph keeps. The toolkit's series merges
/// pairs when it fills, so a long session loses resolution but never loses a
/// spike.
fn default_throughput() -> macroquad_toolkit::series::Series {
    macroquad_toolkit::series::Series::new(120)
}

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

/// Current game state for a planet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanetState {
    /// Immutable definitions injected by the campaign session. They are not
    /// save data: loading reattaches the active session context.
    #[serde(skip, default)]
    pub(crate) data: GameDataContext,
    pub name: String,
    /// Which campaign slot this world is, so it can find its own definition.
    #[serde(default = "default_planet_index")]
    pub planet_index: usize,
    #[serde(default)]
    pub map_seed: u64,
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
    /// Nothing advances while this is set.
    #[serde(skip, default)]
    pub paused: bool,
    /// How much world time passes per second of real time.
    #[serde(skip, default = "default_time_scale")]
    pub time_scale: f32,
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
    #[serde(default)]
    pub features: Vec<PlanetFeature>,
    /// The megastructure this world is being converted into.
    #[serde(default)]
    pub seed_ship: SeedShip,
    /// What this world's Mass Drivers load, and where they throw it. Kept when
    /// the swarm leaves: a world left behind keeps feeding the one it left for.
    #[serde(default)]
    pub export: Option<super::shipping::ExportOrder>,
    #[serde(default)]
    pub export_cooldown: f32,
    /// How much each driver has loaded into the pod it is filling.
    #[serde(default, with = "crate::state::coordinate_map")]
    pub pod_loads: std::collections::HashMap<(i32, i32), f32>,
    /// Pods thrown this step, waiting for the campaign to put them in flight.
    #[serde(default)]
    pub launched_pods: Vec<super::shipping::Shipment>,
    /// What each Landing Pad is currently piled with. A pad holds one cargo at
    /// a time, the same way a drill's pad only ever holds ore.
    #[serde(default, with = "crate::state::coordinate_map")]
    pub pad_cargo: std::collections::HashMap<(i32, i32), ResourceType>,
    /// Everything unlocked research does to the simulation, folded into one
    /// sheet. Derived from `research.unlocked_techs` — never edited directly,
    /// always rebuilt by [`PlanetState::refresh_stats`].
    #[serde(skip, default)]
    pub stats: Stats,
    /// Ore banked at the Core per second, sampled once a second across the
    /// whole session. Not saved: a graph of a session belongs to that session.
    #[serde(skip, default = "default_throughput")]
    pub throughput: macroquad_toolkit::series::Series,
    /// Delivered since the last sample, and how long ago that was.
    #[serde(skip, default)]
    pub delivered_since_sample: f32,
    #[serde(skip, default)]
    pub throughput_timer: f32,
    /// How far the Core has evolved. Earned, monotonic, and saved: the stage
    /// used to be a number the renderer guessed from the stockpile.
    #[serde(default)]
    pub core_stage: u8,
    #[serde(skip, default)]
    pub power_negative_seconds: f32,
    #[serde(skip, default)]
    pub power_collapse_cooldown: f32,
    #[serde(skip, default)]
    pub power_collapse_shutdown: f32,
    /// How long the collapse in progress was going to last, so a readout can
    /// show how far through it is without assuming a fixed length.
    #[serde(skip, default)]
    pub power_collapse_length: f32,
    #[serde(skip, default)]
    pub research_lock_timer: f32,
    #[serde(skip, default)]
    pub collapse_notice_timer: f32,
    /// Counts down while the arrival line for this world is on screen.
    #[serde(skip, default)]
    pub arrival_notice_timer: f32,
    /// Counts down while the saved marker is on screen.
    #[serde(skip, default)]
    pub save_notice_timer: f32,
    /// The last write to disk failed, and the player should know.
    #[serde(skip, default)]
    pub save_failed: bool,
    /// This campaign came back from the backup, not the main save.
    #[serde(skip, default)]
    pub restored_from_backup: bool,
    #[serde(skip, default)]
    pub restored_from_backup_generation: u8,
    #[serde(skip, default)]
    pub forest_harvested_count: i32,
    /// Persisted: a tutorial that restarts every time the game is loaded is
    /// worse than none.
    #[serde(default)]
    pub tutorial_step: u8,
    #[serde(skip, default)]
    pub tutorial_hidden: bool,
    #[serde(default)]
    pub tutorial_done: bool,
    #[serde(skip, default)]
    pub last_offline_seconds: f32,
    #[serde(skip, default)]
    pub last_offline_simulated: f32,
    #[serde(skip, default)]
    pub offline_notice_timer: f32,
    #[serde(skip, default)]
    pub drag_last_pos: Option<GridPos>,
    /// The conduit segment that already explained a resource shortfall during
    /// the current drag gesture.
    #[serde(skip, default)]
    pub route_resource_blocked_at: Option<GridPos>,
    /// The route destination that already explained a no-path failure during
    /// the current drag gesture.
    #[serde(skip, default)]
    pub route_path_blocked_at: Option<GridPos>,
    /// Existing building that began the current Smart Drag route.
    #[serde(skip, default)]
    pub smart_drag_source: Option<GridPos>,
    #[serde(skip, default)]
    pub selected_tile: Option<GridPos>,
    /// Clicking tears buildings down instead of putting them up.
    #[serde(skip, default)]
    pub demolish_mode: bool,
    /// Where the player is looking at this world from. Kept per planet, so a
    /// world is framed as it was left.
    #[serde(default)]
    pub camera: Camera,
    /// Tap, pan, and pinch state for the planetary map.
    #[serde(skip, default)]
    pub touch_gesture: TouchGesture,
    /// The current touch began on the map rather than over a HUD panel.
    #[serde(skip, default)]
    pub touch_camera_active: bool,
    /// The current single-finger map touch is reserved for laying a conduit.
    #[serde(skip, default)]
    pub touch_route_active: bool,
    /// Whether the current touch was already assigned to the map or the HUD.
    #[serde(skip, default)]
    pub touch_gesture_routed: bool,
    /// Cursor position the current middle-drag was last seen at.
    #[serde(skip, default)]
    pub camera_drag_anchor: Option<(f32, f32)>,
    #[serde(skip, default)]
    pub show_help: bool,
    /// Portrait-only drawer selection; this is session UI, never campaign
    /// state, so loading a save returns to the map.
    #[serde(skip, default)]
    pub mobile_panel: super::mobile::MobilePanel,
    /// Hide both side stacks while planning dense routes. Persisted per world
    /// because a returning touch player should keep the viewport they chose.
    #[serde(default)]
    pub focus_mode: bool,
    /// Show recipe nodes and their intended network routes over the map.
    #[serde(default)]
    pub flow_overlay: bool,
    /// Let the swarm boost healthy, fed processors only while power is spare.
    #[serde(default)]
    pub auto_clocking: bool,
    /// The production deck currently receiving extra scheduling attention.
    #[serde(default)]
    pub factory_focus: FactoryFocus,
    /// Whether the in-world factory control deck is expanded.
    #[serde(default)]
    pub factory_deck_open: bool,
    #[serde(skip, default)]
    pub auto_clock_timer: f32,
    #[serde(skip, default)]
    pub build_palette_scroll: ScrollArea,
    /// Scroll position for the portrait Seed Ship yard.
    #[serde(skip, default)]
    pub seed_ship_scroll: ScrollArea,
    /// Where the Records screen's log is scrolled to.
    #[serde(skip, default)]
    pub log_scroll: ScrollArea,
    #[serde(skip, default)]
    pub records_scroll: ScrollArea,
    /// Things worth telling the player about as they happen. Achievements,
    /// finished research and Seed Ship stages all used to land silently.
    #[serde(skip, default)]
    pub notifications: NotificationManager,
    #[serde(skip, default)]
    pub particles: ParticleSystem,
    #[serde(skip, default)]
    pub particle_timer: f32,
    #[serde(skip, default)]
    pub placement_anims: Vec<PlacementAnim>,
    /// Physical output waiting on a producer's pad, including landed freight.
    /// It is durable cargo rather than a derived simulation cache: discarding
    /// it on save would erase resources the player had already produced.
    #[serde(default, with = "crate::state::coordinate_map")]
    pub output_buffers: std::collections::HashMap<(i32, i32), f32>,
    // Ore delivered to a processing building and not yet consumed
    #[serde(default, with = "crate::state::coordinate_map")]
    pub input_buffers: std::collections::HashMap<(i32, i32), f32>,
    /// Per-resource hopper state. `input_buffers` remains as a total for
    /// readable legacy saves and compact inspector summaries.
    #[serde(default, with = "crate::state::coordinate_map")]
    pub input_hoppers:
        std::collections::HashMap<(i32, i32), std::collections::HashMap<ResourceType, f32>>,
    // Drones currently crossing each network tile, rebuilt every tick
    #[serde(skip)]
    pub traffic: std::collections::HashMap<(i32, i32), u32>,
    /// Incremented only when the network topology changes. Path caches and
    /// reservations use this instead of invalidating on every simulation tick.
    #[serde(default)]
    pub network_revision: u64,
    #[serde(skip, default)]
    pub route_cache_revision: u64,
    #[serde(skip, default)]
    pub drone_queues: std::collections::HashMap<(i32, i32), Vec<u32>>,
    #[serde(skip, default)]
    pub route_reservations: std::collections::HashMap<(i32, i32), f32>,
    #[serde(default)]
    pub collapse_history: Vec<CollapseRecord>,
    #[serde(default)]
    pub graph_samples: Vec<GraphSample>,
    /// Session-only graph selector; saves reopen on the factory-flow view.
    #[serde(skip, default)]
    pub graph_mode: GraphMode,
    /// Recipe inputs and outputs accumulated until the next one-second graph
    /// sample. Session-only: a loaded game starts a fresh observation window.
    #[serde(skip, default)]
    pub factory_flow_since_sample: GraphSample,
    #[serde(skip, default)]
    pub last_offline_report: super::progress::OfflineReport,
    #[serde(skip, default)]
    pub audio_events: Vec<super::audio::AudioEvent>,
    #[serde(default)]
    pub blueprint: Vec<BlueprintEntry>,
    #[serde(skip, default)]
    pub relocation_source: Option<GridPos>,
    #[serde(skip, default)]
    pub box_select_mode: bool,
    #[serde(skip, default)]
    pub box_select_start: Option<GridPos>,
    #[serde(skip, default)]
    pub box_selected: Vec<GridPos>,
    /// A destructive pad purge needs the same visible tile tapped twice.
    #[serde(skip, default)]
    pub purge_armed: Option<GridPos>,
    /// The selected-pad purge likewise requires its visible control twice.
    #[serde(skip, default)]
    pub bulk_purge_armed: bool,
    #[serde(skip, default)]
    pub undo_history: Vec<UndoEntry>,
}

impl Default for PlanetState {
    fn default() -> Self {
        Self::new(2, 42, GameConfig::default())
    }
}

/// The achievement set, from `assets/achievements.json`. Used both to seed a
/// new [`PlanetState`] and to reconcile loaded saves via
/// [`Achievements::sync_definitions`], so a save written before an achievement
/// existed picks it up locked rather than losing the rest.
pub(crate) fn achievement_definitions(data: &GameData) -> Vec<Achievement> {
    data.achievements
        .iter()
        .map(|def| Achievement::new(&def.id, &def.name, &def.description))
        .collect()
}

#[cfg(test)]
mod tests;
