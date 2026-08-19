//! Runtime research progression and campaign synchronization.

use crate::{data, engine, state, Game};

impl Game {
    pub(super) fn update_research(&mut self, delta_time: f32) {
        let Some(current_id) = self.research_state.current_research.clone() else {
            self.sync_building_unlocks();
            self.sync_research_to_planet();
            return;
        };
        if self.campaign.current().research_lock_timer > 0.0 {
            return;
        }
        let Some(node) = self.research_tree.get_node(&current_id) else {
            self.research_state.current_research = None;
            self.research_state.research_progress = 0.0;
            self.sync_research_to_planet();
            return;
        };
        if !self.research_tree.can_select_on(
            &current_id,
            &self.research_state.unlocked,
            self.campaign.current().active_planet_condition(),
        ) {
            self.research_state.current_research = None;
            self.research_state.research_progress = 0.0;
            self.campaign
                .current_mut()
                .notifications
                .warning("Research branch unavailable on this world");
            self.sync_research_to_planet();
            return;
        }
        let remaining = (node.data_cost - self.research_state.research_progress).max(0.0);
        if remaining <= 0.0 {
            let name = node.name.clone();
            self.research_state.complete_research();
            self.announce_research(&name);
            self.sync_building_unlocks();
            self.sync_research_to_planet();
            return;
        }
        let available = self.campaign.current().resources.data;
        if available <= 0.0 {
            return;
        }
        let planet = self.campaign.current();
        let rate = planet.stats.apply(
            engine::StatId::ResearchRate,
            planet.config.resources.research_rate,
        );
        let spend = (rate * delta_time).min(available).min(remaining);
        self.campaign.current_mut().resources.data -= spend;
        self.research_state.research_progress += spend;
        if self.research_state.research_progress >= node.data_cost {
            let name = node.name.clone();
            self.research_state.complete_research();
            self.announce_research(&name);
        }
        self.sync_building_unlocks();
        self.sync_research_to_planet();
    }

    fn announce_research(&mut self, name: &str) {
        let planet = self.campaign.current_mut();
        planet
            .notifications
            .success(format!("Research complete: {}", name));
        planet.emit_audio(state::AudioEvent::Research);
    }

    pub(super) fn sync_research_from_planet(&mut self) {
        self.research_state.unlocked = self.campaign.research.unlocked_techs.clone();
        for tech in &data::game_data().research.starting_unlocked {
            if !self.research_state.unlocked.contains(tech) {
                self.research_state.unlocked.push(tech.clone());
            }
        }
        self.research_state.current_research = self.campaign.research.current_research.clone();
        self.research_state.research_progress = self.campaign.research.research_progress;
    }

    pub(super) fn sync_research_to_planet(&mut self) {
        self.campaign.research.unlocked_techs = self.research_state.unlocked.clone();
        self.campaign.research.current_research = self.research_state.current_research.clone();
        self.campaign.research.research_progress = self.research_state.research_progress;
        self.campaign.sync_research();
    }

    pub(super) fn sync_building_unlocks(&mut self) {
        let before: Vec<engine::BuildingType> = data::game_data()
            .buildings
            .iter()
            .filter_map(|def| engine::BuildingType::from_id(&def.id))
            .filter(|kind| self.campaign.current().is_building_researched(*kind))
            .collect();
        self.campaign.sync_research();
        for def in &data::game_data().buildings {
            let Some(building_type) = engine::BuildingType::from_id(&def.id) else {
                continue;
            };
            if def.start_unlocked || before.contains(&building_type) {
                continue;
            }
            let planet = self.campaign.current_mut();
            if planet.is_building_researched(building_type)
                && !planet.is_building_banned(building_type)
            {
                planet
                    .notifications
                    .info(format!("Available: {}", def.name));
            }
        }
    }
}
