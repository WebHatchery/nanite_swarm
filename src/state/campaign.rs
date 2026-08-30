//! The solar campaign: every world the swarm has landed on.
//!
//! The GDD's meta-layer rests on one promise — "when the player moves to
//! Planet 2, Planet 1 does not disappear". So a planet is not rebuilt on
//! arrival: it lives in its slot for the whole campaign, and travelling only
//! changes which slot is in front of the player.

use macroquad_toolkit::notifications::LoggedNotification;
use serde::{Deserialize, Serialize};

use crate::data::{GameConfig, GameDataContext};
use crate::directives::{pick_directive, Directive};
use crate::engine::{ResearchTree, ResourceType, StatId};

use super::consumption::ConsumptionTotals;
use super::core_stage::CoreRequirementProgress;
use super::game_state::{PlanetState, ResearchProgress};
use super::UpkeepAlertCause;

pub const PLANET_COUNT: usize = 5;
/// Mars, per the GDD's Zone 1.
pub const STARTING_PLANET: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveRecord {
    pub description: String,
    pub completed: bool,
    pub reward_data: f32,
    pub tier: i32,
}

/// The small live dossier the Solar System map can show for every world the
/// swarm has already reached. These are copied values rather than references
/// so the map can assemble one stable snapshot while the campaign remains
/// mutable behind the screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorldAlert {
    Nominal,
    Collapse,
    Power,
    Hibernation,
    Stalled,
    Starved,
    Blocked,
    Storage,
    Maintenance,
}

impl WorldAlert {
    pub fn label(self) -> &'static str {
        match self {
            Self::Nominal => "NOMINAL",
            Self::Collapse => "COLLAPSE",
            Self::Power => "POWER",
            Self::Hibernation => "HIBERNATION",
            Self::Stalled => "STALL",
            Self::Starved => "INPUT",
            Self::Blocked => "BLOCKED",
            Self::Storage => "STORAGE",
            Self::Maintenance => "MAINTENANCE",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldTelemetry {
    pub minerals: f32,
    pub data: f32,
    pub biomass: f32,
    pub alloy: f32,
    pub components: f32,
    pub ore_rate: f32,
    pub data_rate: f32,
    pub biomass_rate: f32,
    pub alloy_rate: f32,
    pub components_rate: f32,
    pub power_balance: f32,
    pub structures: usize,
    pub drones: usize,
    /// The world's earned Core evolution, one-based for player-facing
    /// telemetry. The total travels with it so the dossier remains honest if
    /// the data file gains another stage later.
    pub core_stage: usize,
    pub core_stage_count: usize,
    /// The first unmet requirement for the next Core stage, if one exists.
    /// Keeping this as a copy lets the Solar System explain remote growth
    /// without borrowing a planet through the rendering layer.
    pub core_next_requirement: Option<CoreRequirementProgress>,
    pub alerts: usize,
    pub alert_kind: WorldAlert,
    /// The first stored resource at its live ceiling while that resource has
    /// a live producer, if any. Energy is excluded because a full battery is
    /// normal for a healthy remote world, and idle full reserves are not waste.
    /// The alert count still reports saturation as one distinct condition even
    /// when more than one stored resource is full.
    pub storage_full_resource: Option<ResourceType>,
    /// Whether remote maintenance is caused by dust, corrosion, or both.
    pub maintenance_cause: Option<UpkeepAlertCause>,
}

/// Step length for worlds the player is not looking at. Coarse on purpose: a
/// left-behind world only has to keep producing, not animate.
const BACKGROUND_TICK_SECONDS: f32 = 1.0;
/// Ceiling on background steps per call, so a long stall cannot turn into a
/// stutter across four planets at once.
const MAX_BACKGROUND_TICKS: u32 = 4;
/// How long a world's arrival line stays on screen.
const ARRIVAL_NOTICE_SECONDS: f32 = 10.0;
/// How long the saved marker stays on screen.
const SAVE_NOTICE_SECONDS: f32 = 3.0;

/// Every planet the swarm holds, plus which one it is currently standing on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    /// Session-owned immutable data; omitted from player saves and reattached
    /// by the persistence boundary after loading.
    #[serde(skip, default)]
    data: GameDataContext,
    /// Player-visible save slot. The campaign owns this label so a save can
    /// never silently change slots when loaded by another menu state.
    #[serde(default = "default_slot_name")]
    pub slot_name: String,
    pub(super) planets: [Option<PlanetState>; PLANET_COUNT],
    current: usize,
    seed: u64,
    pub directive: Directive,
    #[serde(default)]
    pub directive_history: Vec<DirectiveRecord>,
    /// What the swarm knows. One copy for the whole campaign: research is the
    /// swarm's, not a world's, and a world left behind was simulating with a
    /// stale stat sheet while this lived on each planet separately.
    #[serde(default)]
    pub research: ResearchProgress,
    directive_timer: f32,
    directive_tier: i32,
    /// World time since the campaign was last written to disk.
    #[serde(skip, default)]
    since_save: f32,
    /// Unspent time owed to the worlds nobody is watching.
    #[serde(skip, default)]
    background_accumulator: f32,
    /// Everything a Mass Driver has thrown and nothing has caught yet.
    #[serde(default)]
    pub(super) shipments: Vec<super::shipping::Shipment>,
    /// One recoverable log for the campaign, regardless of which world raised
    /// the event. It survives travel and save/load with the campaign itself.
    #[serde(default)]
    pub toast_history: Vec<LoggedNotification>,
    /// The completion screen has been acknowledged; persisted so choosing to
    /// keep building remains a choice after the campaign is reloaded.
    #[serde(default)]
    pub ending_seen: bool,
    #[serde(skip, default)]
    notification_cursors: [usize; PLANET_COUNT],
}

fn default_slot_name() -> String {
    "slot_1".to_string()
}

impl Campaign {
    /// Start a campaign: one colonized world, the rest untouched.
    pub fn new(config: GameConfig, seed: u64) -> Self {
        Self::new_with_data(config, seed, GameDataContext::default())
    }

