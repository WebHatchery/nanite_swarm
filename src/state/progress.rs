use crate::engine::{BuildingType, GridPos, StatId};

use super::game_state::PlanetState;

/// Offline catch-up step. Coarser than the live tick by design — see
/// [`apply_offline_progress`](PlanetState::apply_offline_progress).
const OFFLINE_TICK_SECONDS: f32 = 1.0;

impl PlanetState {
    /// Select a building type for placement.
    pub fn select_building(&mut self, building_type: BuildingType) {
        if self.is_building_unlocked(building_type) {
            self.selected_building = Some(building_type);
            // Building and demolishing are the same click, so they cannot both
            // be armed.
            self.demolish_mode = false;
        }
    }

    /// Arm or disarm demolition. Picking it up puts the build cursor down.
    pub fn toggle_demolish_mode(&mut self) {
        self.demolish_mode = !self.demolish_mode;
        if self.demolish_mode {
            self.selected_building = None;
        }
    }

    /// Clear building selection.
    pub fn clear_selection(&mut self) {
        self.selected_building = None;
    }

    pub fn is_building_unlocked(&self, building_type: BuildingType) -> bool {
        self.is_building_researched(building_type) && !self.is_building_banned(building_type)
    }

    /// Research has opened this building up, whether or not this particular
    /// world will accept it.
    pub fn is_building_researched(&self, building_type: BuildingType) -> bool {
        matches!(building_type, BuildingType::Core)
            || self.unlocked_buildings.contains(&building_type)
    }

    /// This world refuses the building outright, however much research the
    /// swarm has done.
    pub fn is_building_banned(&self, building_type: BuildingType) -> bool {
        self.banned_buildings.contains(&building_type)
    }

    pub fn unlock_building(&mut self, building_type: BuildingType) {
        if !self.unlocked_buildings.contains(&building_type) {
            self.unlocked_buildings.push(building_type);
        }
    }

    pub fn mineral_capacity(&self) -> f32 {
        let storage_count = self.grid.find_buildings(BuildingType::Storage).len() as f32;
        let built = self.config.resources.base_mineral_cap
            + storage_count * self.config.resources.storage_bonus;
        self.stats.apply(StatId::MineralCapacity, built)
    }

    /// Power produced this tick, including whatever the biomass harvesters
    /// burned. Generation and consumption are asked for separately because the
    /// HUD shows both halves.
    pub fn power_generation(&self) -> f32 {
        self.grid.total_power_generation() + self.biomass_power_bonus
    }

    /// Power drawn this tick, after research efficiencies.
    pub fn power_consumption(&self) -> f32 {
        self.stats.apply(
            StatId::PowerConsumption,
            self.grid.total_power_consumption(),
        )
    }

    pub fn net_power(&self) -> f32 {
        self.power_generation() - self.power_consumption()
    }

    /// Acid still reaching the network after research counters it.
    pub fn acid_strength(&self) -> f32 {
        (self.hazards.acid_rain * self.stats.multiplier(StatId::AcidResistance)).max(0.0)
    }

    /// Share of drone speed the cold still takes, after research. Capped so a
    /// world can slow the swarm to a crawl but never stop it dead.
    pub fn freeze_strength(&self) -> f32 {
        (self.hazards.freeze * self.stats.multiplier(StatId::FreezeResistance)).clamp(0.0, 0.9)
    }

    /// Alloy the working smelters would produce per second, for the readout.
    pub fn alloy_rate(&self) -> f32 {
        crate::data::game_data()
            .buildings
            .iter()
            .filter(|def| def.recipe.alloy_out > 0.0)
            .filter_map(|def| {
                BuildingType::from_id(&def.id).map(|kind| (kind, def.recipe.alloy_out))
            })
            .map(|(kind, out)| {
                self.grid
                    .find_buildings(kind)
                    .into_iter()
                    .filter_map(|pos| self.grid.get(pos).and_then(|tile| tile.building.as_ref()))
                    .filter(|building| building.powered && !building.is_dust_stalled())
                    .map(|building| out * building.dust_efficiency())
                    .sum::<f32>()
            })
            .sum()
    }

