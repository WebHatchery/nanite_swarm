use crate::engine::{ResearchTree, ResourceType, StatId};
use crate::state::audio::AudioEvent;

use super::Campaign;

impl Campaign {
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
            planet.emit_audio(AudioEvent::Research);
        }
        self.sync_research();
        (spent, completed)
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
}
