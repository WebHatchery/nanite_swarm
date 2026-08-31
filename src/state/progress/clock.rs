//! Pause, speed, and next-event controls for a planetary simulation.

use super::{NextEvent, PlanetState};

impl PlanetState {
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

    pub fn next_interesting_event(&self) -> Option<NextEvent> {
        [
            (self.power_collapse_shutdown, "Power recovery"),
            (self.power_collapse_cooldown, "Collapse recovery"),
            (self.research_lock_timer, "Research unlock"),
            (self.export_cooldown, "Mass Driver ready"),
        ]
        .into_iter()
        .filter(|(seconds, _)| *seconds > 0.01 && seconds.is_finite())
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(seconds, label)| NextEvent { label, seconds })
    }

    pub fn fast_forward_to_next_event(&mut self) -> bool {
        let Some(event) = self.next_interesting_event() else {
            return false;
        };
        let mut remaining = event.seconds;
        while remaining > 0.0001 {
            let delta = remaining.min(crate::state::TICK_SECONDS);
            self.step(delta, false);
            remaining -= delta;
        }
        self.sim_accumulator = 0.0;
        true
    }
}
