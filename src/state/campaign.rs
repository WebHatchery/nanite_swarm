//! The solar campaign: every world the swarm has landed on.
//!
//! The GDD's meta-layer rests on one promise — "when the player moves to
//! Planet 2, Planet 1 does not disappear". So a planet is not rebuilt on
//! arrival: it lives in its slot for the whole campaign, and travelling only
//! changes which slot is in front of the player.

use serde::{Deserialize, Serialize};

use crate::data::GameConfig;
use crate::directives::{pick_directive, Directive};

use super::game_state::PlanetState;

pub const PLANET_COUNT: usize = 5;
/// Mars, per the GDD's Zone 1.
pub const STARTING_PLANET: usize = 2;

const DIRECTIVE_ROTATION_SECONDS: f32 = 600.0;
/// How long a world's arrival line stays on screen.
const ARRIVAL_NOTICE_SECONDS: f32 = 10.0;

/// Every planet the swarm holds, plus which one it is currently standing on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Campaign {
    planets: [Option<PlanetState>; PLANET_COUNT],
    current: usize,
    seed: u64,
    pub directive: Directive,
    directive_timer: f32,
    directive_tier: i32,
}

impl Campaign {
    /// Start a campaign: one colonized world, the rest untouched.
    pub fn new(config: GameConfig, seed: u64) -> Self {
        let mut campaign = Self {
            planets: std::array::from_fn(|_| None),
            current: STARTING_PLANET,
            seed,
            directive: pick_directive(0),
            directive_timer: 0.0,
            directive_tier: 0,
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
            planets,
            current: STARTING_PLANET,
            seed,
            directive: pick_directive(0),
            directive_timer: 0.0,
            directive_tier: 0,
        }
    }

