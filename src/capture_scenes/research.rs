//! Research-specific screenshot staging.

use crate::{Game, GamePhase};

impl Game {
    pub(super) fn stage_research_scene(&mut self) {
        self.phase = GamePhase::Research;
        // A node that both moves a stat and opens a building, so the panel
        // shows everything it can say about one.
        // Open the path into the new processor branch while keeping the
        // final capability in progress for the detail panel.
        self.research_state.unlocked.extend(
            [
                "power_grid",
                "data_processing",
                "efficient_drills",
                "drone_capacity",
                "smelting",
                "advanced_research",
                "precision_assembly",
            ]
            .into_iter()
            .map(str::to_string),
        );
        self.research_state.current_research = Some("adaptive_clocking".to_string());
        self.research_state
            .parked_progress
            .insert("power_efficiency".to_string(), 12.0);
        self.sync_research_to_planet();
        // Put the capture on the launch-rail research lesson so the target
        // ring and compact auto-focus are visible in both research layouts.
        let planet = self.campaign.current_mut();
        planet.tutorial_step = 7;
        planet.tutorial_done = false;
        planet.tutorial_hidden = false;
        // Meet the standing directive for real, so the toast on this screen
        // is one the game actually raised.
        self.campaign.directive.completed = true;
        self.campaign.update_directive(0.1);
    }
}
