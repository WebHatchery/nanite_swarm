//! Coarse background worlds and the campaign-wide directive clock.

use crate::directives::{pick_directive, rotation_seconds};

use super::super::audio::AudioEvent;
use super::Campaign;

/// Step length for worlds the player is not looking at. Coarse on purpose: a
/// left-behind world only has to keep producing, not animate.
const BACKGROUND_TICK_SECONDS: f32 = 1.0;
/// Ceiling on background steps per call, so a long stall cannot turn into a
/// stutter across four planets at once.
const MAX_BACKGROUND_TICKS: u32 = 4;

impl Campaign {
    /// Continue the active standing directive across a save gap.
    ///
    /// Directive time is campaign-owned, so it cannot be handled by the
    /// foreground world's offline production pass. Consume the gap in bounded
    /// directive-sized segments, settling each completion or lapse before the
    /// next one starts. This keeps rewards and the durable history aligned
    /// with the live update path even when several rotations fit in the gap.
    pub fn apply_offline_directive_progress(&mut self, simulated_seconds: f32) -> u32 {
        if simulated_seconds <= 0.0 || !simulated_seconds.is_finite() {
            return 0;
        }

        let rotation = rotation_seconds(self.data.data());
        let mut remaining = simulated_seconds;
        let mut completed = 0;
        while remaining > 0.0001 {
            if self.directive.completed
                || self.directive.duration <= 0.0
                || self.directive_timer >= rotation
            {
                let was_completed = self.directive.completed;
                let before_tier = self.directive_tier;
                self.update_directive(0.0);
                if was_completed && self.directive_tier > before_tier {
                    completed += 1;
                }
                continue;
            }

            let until_rotation = (rotation - self.directive_timer).max(0.0);
            let step = remaining.min(self.directive.duration).min(until_rotation);
            if step <= 0.0 {
                continue;
            }
            self.update_directive(step);
            remaining -= step;
        }

        if self.directive.completed || self.directive.duration <= 0.0 {
            let was_completed = self.directive.completed;
            let before_tier = self.directive_tier;
            self.update_directive(0.0);
            if was_completed && self.directive_tier > before_tier {
                completed += 1;
            }
        }
        completed
    }

    /// Run every world the player is *not* standing on.
    ///
    /// The GDD's meta-layer promise is that Planet 1 does not disappear, and a
    /// world that is preserved but frozen only half keeps it: come back and
    /// nothing has happened. These run the same simulation the foreground
    /// world does, in coarse one-second steps and without visuals, which is
    /// cheap enough for four planets and avoids a second economy that could
    /// disagree with the real one.
    pub fn update_background(&mut self, delta_time: f32) {
        if delta_time <= 0.0 || !delta_time.is_finite() {
            return;
        }
        self.background_accumulator += delta_time;

        let mut ticks = 0;
        while self.background_accumulator >= BACKGROUND_TICK_SECONDS && ticks < MAX_BACKGROUND_TICKS
        {
            self.background_accumulator -= BACKGROUND_TICK_SECONDS;
            ticks += 1;
        }
        if ticks == MAX_BACKGROUND_TICKS {
            self.background_accumulator = 0.0;
        }
        if ticks == 0 {
            return;
        }

        let current = self.current;
        for (index, slot) in self.planets.iter_mut().enumerate() {
            if index == current {
                continue;
            }
            let Some(planet) = slot.as_mut() else {
                continue;
            };
            for _ in 0..ticks {
                planet.step(BACKGROUND_TICK_SECONDS, false);
            }
        }
    }

    /// Tick the active directive against the current planet, rotating to the
    /// next one when it is done or its window has run out.
    pub fn update_directive(&mut self, delta_time: f32) {
        self.since_save += delta_time;
        self.directive_timer += delta_time;
        let expired = self.directive_timer >= crate::directives::rotation_seconds(self.data.data())
            || self.directive.duration <= 0.0
            || self.directive.completed;

        if expired {
            let reward = if self.directive.completed {
                self.directive.reward_data
            } else {
                0.0
            };
            // A directive used to pay out and rotate without a word, so the
            // only evidence was the panel quietly saying something else.
            let announcement = if self.directive.completed {
                Some(format!(
                    "Directive met: {} (+{:.0} Data)",
                    self.directive.description, reward
                ))
            } else if self.directive.duration > 0.0 {
                Some(format!("Directive lapsed: {}", self.directive.description))
            } else {
                None
            };
            let completed = self.directive.completed;
            self.directive_history.push(super::DirectiveRecord {
                description: self.directive.description.clone(),
                completed,
                reward_data: reward,
                tier: self.directive_tier,
            });
            if self.directive_history.len() > 100 {
                self.directive_history.remove(0);
            }
            if let Some(message) = announcement {
                let planet = self.current_mut();
                planet.emit_audio(AudioEvent::Directive);
                if completed {
                    planet.notifications.success(message);
                } else {
                    planet.notifications.warning(message);
                }
            }
            if reward > 0.0 {
                self.current_mut().resources.data += reward;
            }
            self.directive_timer = 0.0;
            self.directive_tier += 1;
            self.directive = pick_directive(self.data.data(), self.directive_tier);
        }

        let Some(planet) = self.planets[self.current].as_ref() else {
            return;
        };
        self.directive.update(planet, delta_time);
    }
}