    fn generate(&self, index: usize, config: &GameConfig) -> PlanetState {
        // Derived from the campaign seed so a campaign replays identically.
        let planet_seed = self.seed ^ (index as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        PlanetState::new(index, planet_seed, config.clone())
    }

    pub fn current_index(&self) -> usize {
        self.current
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
        self.planets[index] = Some(self.generate(index, &config));
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
        self.directive_timer += delta_time;
        let expired = self.directive_timer >= DIRECTIVE_ROTATION_SECONDS
            || self.directive.duration <= 0.0
            || self.directive.completed;

        if expired {
            let reward = if self.directive.completed {
                self.directive.reward_data
            } else {
                0.0
            };
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
mod tests {
    use super::*;
    use crate::engine::{BuildingType, GridPos};

    fn campaign() -> Campaign {
        Campaign::new(GameConfig::default(), 42)
    }

    #[test]
    fn a_new_campaign_starts_on_mars_with_nothing_else_colonized() {
        let campaign = campaign();
        assert_eq!(campaign.current_index(), STARTING_PLANET);
        assert_eq!(campaign.current().name, "Mars");
        assert_eq!(
            campaign.colonized_flags(),
            [false, false, true, false, false]
        );
    }

    #[test]
    fn travelling_is_refused_for_a_world_the_swarm_has_not_reached() {
        let mut campaign = campaign();
        assert!(!campaign.travel_to(0));
        assert_eq!(campaign.current_index(), STARTING_PLANET);
    }

    #[test]
    fn a_planet_left_behind_keeps_everything_it_had() {
        let mut campaign = campaign();
        let core = campaign.current().grid.find_core().unwrap();
        let drill = GridPos::new(core.x + 1, core.y);
        campaign.current_mut().grid.reveal_around(drill, 1);
        campaign.current_mut().select_building(BuildingType::Drill);
        assert!(campaign.current_mut().try_place_building(drill));
        campaign.current_mut().resources.minerals = 123.0;

        assert!(campaign.colonize(0));
        assert!(campaign.travel_to(0));
        assert_eq!(campaign.current().name, "Mercury");
        // The new world is untouched...
        assert!(campaign
            .current()
            .grid
            .find_buildings(BuildingType::Drill)
            .is_empty());

        assert!(campaign.travel_to(STARTING_PLANET));
        // ...and Mars is exactly as it was left.
        assert_eq!(campaign.current().resources.minerals, 123.0);
        assert!(campaign
            .current()
            .grid
            .get(drill)
            .unwrap()
            .building
            .is_some());
    }

    #[test]
    fn colonizing_twice_does_not_wipe_the_world() {
        let mut campaign = campaign();
        assert!(campaign.colonize(4));
        campaign.travel_to(4);
        campaign.current_mut().resources.minerals = 77.0;
        campaign.travel_to(STARTING_PLANET);

        assert!(!campaign.colonize(4));
        campaign.travel_to(4);
        assert_eq!(campaign.current().resources.minerals, 77.0);
    }

    #[test]
    fn every_world_is_built_from_its_own_entry_in_the_data() {
        let mut campaign = campaign();
        for index in 0..PLANET_COUNT {
            campaign.colonize(index);
        }
        for index in 0..PLANET_COUNT {
            campaign.travel_to(index);
            let def = crate::data::game_data().planet(index);
            let planet = campaign.current();
            assert_eq!(planet.name, def.name);
            assert_eq!(planet.grid.width, def.width);
            assert_eq!(planet.grid.height, def.height);
        }
    }

    #[test]
    fn a_world_can_refuse_a_building_research_has_already_unlocked() {
        let mut campaign = campaign();
        campaign.colonize(1); // Venus: no wind, nothing to burn.
        campaign.travel_to(1);

        let planet = campaign.current_mut();
        planet.unlock_building(BuildingType::WindTurbine);
        planet.unlock_building(BuildingType::Drill);

        assert!(planet.is_building_banned(BuildingType::WindTurbine));
        assert!(!planet.is_building_unlocked(BuildingType::WindTurbine));
        // Everything else it has researched still works.
        assert!(planet.is_building_unlocked(BuildingType::Drill));

        // And it cannot be selected, let alone built.
        planet.select_building(BuildingType::WindTurbine);
        assert_ne!(planet.selected_building, Some(BuildingType::WindTurbine));
    }

    #[test]
    fn arriving_somewhere_says_what_the_world_is() {
        let mut campaign = campaign();
        assert_eq!(campaign.current().arrival_notice_timer, 0.0);

        campaign.colonize(0);
        assert!(campaign.travel_to(0));

        let planet = campaign.current();
        assert!(planet.arrival_notice_timer > 0.0);
        assert_eq!(
            planet.arrival_line(),
            crate::data::game_data().planet(0).arrival
        );
        assert!(!planet.arrival_line().is_empty());
    }

    #[test]
    fn worlds_differ_in_what_they_are_made_of() {
        let mut campaign = campaign();
        campaign.colonize(0);

        let count_forest = |campaign: &Campaign| {
            campaign
                .current()
                .grid
                .iter_tiles()
                .filter(|(_, tile)| tile.terrain == crate::engine::TerrainType::Forest)
                .count()
        };

        // Mars has forests in its weights; Mercury has none at all.
        let mars = count_forest(&campaign);
        campaign.travel_to(0);
        let mercury = count_forest(&campaign);

        assert!(mars > 0, "Mars generated no forest");
        assert_eq!(mercury, 0, "Mercury generated forest it has no weight for");
    }

    #[test]
    fn each_world_generates_its_own_terrain() {
        let mut campaign = campaign();
        campaign.colonize(0);
        let mars: Vec<_> = campaign
            .current()
            .grid
            .iter_tiles()
            .map(|(_, tile)| tile.terrain)
            .collect();
        campaign.travel_to(0);
        let mercury: Vec<_> = campaign
            .current()
            .grid
            .iter_tiles()
            .map(|(_, tile)| tile.terrain)
            .collect();
        assert_ne!(mars, mercury);
    }

    #[test]
    fn the_same_campaign_seed_lays_out_the_same_worlds() {
        let mut first = Campaign::new(GameConfig::default(), 7);
        let mut second = Campaign::new(GameConfig::default(), 7);
        first.colonize(3);
        second.colonize(3);
        first.travel_to(3);
        second.travel_to(3);

        let terrain = |campaign: &Campaign| -> Vec<_> {
            campaign
                .current()
                .grid
                .iter_tiles()
                .map(|(_, tile)| tile.terrain)
                .collect()
        };
        assert_eq!(terrain(&first), terrain(&second));
    }

    /// Finish the current world's Seed Ship the slow way, through the sim.
    fn build_seed_ship(campaign: &mut Campaign) {
        let planet = campaign.current_mut();
        planet.config.resources.base_mineral_cap = 1_000_000.0;
        planet.resources.minerals = 100_000.0;
        planet.resources.data = 100_000.0;
        planet.resources.biomass = 100_000.0;
        planet.toggle_seed_ship_commitment();
        for _ in 0..2_000 {
            planet.update_seed_ship(1.0);
        }
        assert!(planet.seed_ship.is_ready_to_launch());
    }

    #[test]
    fn an_untouched_world_cannot_be_reached_without_a_ship() {
        let mut campaign = campaign();
        assert!(!campaign.current().seed_ship.is_ready_to_launch());
        assert!(!campaign.launch_seed_ship(0));
        assert!(!campaign.is_colonized(0));
        assert_eq!(campaign.current_index(), STARTING_PLANET);
    }

    #[test]
    fn launching_colonizes_the_target_and_carries_the_swarm_there() {
        let mut campaign = campaign();
        build_seed_ship(&mut campaign);

        assert!(campaign.launch_seed_ship(1));
        assert!(campaign.is_colonized(1));
        assert_eq!(campaign.current_index(), 1);
        assert_eq!(campaign.current().name, "Venus");
    }

    #[test]
    fn the_ship_is_spent_by_the_launch() {
        let mut campaign = campaign();
        build_seed_ship(&mut campaign);
        campaign.launch_seed_ship(1);
        campaign.travel_to(STARTING_PLANET);

        let ship = &campaign.current().seed_ship;
        assert_eq!(ship.launches(), 1);
        assert!(!ship.is_ready_to_launch());
        assert_eq!(ship.stage_index(), 0);
        assert!(!ship.committed);
    }

    #[test]
    fn one_ship_cannot_reach_two_worlds() {
        let mut campaign = campaign();
        build_seed_ship(&mut campaign);
        assert!(campaign.launch_seed_ship(1));
        // Standing on Venus, whose yard is empty.
        assert!(!campaign.launch_seed_ship(3));
        assert!(!campaign.is_colonized(3));
    }

    #[test]
    fn a_launch_at_an_already_colonized_world_is_refused() {
        let mut campaign = campaign();
        build_seed_ship(&mut campaign);
        assert!(!campaign.launch_seed_ship(STARTING_PLANET));
        // The ship is still on the pad.
        assert!(campaign.current().seed_ship.is_ready_to_launch());
    }

    #[test]
    fn a_completed_directive_pays_out_and_rotates() {
        let mut campaign = campaign();
        campaign.directive.completed = true;
        campaign.directive.reward_data = 25.0;
        let before = campaign.current().resources.data;

        campaign.update_directive(0.1);

        assert_eq!(campaign.current().resources.data, before + 25.0);
        assert!(!campaign.directive.completed);
        assert_eq!(campaign.directive_tier, 1);
    }
}
