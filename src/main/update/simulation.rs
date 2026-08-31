//! Shared simulation timing and campaign-level update lifecycle.

use macroquad::prelude::get_frame_time;

use crate::{state, Game, GamePhase};

impl Game {
    /// Advance the world and all campaign-level timers by the same simulated time.
    pub(super) fn advance_simulation(&mut self) {
        let ticks = self.campaign.current_mut().advance(get_frame_time(), true);
        if ticks == 0 {
            return;
        }
        let simulated = ticks as f32 * state::TICK_SECONDS;
        self.advance_campaign_systems(simulated);
        self.finish_simulation_update();
    }

    /// Skip to the next meaningful timer without leaving campaign-level
    /// research, directives, background worlds, or freight behind.
    pub(super) fn fast_forward_to_next_event(&mut self) {
        let Some(event) = self.campaign.current().next_interesting_event() else {
            return;
        };
        let mut remaining = event.seconds;
        while remaining > 0.0001 {
            let delta = remaining.min(state::TICK_SECONDS);
            self.campaign.current_mut().step(delta, false);
            self.advance_campaign_systems(delta);
            remaining -= delta;
        }
        self.campaign.current_mut().sim_accumulator = 0.0;
        self.finish_simulation_update();
    }

    fn advance_campaign_systems(&mut self, simulated: f32) {
        self.update_research(simulated);
        self.campaign.update_directive(simulated);
        self.campaign.update_background(simulated);
        self.campaign.update_shipments(simulated);
        self.campaign.sync_notification_history();
    }

    fn finish_simulation_update(&mut self) {
        self.check_campaign_complete();
        if self
            .campaign
            .due_for_autosave(self.settings.autosave_interval)
        {
            self.save_campaign();
        }
    }

    fn check_campaign_complete(&mut self) {
        if self.campaign.ending_seen || !self.campaign.is_complete() {
            return;
        }
        self.campaign.ending_seen = true;
        self.campaign
            .current_mut()
            .announce_achievement("system_consumed");
        self.save_campaign();
        self.phase = GamePhase::CampaignComplete;
    }
}
