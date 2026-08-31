//! Planet-state integration for the Seed Ship yard.

use crate::state::game_state::PlanetState;

impl PlanetState {
    /// Pour this tick's production into the Seed Ship yard, if the swarm has
    /// been told to.
    pub fn update_seed_ship(&mut self, delta_time: f32) {
        if !self.seed_ship.committed || self.seed_ship.is_complete() {
            return;
        }
        // A stage nobody has worked out how to build yet takes nothing: the
        // yard sits idle rather than quietly banking resources against it.
        if self
            .seed_ship
            .blocked_by(&self.research.unlocked_techs)
            .is_some()
        {
            return;
        }

        let intake = self.data().seed_ship.intake_per_second;
        let finished_stage = self
            .seed_ship
            .absorb(&mut self.resources, intake, delta_time);
        if !finished_stage {
            return;
        }

        // A finished stage changes what this world can do, so the stat sheet
        // has to be rebuilt before anything reads it again.
        self.refresh_stats();

        if self.seed_ship.is_complete() {
            self.notifications
                .success("Seed Ship complete. This world is spent.");
            self.announce_achievement("seed_ship");
            return;
        }

        // Report the boon of the stage just finished, not the next one's name:
        // what the player gained is the interesting half.
        let built = self.seed_ship.standing_stages();
        match built.last().filter(|stage| !stage.boon.is_empty()) {
            Some(stage) => self
                .notifications
                .success(format!("{} standing. {}", stage.name, stage.boon)),
            None => {
                if let Some(stage) = self.seed_ship.stage() {
                    self.notifications
                        .info(format!("Seed Ship: {} under way", stage.name));
                }
            }
        }
    }

    /// Replay the committed yard during a save/load gap using the same capped
    /// intake as live simulation. One-second slices let a stage hand its
    /// leftover time to the next stage without making a large offline delta
    /// pay every stage in full.
    pub fn apply_offline_seed_ship_progress(&mut self, simulated_seconds: f32) -> u32 {
        if simulated_seconds <= 0.0 || !simulated_seconds.is_finite() {
            return 0;
        }

        let mut remaining = simulated_seconds;
        let mut completed = 0;
        while remaining > 0.0001 && self.seed_ship.committed && !self.seed_ship.is_complete() {
            let slice = remaining.min(1.0);
            let before_stage = self.seed_ship.stage_index();
            let before_progress = self.seed_ship.progress();
            self.update_seed_ship(slice);
            completed += self.seed_ship.stage_index().saturating_sub(before_stage) as u32;

            // A blocked or resource-starved yard cannot make progress for the
            // rest of this offline interval. Avoid burning through all 24
            // hours one empty second at a time.
            if self.seed_ship.stage_index() == before_stage
                && self.seed_ship.progress() == before_progress
            {
                break;
            }
            remaining -= slice;
        }
        completed
    }

    /// The research the yard is waiting on, and what it is called, for the
    /// ship screen to say so.
    pub fn seed_ship_blocked_by(&self) -> Option<&str> {
        let id = self.seed_ship.blocked_by(&self.research.unlocked_techs)?;
        self.data()
            .research
            .nodes
            .iter()
            .find(|node| node.id == id)
            .map(|node| node.name.as_str())
    }

    /// Stop or start diverting production into the yard.
    pub fn toggle_seed_ship_commitment(&mut self) {
        if self.seed_ship.is_complete() {
            return;
        }
        self.seed_ship.committed = !self.seed_ship.committed;
    }
}