    pub fn new_with_data(config: GameConfig, seed: u64, data: GameDataContext) -> Self {
        let directive = pick_directive(data.data(), 0);
        let research = ResearchProgress::from_data(data.data());
        let mut campaign = Self {
            data,
            slot_name: default_slot_name(),
            planets: std::array::from_fn(|_| None),
            current: STARTING_PLANET,
            seed,
            directive,
            directive_history: Vec::new(),
            research,
            directive_timer: 0.0,
            directive_tier: 0,
            since_save: 0.0,
            background_accumulator: 0.0,
            shipments: Vec::new(),
            toast_history: Vec::new(),
            ending_seen: false,
            notification_cursors: [0; PLANET_COUNT],
        };
        campaign.planets[STARTING_PLANET] = Some(campaign.generate(STARTING_PLANET, &config));
        campaign
    }

    /// Wrap a single planet as a campaign, for a save written before the
    /// campaign existed.
    pub fn from_single_planet(planet: PlanetState, seed: u64) -> Self {
        let data = GameDataContext::default();
        let directive = pick_directive(data.data(), 0);
        let research = ResearchProgress::from_data(data.data());
        let mut planets: [Option<PlanetState>; PLANET_COUNT] = std::array::from_fn(|_| None);
        planets[STARTING_PLANET] = Some(planet);
        Self {
            data,
            slot_name: default_slot_name(),
            planets,
            current: STARTING_PLANET,
            seed,
            directive,
            directive_history: Vec::new(),
            research,
            directive_timer: 0.0,
            directive_tier: 0,
            since_save: 0.0,
            background_accumulator: 0.0,
            shipments: Vec::new(),
            toast_history: Vec::new(),
            ending_seen: false,
            notification_cursors: [0; PLANET_COUNT],
        }
    }

    /// Push the campaign's research at every world it holds.
    ///
    /// All of them, not just the one in front of the player: the others are
    /// being simulated, and a world running on research it does not know about
    /// produces the wrong numbers quietly.
    pub fn sync_research(&mut self) {
        let research = self.research.clone();
        for planet in self.planets.iter_mut().flatten() {
            planet.adopt_research(&research);
        }
    }