    /// Stop and start the world.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        // Drop the part-finished step: resuming should not owe time.
        self.sim_accumulator = 0.0;
    }

    /// Step to the next speed up or down the ladder, clamped at both ends.
    pub fn change_speed(&mut self, faster: bool) {
        let scales = crate::state::TIME_SCALES;
        let current = scales
            .iter()
            .position(|scale| (*scale - self.time_scale).abs() < 1e-3)
            .unwrap_or(1);
        let next = if faster {
            (current + 1).min(scales.len() - 1)
        } else {
            current.saturating_sub(1)
        };
        self.time_scale = scales[next];
    }

    /// A short label for whatever this world is doing to the machinery, for
    /// the HUD. Empty when the world is merely somewhere to build.
    pub fn hazard_label(&self) -> String {
        let mut parts = Vec::new();
        if self.acid_strength() > 0.0 {
            parts.push("ACID RAIN");
        }
        if self.freeze_strength() > 0.0 {
            parts.push("DEEP FREEZE");
        }
        parts.join(" / ")
    }

    /// What this world has to say for itself, shown on arrival.
    pub fn arrival_line(&self) -> &'static str {
        crate::data::game_data()
            .planet(self.planet_index)
            .arrival
            .as_str()
    }

    pub fn battery_time_left(&self) -> (i32, i32) {
        let total = self.battery_seconds.max(0.0) as i32;
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        (hours, minutes)
    }

    pub fn apply_offline_progress(&mut self, offline_seconds: f32) {
        if offline_seconds <= 0.0 {
            self.last_offline_seconds = 0.0;
            self.last_offline_simulated = 0.0;
            return;
        }

        // Catch-up runs whole steps like the live loop, just coarser ones:
        // four hours at the live tick rate would be 432,000 steps on load.
        let steps = (offline_seconds / OFFLINE_TICK_SECONDS).floor() as u64;
        for _ in 0..steps {
            self.step(OFFLINE_TICK_SECONDS, false);
        }
        let remainder = offline_seconds - steps as f32 * OFFLINE_TICK_SECONDS;
        if remainder > 0.0 {
            self.step(remainder, false);
        }

        self.last_offline_seconds = offline_seconds;
        let full_speed = offline_seconds.min(4.0 * 60.0 * 60.0);
        let hibernation = (offline_seconds - full_speed).max(0.0) * 0.1;
        self.last_offline_simulated = full_speed + hibernation;
        self.offline_notice_timer = 8.0;
    }

    pub fn achievements_progress(&self) -> (usize, usize) {
        self.achievements.progress()
    }

    pub fn placement_scale(&self, pos: GridPos) -> f32 {
        let Some(anim) = self
            .placement_anims
            .iter()
            .find(|anim| anim.position == pos)
        else {
            return 1.0;
        };
        let progress = (anim.timer / 0.3).clamp(0.0, 1.0);
        let bounce_phase = (1.0 - progress) * std::f32::consts::PI * 2.0;
        1.0 + (bounce_phase.sin() * 0.12)
    }

    pub(super) fn update_achievements(&mut self) {
        let has_drill = !self.grid.find_buildings(BuildingType::Drill).is_empty();
        if has_drill {
            self.announce_achievement("first_drill");
        }

        if self.power_balance > 0.0 {
            self.announce_achievement("power_surplus");
        }

        if self.resources.data >= 25.0 {
            self.announce_achievement("data_miner");
        }

        if self.grid.total_buildings() >= 10 {
            self.announce_achievement("builder");
        }
    }

    /// Unlock an achievement, and say so if it had not already fired.
    ///
    /// `unlock` returns whether this was the moment it happened, which is the
    /// only reason the toast is not repeated every tick.
    pub fn announce_achievement(&mut self, id: &str) {
        if !self.achievements.unlock(id) {
            return;
        }
        let name = self
            .achievements
            .get(id)
            .map(|achievement| achievement.name.clone())
            .unwrap_or_else(|| id.to_string());
        self.notifications.success(format!("Achievement: {}", name));
    }
}

#[cfg(test)]
mod tests {
    use super::PlanetState;
    use crate::engine::BuildingType;

    fn approx_eq(a: f32, b: f32, eps: f32) -> bool {
        (a - b).abs() <= eps
    }

    #[test]
    fn offline_simulation_uses_hibernation_rate() {
        let mut state = PlanetState {
            battery_seconds: 4.0 * 60.0 * 60.0,
            ..Default::default()
        };

        let offline = 6.0 * 60.0 * 60.0;
        state.apply_offline_progress(offline);

        let expected_sim = (4.0 * 60.0 * 60.0) + (2.0 * 60.0 * 60.0) * 0.1;
        assert!(approx_eq(state.last_offline_simulated, expected_sim, 0.5));
        assert!(approx_eq(state.last_offline_seconds, offline, 0.5));
        assert!(state.battery_seconds <= 0.0);
        assert!(state.offline_notice_timer > 0.0);
    }

    #[test]
    fn apply_offline_progress_is_noop_for_zero_or_negative_duration() {
        let mut state = PlanetState::default();
        state.apply_offline_progress(0.0);
        assert_eq!(state.last_offline_seconds, 0.0);
        assert_eq!(state.last_offline_simulated, 0.0);
    }

    #[test]
    fn core_is_always_unlocked_others_require_explicit_unlock() {
        let mut state = PlanetState::default();
        assert!(state.is_building_unlocked(BuildingType::Core));
        assert!(!state.is_building_unlocked(BuildingType::Conduit));
        state.unlock_building(BuildingType::Conduit);
        assert!(state.is_building_unlocked(BuildingType::Conduit));
    }

    #[test]
    fn unlock_building_does_not_duplicate_entries() {
        let mut state = PlanetState::default();
        state.unlock_building(BuildingType::Storage);
        state.unlock_building(BuildingType::Storage);
        assert_eq!(
            state
                .unlocked_buildings
                .iter()
                .filter(|b| **b == BuildingType::Storage)
                .count(),
            1
        );
    }

