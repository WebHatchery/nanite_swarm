//! Post-landing capture state for the first actionable frame on a new world.

use crate::{Game, GamePhase};

impl Game {
    pub(super) fn stage_arrival_notice_scene(&mut self) {
        self.phase = GamePhase::Playing;
        self.campaign.colonize(1);
        self.campaign.travel_to(1);
        self.campaign.current_mut().arrival_notice_timer = 8.0;
    }
}
