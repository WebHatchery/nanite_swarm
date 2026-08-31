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

use super::consumption::ConsumptionTotals;
use super::game_state::{PlanetState, ResearchProgress};

mod construction;
mod persistence;
mod research;
mod simulation;
mod telemetry;
mod validation;

pub use telemetry::{WorldAlert, WorldTelemetry};

pub const PLANET_COUNT: usize = 5;
/// Mars, per the GDD's Zone 1.
pub const STARTING_PLANET: usize = 2;

/// How long a world's arrival line stays on screen.
const ARRIVAL_NOTICE_SECONDS: f32 = 10.0;
/// How long the saved marker stays on screen.
const SAVE_NOTICE_SECONDS: f32 = 3.0;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirectiveRecord {
    pub description: String,
    pub completed: bool,
    pub reward_data: f32,
    pub tier: i32,
}

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
}

#[cfg(test)]
mod tests;