    /// Continue the active research job on the foreground world after a save
    /// gap. Offline resource gains are applied before this method, matching
    /// the live loop where a world's production feeds research afterward.
    /// Returns the Data spent and whether the job completed.
    pub fn apply_offline_research(&mut self, simulated_seconds: f32) -> (f32, bool) {
        if simulated_seconds <= 0.0 || !simulated_seconds.is_finite() {
            return (0.0, false);
        }
        let Some(current_id) = self.research.current_research.clone() else {
            return (0.0, false);
        };
        let tree = ResearchTree::from_data(self.data.data());
        let Some(node) = tree.get_node(&current_id).cloned() else {
            return (0.0, false);
        };
        let condition = self.current().active_planet_condition();
        if self.current().research_lock_timer > 0.0
            || !tree.can_select_on(&current_id, &self.research.unlocked_techs, condition)
        {
            return (0.0, false);
        }

        let remaining = (node.data_cost - self.research.research_progress).max(0.0);
        if remaining <= 0.0 {
            self.research.current_research = None;
            self.research.research_progress = 0.0;
            self.research.parked_progress.remove(&current_id);
            if !self
                .research
                .unlocked_techs
                .iter()
                .any(|id| id == &current_id)
            {
                self.research.unlocked_techs.push(current_id);
            }
            self.sync_research();
            return (0.0, true);
        }

        let available = self.current().resources.data;
        if available <= 0.0 {
            return (0.0, false);
        }
        let planet = self.current();
        let rate = planet
            .stats
            .apply(StatId::ResearchRate, planet.config.resources.research_rate);
        if rate <= 0.0 || !rate.is_finite() {
            return (0.0, false);
        }
        let spend = (rate * simulated_seconds).min(available).min(remaining);
        if spend <= 0.0 || !spend.is_finite() {
            return (0.0, false);
        }
        let spent = self
            .current_mut()
            .resources
            .consume(ResourceType::Data, spend);
        self.research.research_progress += spent;

        let completed = self.research.research_progress >= node.data_cost;
        if completed {
            self.research.current_research = None;
            self.research.research_progress = 0.0;
            self.research.parked_progress.remove(&current_id);
            if !self
                .research
                .unlocked_techs
                .iter()
                .any(|id| id == &current_id)
            {
                self.research.unlocked_techs.push(current_id);
            }
            let planet = self.current_mut();
            planet
                .notifications
                .success(format!("Research complete: {}", node.name));
            planet.emit_audio(super::audio::AudioEvent::Research);
        }
        self.sync_research();
        (spent, completed)
    }

    /// Merge each world's event stream into the campaign log. The cursors are
    /// transient because a loaded campaign starts by replaying only entries
    /// raised after load; the durable copy is `toast_history`.
    pub fn sync_notification_history(&mut self) {
        for (index, slot) in self.planets.iter().enumerate() {
            let Some(planet) = slot.as_ref() else {
                continue;
            };
            let history = planet.notifications.history();
            let cursor = self.notification_cursors[index].min(history.len());
            self.toast_history.extend(history[cursor..].iter().cloned());
            self.notification_cursors[index] = history.len();
        }
        if self.toast_history.len() > 400 {
            let trim = self.toast_history.len() - 400;
            self.toast_history.drain(0..trim);
        }
    }

    /// Adopt whatever a world was saved with, for a save written before
    /// research was campaign-wide.
    pub fn adopt_planet_research(&mut self) {
        if !self.research.unlocked_techs.is_empty() {
            return;
        }
        self.research = self.current().research.clone();
        self.sync_research();
    }

    /// Run every world the player is *not* standing on.
    ///
    /// The GDD's meta-layer promise is that Planet 1 does not disappear, and a
    /// world that is preserved but frozen only half keeps it: come back and
    /// nothing has happened. These run the same simulation the foreground
    /// world does, in coarse one-second steps and without visuals, which is
    /// cheap enough for four planets and avoids a second economy that could
    /// disagree with the real one.
    pub fn update_background(&mut self, delta_time: f32) {
        if delta_time <= 0.0 || !delta_time.is_finite() {
            return;
        }
        self.background_accumulator += delta_time;

        let mut ticks = 0;
        while self.background_accumulator >= BACKGROUND_TICK_SECONDS && ticks < MAX_BACKGROUND_TICKS
        {
            self.background_accumulator -= BACKGROUND_TICK_SECONDS;
            ticks += 1;
        }
        if ticks == MAX_BACKGROUND_TICKS {
            self.background_accumulator = 0.0;
        }
        if ticks == 0 {
            return;
        }

        let current = self.current;
        for (index, slot) in self.planets.iter_mut().enumerate() {
            if index == current {
                continue;
            }
            let Some(planet) = slot.as_mut() else {
                continue;
            };
            for _ in 0..ticks {
                planet.step(BACKGROUND_TICK_SECONDS, false);
            }
        }
    }

