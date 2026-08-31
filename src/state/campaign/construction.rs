//! Campaign construction and session data reattachment.

use super::*;

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

    pub(super) fn generate(&self, index: usize, config: &GameConfig) -> PlanetState {
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
}
