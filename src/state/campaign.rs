//! The solar campaign: every world the swarm has landed on.
//!
//! The GDD's meta-layer rests on one promise — "when the player moves to
//! Planet 2, Planet 1 does not disappear". So a planet is not rebuilt on
//! arrival: it lives in its slot for the whole campaign, and travelling only
//! changes which slot is in front of the player.

use macroquad_toolkit::notifications::LoggedNotification;
use serde::{Deserialize, Serialize};

use crate::data::GameConfig;
use crate::directives::{pick_directive, Directive};

use super::game_state::{PlanetState, ResearchProgress};

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
    #[serde(skip, default)]
    notification_cursors: [usize; PLANET_COUNT],
}

fn default_slot_name() -> String {
    "slot_1".to_string()
}

impl Campaign {
    /// Start a campaign: one colonized world, the rest untouched.
    pub fn new(config: GameConfig, seed: u64) -> Self {
        let mut campaign = Self {
            slot_name: default_slot_name(),
            planets: std::array::from_fn(|_| None),
            current: STARTING_PLANET,
            seed,
            directive: pick_directive(0),
            directive_history: Vec::new(),
            research: ResearchProgress::default(),
            directive_timer: 0.0,
            directive_tier: 0,
            since_save: 0.0,
            background_accumulator: 0.0,
            shipments: Vec::new(),
            toast_history: Vec::new(),
            notification_cursors: [0; PLANET_COUNT],
        };
        campaign.planets[STARTING_PLANET] = Some(campaign.generate(STARTING_PLANET, &config));
        campaign
    }

    /// Wrap a single planet as a campaign, for a save written before the
    /// campaign existed.
    pub fn from_single_planet(planet: PlanetState, seed: u64) -> Self {
        let mut planets: [Option<PlanetState>; PLANET_COUNT] = std::array::from_fn(|_| None);
        planets[STARTING_PLANET] = Some(planet);
        Self {
            slot_name: default_slot_name(),
            planets,
            current: STARTING_PLANET,
            seed,
            directive: pick_directive(0),
            directive_history: Vec::new(),
            research: ResearchProgress::default(),
            directive_timer: 0.0,
            directive_tier: 0,
            since_save: 0.0,
            background_accumulator: 0.0,
            shipments: Vec::new(),
            toast_history: Vec::new(),
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
        self.current_mut().save_notice_timer = SAVE_NOTICE_SECONDS;
    }

    fn generate(&self, index: usize, config: &GameConfig) -> PlanetState {
        // Derived from the campaign seed so a campaign replays identically.
        let planet_seed = self.seed ^ (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        PlanetState::new(index, planet_seed, config.clone())
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
        self.planets[index] = Some(planet);
        true
    }

    /// Send the finished Seed Ship to an untouched world, and ride it there.
    ///
    /// This is the only way to reach somewhere the swarm has never been: free
    /// travel works between worlds it already holds, but a new one costs a
    /// whole ship.
    pub fn launch_seed_ship(&mut self, target: usize) -> bool {
        if !self.current().seed_ship.is_ready_to_launch() {
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
        let expired = self.directive_timer >= crate::directives::rotation_seconds()
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
            self.directive = pick_directive(self.directive_tier);
        }

        let Some(planet) = self.planets[self.current].as_ref() else {
            return;
        };
        self.directive.update(planet, delta_time);
    }
}

#[cfg(test)]
mod tests;