    /// Minerals stockpiled on a world, for the map to show what is waiting.
    pub fn stockpile(&self, index: usize) -> Option<f32> {
        self.planets
            .get(index)
            .and_then(Option::as_ref)
            .map(|planet| planet.resources.minerals)
    }

    /// Read the operational pulse of a colonized world without travelling to
    /// it. The map uses the same observed production windows as the planetary
    /// HUD, so a background world can say that it is quiet or bottlenecked
    /// rather than pretending its theoretical capacity is live output.
    pub fn world_telemetry(&self, index: usize) -> Option<WorldTelemetry> {
        let planet = self.planet(index)?;
        let power_alert = planet.power_balance < 0.0;
        let hibernating = planet.battery_seconds <= 0.0;
        let collapse = planet.power_collapse_shutdown > 0.0;
        let stalled = planet.stalled_drone_count() > 0;
        let starved = !planet.starved_factories().is_empty();
        let blocked = !planet.blocked_factories().is_empty();
        let storage_full_resource = [
            (ResourceType::Minerals, planet.mineral_output_rate()),
            (ResourceType::Data, planet.data_generation_rate()),
            (ResourceType::Biomass, planet.biomass_harvest_rate()),
            (ResourceType::Alloy, planet.alloy_rate()),
            (ResourceType::Components, planet.components_rate()),
        ]
        .into_iter()
        .find_map(|(resource, rate)| {
            (rate > 0.0 && planet.resource_is_full(resource)).then_some(resource)
        });
        let storage_full = storage_full_resource.is_some();
        let maintenance_cause = planet.upkeep_alert_cause();
        let maintenance = maintenance_cause.is_some();
        let alerts = usize::from(power_alert)
            + usize::from(hibernating)
            + usize::from(collapse)
            + usize::from(stalled)
            + usize::from(starved)
            + usize::from(blocked)
            + usize::from(storage_full)
            + usize::from(maintenance);
        let alert_kind = if collapse {
            WorldAlert::Collapse
        } else if power_alert {
            WorldAlert::Power
        } else if hibernating {
            WorldAlert::Hibernation
        } else if stalled {
            WorldAlert::Stalled
        } else if starved {
            WorldAlert::Starved
        } else if blocked {
            WorldAlert::Blocked
        } else if storage_full {
            WorldAlert::Storage
        } else if maintenance {
            WorldAlert::Maintenance
        } else {
            WorldAlert::Nominal
        };
        Some(WorldTelemetry {
            minerals: planet.resources.minerals,
            data: planet.resources.data,
            biomass: planet.resources.biomass,
            alloy: planet.resources.alloy,
            components: planet.resources.components,
            ore_rate: planet.throughput.last().unwrap_or(0.0),
            data_rate: planet.data_generation_rate(),
            biomass_rate: planet.biomass_harvest_rate(),
            alloy_rate: planet.observed_alloy_rate(),
            components_rate: planet.observed_components_rate(),
            power_balance: planet.power_balance,
            structures: planet.grid.total_buildings(),
            drones: planet.drones.total_count(),
            core_stage: planet.core_stage_index() + 1,
            core_stage_count: planet.core_stages().len(),
            core_next_requirement: planet.next_core_requirement(),
            alerts,
            alert_kind,
            storage_full_resource,
            maintenance_cause,
        })
    }

    /// Enough has happened that the campaign is worth writing down.
    ///
    /// The interval comes from the player's settings rather than a constant,
    /// because how much work someone is willing to risk is their call. Time is
    /// counted in world seconds, so a paused game does not keep writing to
    /// disk and a fast-forwarded one saves as often as the progress warrants.
    pub fn due_for_autosave(&self, interval_seconds: f32) -> bool {
        self.since_save >= interval_seconds.max(1.0)
    }

    /// Called after a successful write, whatever prompted it.
    pub fn mark_saved(&mut self) {
        self.since_save = 0.0;
        let planet = self.current_mut();
        planet.save_notice_timer = SAVE_NOTICE_SECONDS;
        // A successful write promotes a backup recovery into a healthy
        // primary save. Keep the warning visible until that point, then let
        // the normal saved marker replace it.
        planet.restored_from_backup = false;
    }

