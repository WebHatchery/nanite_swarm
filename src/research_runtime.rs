//! Runtime research progression and campaign synchronization.

use crate::{engine, state, Game};

impl Game {
    pub(super) fn update_research(&mut self, delta_time: f32) {
        let Some(current_id) = self.research_state.current_research.clone() else {
            self.sync_building_unlocks();
            self.sync_research_to_planet();
            return;
        };
        if self.campaign.current().research_lock_timer > 0.0 {
            // Collapse mutates the foreground world's persisted progress.
            // Pull that loss into the runtime mirror before the next research
            // tick can write the old value back over it.
            let (current_research, research_progress, parked_progress) = {
                let planet_research = &self.campaign.current().research;
                (
                    planet_research.current_research.clone(),
                    planet_research.research_progress,
                    planet_research.parked_progress.clone(),
                )
            };
            self.research_state.current_research = current_research;
            self.research_state.research_progress = research_progress;
            self.research_state.parked_progress = parked_progress;
            self.sync_research_to_planet();
            return;
        }
        let Some(node) = self.research_tree.get_node(&current_id).cloned() else {
            self.research_state.pause_research();
            self.sync_research_to_planet();
            return;
        };
        if !self.research_tree.can_select_on(
            &current_id,
            &self.research_state.unlocked,
            self.campaign.current().active_planet_condition(),
        ) {
            self.research_state.pause_research();
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
        let spent = self
            .campaign
            .current_mut()
            .resources
            .consume(crate::engine::ResourceType::Data, spend);
        self.research_state.research_progress += spent;
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
        let starting_unlocked = self.data.data().research.starting_unlocked.clone();
        for tech in &starting_unlocked {
            if !self.research_state.unlocked.contains(tech) {
                self.research_state.unlocked.push(tech.clone());
            }
        }
        self.research_state.current_research = self.campaign.research.current_research.clone();
        self.research_state.research_progress = self.campaign.research.research_progress;
        self.research_state.parked_progress = self.campaign.research.parked_progress.clone();
    }

    pub(super) fn sync_research_to_planet(&mut self) {
        self.campaign.research.unlocked_techs = self.research_state.unlocked.clone();
        self.campaign.research.current_research = self.research_state.current_research.clone();
        self.campaign.research.research_progress = self.research_state.research_progress;
        self.campaign.research.parked_progress = self.research_state.parked_progress.clone();
        self.campaign.sync_research();
    }

    pub(super) fn sync_building_unlocks(&mut self) {
        let before: Vec<engine::BuildingType> = self
            .data
            .data()
            .buildings
            .iter()
            .filter_map(|def| engine::BuildingType::from_id(&def.id))
            .filter(|kind| self.campaign.current().is_building_researched(*kind))
            .collect();
        self.campaign.sync_research();
        let available_buildings: Vec<_> = self
            .data
            .data()
            .buildings
            .iter()
            .filter_map(|def| engine::BuildingType::from_id(&def.id).map(|kind| (def, kind)))
            .filter(|(def, kind)| !def.start_unlocked && !before.contains(kind))
            .map(|(def, kind)| (def.name.clone(), kind))
            .collect();
        for (name, building_type) in available_buildings {
            let planet = self.campaign.current_mut();
            if planet.is_building_researched(building_type)
                && !planet.is_building_banned(building_type)
            {
                planet.notifications.info(format!("Available: {name}"));
            }
        }
    }
}