    /// Every tech that claims a number in `research.json` has to move one here.
    /// These are the five that shipped as no-ops.
    #[test]
    fn researching_storage_optimization_raises_the_mineral_cap() {
        let mut state = PlanetState::default();
        let before = state.mineral_capacity();
        state
            .research
            .unlocked_techs
            .push("storage_optimization".into());
        state.refresh_stats();
        assert_eq!(state.mineral_capacity(), before + 50.0);
    }

    #[test]
    fn researching_power_efficiency_lowers_consumption() {
        let mut state = PlanetState::default();
        let core = state.grid.find_core().unwrap();
        let pos = crate::engine::GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);

        let before = state.power_consumption();
        assert!(before > 0.0);
        state
            .research
            .unlocked_techs
            .push("power_efficiency".into());
        state.refresh_stats();
        assert!(approx_eq(state.power_consumption(), before * 0.75, 1e-4));
    }

    #[test]
    fn researching_drone_capacity_grows_existing_drones_too() {
        let mut state = PlanetState::default();
        let core = state.grid.find_core().unwrap();
        let pos = crate::engine::GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);
        let before = state.drones.drone_capacity;

        state.research.unlocked_techs.push("drone_capacity".into());
        state.refresh_stats();

        assert_eq!(state.drones.drone_capacity, before * 2.0);
        assert_eq!(state.drones.drones()[0].capacity, before * 2.0);
    }

    #[test]
    fn researching_efficient_drills_and_advanced_research_speeds_production() {
        let mut plain = PlanetState::default();
        let mut upgraded = PlanetState::default();
        upgraded
            .research
            .unlocked_techs
            .extend(["efficient_drills".into(), "advanced_research".into()]);
        upgraded.refresh_stats();

        for state in [&mut plain, &mut upgraded] {
            let core = state.grid.find_core().unwrap();
            let pos = crate::engine::GridPos::new(core.x + 1, core.y);
            state.grid.reveal_around(pos, 1);
            state.select_building(BuildingType::Drill);
            state.try_place_building(pos);
            state.resources.minerals = 0.0;
            // Both runs would otherwise clamp at the storage cap.
            state.config.resources.base_mineral_cap = 100_000.0;
            for _ in 0..300 {
                state.step(0.1, false);
            }
        }

        assert!(upgraded.resources.minerals > plain.resources.minerals);
        assert!(upgraded.resources.data > plain.resources.data);
    }

    #[test]
    fn mineral_capacity_grows_with_storage_buildings() {
        let mut state = PlanetState::default();
        let base = state.mineral_capacity();
        let core = state.grid.find_core().unwrap();
        let pos = crate::engine::GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.unlock_building(BuildingType::Storage);
        state.select_building(BuildingType::Storage);
        state.try_place_building(pos);
        assert!(state.mineral_capacity() > base);
    }

    #[test]
    fn battery_time_left_converts_seconds_to_hours_and_minutes() {
        let state = PlanetState {
            battery_seconds: 3661.0, // 1h 1m 1s
            ..Default::default()
        };
        assert_eq!(state.battery_time_left(), (1, 1));
    }

    #[test]
    fn an_achievement_announces_itself_once_and_not_again() {
        let mut state = PlanetState::default();
        assert!(state.notifications.is_empty());

        state.announce_achievement("first_drill");
        assert_eq!(state.notifications.count(), 1);
        let message = state.notifications.get_notifications()[0].message.clone();
        assert!(message.contains("First Drill"), "{message}");

        // Ticking on does not re-announce something already unlocked.
        state.announce_achievement("first_drill");
        assert_eq!(state.notifications.count(), 1);
    }

    #[test]
    fn toasts_fade_in_real_time_even_while_the_world_is_paused() {
        let mut state = PlanetState::default();
        state.announce_achievement("first_drill");
        // `opacity` only moves in the toast's last second, so the timer itself
        // is the thing to watch.
        let before = state.notifications.get_notifications()[0].progress();

        state.toggle_pause();
        for _ in 0..20 {
            assert_eq!(state.advance(0.1, false), 0, "the world moved while paused");
        }

        let after = state.notifications.get_notifications()[0].progress();
        assert!(
            after > before,
            "the toast froze with the world: {before} then {after}"
        );
    }

    #[test]
    fn achievements_progress_unlocks_first_drill_and_power_surplus() {
        let mut state = PlanetState::default();
        let (unlocked_before, total) = state.achievements_progress();
        assert_eq!(unlocked_before, 0);

        let core = state.grid.find_core().unwrap();
        let pos = crate::engine::GridPos::new(core.x + 1, core.y);
        state.grid.reveal_around(pos, 1);
        state.select_building(BuildingType::Drill);
        state.try_place_building(pos);

        let (unlocked_after, total_after) = state.achievements_progress();
        assert!(unlocked_after > unlocked_before);
        assert_eq!(total_after, total);
    }
}