    /// Give every persisted world the same save boundary. Background worlds
    /// are simulated before the campaign is written, so leaving their older
    /// timestamps in place would replay that same offline interval on reload.
    pub(crate) fn stamp_saved_at(&mut self, unix_seconds: i64) {
        for planet in self.planets.iter_mut().flatten() {
            planet.last_saved_unix = unix_seconds;
        }
    }

    fn generate(&self, index: usize, config: &GameConfig) -> PlanetState {
        // Derived from the campaign seed so a campaign replays identically.
        let planet_seed = self.seed ^ (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        PlanetState::new_with_data(index, planet_seed, config.clone(), self.data.clone())
    }

    pub fn data(&self) -> &GameDataContext {
        &self.data
    }

    pub fn attach_data(&mut self, data: GameDataContext) {
        self.data = data.clone();
        for planet in self.planets.iter_mut().flatten() {
            planet.attach_data(data.clone());
        }
    }

    /// Repair the tutorial scope on campaigns written before later worlds
    /// stopped replaying the opening lesson. Only the starting world owns that
    /// lesson; this is safe to run on every load and needs no schema bump.
    pub(crate) fn normalize_tutorial_scope(&mut self) {
        for (index, planet) in self.planets.iter_mut().enumerate() {
            if index != STARTING_PLANET {
                if let Some(planet) = planet {
                    planet.tutorial_done = true;
                }
            }
        }
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn set_slot_name(&mut self, slot_name: impl Into<String>) {
        self.slot_name = slot_name.into();
    }

    pub fn apply_preferred_speed(&mut self, speed_index: i32) {
        let index = speed_index.clamp(0, (super::TIME_SCALES.len() - 1) as i32) as usize;
        let speed = super::TIME_SCALES[index];
        for planet in self.planets.iter_mut().flatten() {
            planet.time_scale = speed;
        }
    }

    pub fn current(&self) -> &PlanetState {
        self.planets[self.current]
            .as_ref()
            .expect("the current planet is always colonized")
    }

    pub fn current_mut(&mut self) -> &mut PlanetState {
        self.planets[self.current]
            .as_mut()
            .expect("the current planet is always colonized")
    }

    /// The current planet and the active directive together: the planetary
    /// view needs both, and taking them one at a time borrows the whole
    /// campaign twice.
    pub fn current_and_directive(&mut self) -> (&mut PlanetState, &Directive) {
        let planet = self.planets[self.current]
            .as_mut()
            .expect("the current planet is always colonized");
        (planet, &self.directive)
    }

    /// A world by slot, for anything that has to reach past the one in front
    /// of the player - a pod landing on a world nobody is watching.
    pub fn planet(&self, index: usize) -> Option<&PlanetState> {
        self.planets.get(index).and_then(Option::as_ref)
    }

    pub fn planet_mut(&mut self, index: usize) -> Option<&mut PlanetState> {
        self.planets.get_mut(index).and_then(Option::as_mut)
    }

    pub fn is_colonized(&self, index: usize) -> bool {
        self.planets.get(index).is_some_and(Option::is_some)
    }

    pub fn colonized_flags(&self) -> [bool; PLANET_COUNT] {
        std::array::from_fn(|index| self.is_colonized(index))
    }

    /// Land on a new world. It is generated once and kept for good.
    pub fn colonize(&mut self, index: usize) -> bool {
        if index >= PLANET_COUNT || self.is_colonized(index) {
            return false;
        }
        let config = self.current().config.clone();
        let mut planet = self.generate(index, &config);
        // A world arrives knowing everything the swarm knows. Leaving this to
        // the caller is how a freshly colonized world ends up simulating on
        // starting-tier research.
        planet.adopt_research(&self.research);
        // The tutorial is the opening lesson, not a per-world checklist. A
        // later world can ban one of its early building goals (Venus and
        // Saturn both reject Wind Turbines), so restarting it here would trap
        // a successful launch behind an impossible step.
        planet.tutorial_done = true;
        self.planets[index] = Some(planet);
        true
    }

    /// Send the finished Seed Ship to an untouched world, and ride it there.
    ///
    /// This is the only way to reach somewhere the swarm has never been: free
    /// travel works between worlds it already holds, but a new one costs a
    /// whole ship.
    pub fn launch_seed_ship(&mut self, target: usize) -> bool {
        // Research names the technology, but the launch still needs a live
        // piece of infrastructure on the world being left. Without this
        // guard a finished ship could silently bypass the Mass Driver puzzle
        // and carry the campaign away from an unconnected Core.
        if !self.current().seed_ship.is_ready_to_launch()
            || self.current().mass_drivers_online() == 0
        {
            return false;
        }
        if !self.colonize(target) {
            return false;
        }
        self.current_mut().seed_ship.mark_launched();
        self.current = target;
        self.current_mut().arrival_notice_timer = ARRIVAL_NOTICE_SECONDS;
        true
    }

    /// The system is spent: every world taken, and a finished ship with
    /// nowhere left to send it.
    ///
    /// The campaign ends on the ship that has no destination rather than on a
    /// counter reaching five, because that is the moment the loop the whole
    /// game is built on runs out of somewhere to point.
    pub fn is_complete(&self) -> bool {
        self.planets.iter().all(Option::is_some) && self.current().seed_ship.is_ready_to_launch()
    }

    /// Ships sent across the whole campaign, for the ending to count.
    pub fn total_launches(&self) -> u32 {
        self.planets
            .iter()
            .filter_map(Option::as_ref)
            .map(|planet| planet.seed_ship.launches())
            .sum()
    }

    /// World time lived across every planet.
    pub fn total_time_played(&self) -> f64 {
        self.planets
            .iter()
            .filter_map(Option::as_ref)
            .map(|planet| planet.time_played)
            .sum()
    }

    /// Buildings standing across the whole campaign.
    pub fn total_structures(&self) -> usize {
        self.planets
            .iter()
            .filter_map(Option::as_ref)
            .map(|planet| planet.grid.total_buildings())
            .sum()
    }

    /// Lifetime resource sinks across every world the swarm has reached.
    pub fn total_consumption(&self) -> ConsumptionTotals {
        self.planets
            .iter()
            .filter_map(Option::as_ref)
            .map(|planet| planet.resources.consumed)
            .fold(ConsumptionTotals::default(), |mut total, consumed| {
                total.add(consumed);
                total
            })
    }

    /// Move the swarm's attention to another colonized world. The world it
    /// leaves keeps everything it had.
    pub fn travel_to(&mut self, index: usize) -> bool {
        if !self.is_colonized(index) || index == self.current {
            return false;
        }
        self.current = index;
        self.current_mut().arrival_notice_timer = ARRIVAL_NOTICE_SECONDS;
        true
    }

    /// Tick the active directive against the current planet, rotating to the
    /// next one when it is done or its window has run out.
    pub fn update_directive(&mut self, delta_time: f32) {
        self.since_save += delta_time;
        self.directive_timer += delta_time;
        let expired = self.directive_timer >= crate::directives::rotation_seconds(self.data.data())
            || self.directive.duration <= 0.0
            || self.directive.completed;

        if expired {
            let reward = if self.directive.completed {
                self.directive.reward_data
            } else {
                0.0
            };
            // A directive used to pay out and rotate without a word, so the
            // only evidence was the panel quietly saying something else.
            let announcement = if self.directive.completed {
                Some(format!(
                    "Directive met: {} (+{:.0} Data)",
                    self.directive.description, reward
                ))
            } else if self.directive.duration > 0.0 {
                Some(format!("Directive lapsed: {}", self.directive.description))
            } else {
                None
            };
            let completed = self.directive.completed;
            self.directive_history.push(DirectiveRecord {
                description: self.directive.description.clone(),
                completed,
                reward_data: reward,
                tier: self.directive_tier,
            });
            if self.directive_history.len() > 100 {
                self.directive_history.remove(0);
            }
            if let Some(message) = announcement {
                let planet = self.current_mut();
                planet.emit_audio(super::audio::AudioEvent::Directive);
                if completed {
                    planet.notifications.success(message);
                } else {
                    planet.notifications.warning(message);
                }
            }
            if reward > 0.0 {
                self.current_mut().resources.data += reward;
            }
            self.directive_timer = 0.0;
            self.directive_tier += 1;
            self.directive = pick_directive(self.data.data(), self.directive_tier);
        }

        let Some(planet) = self.planets[self.current].as_ref() else {
            return;
        };
        self.directive.update(planet, delta_time);
    }
}

#[cfg(test)]
mod tests;
