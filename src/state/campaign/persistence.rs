//! Campaign save timing and durable notification history.

use super::Campaign;

impl Campaign {
    /// Merge each world's event stream into the campaign log. The cursors are
    /// transient because a loaded campaign starts by replaying only entries
    /// raised after load; the durable copy is `toast_history`.
    pub fn sync_notification_history(&mut self) {
        for (index, slot) in self.planets.iter().enumerate() {
            let Some(planet) = slot.as_ref() else {
                continue;
            };
            let history = planet.notifications.history();
            let cursor = self.notification_cursors[index].min(history.len());
            self.toast_history.extend(history[cursor..].iter().cloned());
            self.notification_cursors[index] = history.len();
        }
        if self.toast_history.len() > 400 {
            let trim = self.toast_history.len() - 400;
            self.toast_history.drain(0..trim);
        }
    }

    /// Enough has happened that the campaign is worth writing down.
    ///
    /// The interval comes from the player's settings rather than a constant,
    /// because how much work someone is willing to risk is their call. Time
    /// is counted in world seconds, so a paused game does not keep writing to
    /// disk and a fast-forwarded one saves as often as the progress warrants.
    pub fn due_for_autosave(&self, interval_seconds: f32) -> bool {
        self.since_save >= interval_seconds.max(1.0)
    }

    /// Called after a successful write, whatever prompted it.
    pub fn mark_saved(&mut self) {
        self.since_save = 0.0;
        let planet = self.current_mut();
        planet.save_notice_timer = super::SAVE_NOTICE_SECONDS;
        // A successful write promotes a backup recovery into a healthy
        // primary save. Keep the warning visible until that point, then let
        // the normal saved marker replace it.
        planet.restored_from_backup = false;
    }

    /// Give every persisted world the same save boundary. Background worlds
    /// are simulated before the campaign is written, so leaving their older
    /// timestamps in place would replay that same offline interval on reload.
    pub(crate) fn stamp_saved_at(&mut self, unix_seconds: i64) {
        for planet in self.planets.iter_mut().flatten() {
            planet.last_saved_unix = unix_seconds;
        }
    }

    /// Repair the tutorial scope on campaigns written before later worlds
    /// stopped replaying the opening lesson. Only the starting world owns that
    /// lesson; this is safe to run on every load and needs no schema bump.
    pub(crate) fn normalize_tutorial_scope(&mut self) {
        for (index, planet) in self.planets.iter_mut().enumerate() {
            if index != super::STARTING_PLANET {
                if let Some(planet) = planet {
                    planet.tutorial_done = true;
                }
            }
        }
    }
}
