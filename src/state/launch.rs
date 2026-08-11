//! The launch: the one moment the loop the whole game is built on turns over.
//!
//! The campaign has already moved by the time this runs — the ship is spent,
//! the new world is colonized, and the swarm's attention is on it. What lives
//! here is purely the telling of it, four timed beats over state that is
//! already settled, so quitting mid-sequence cannot leave a half-launched
//! campaign behind.

use crate::data::LaunchSequenceDef;

/// Where the sequence has got to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchBeat {
    /// On the pad, counting down.
    Countdown,
    /// Leaving the world it was built on.
    Ascent,
    /// Between worlds, with nothing aboard awake.
    Transit,
    /// Over the new world, which has something to say for itself.
    Arrival,
}

/// A launch being played out. Cosmetic: it owns no campaign state and cannot
/// change any.
#[derive(Debug, Clone)]
pub struct LaunchSequence {
    /// Campaign slot of the world that was spent. Kept because the campaign
    /// is no longer pointing at it.
    origin: usize,
    /// Campaign slot of the world arrived at.
    target: usize,
    elapsed: f32,
}

impl LaunchSequence {
    pub fn new(origin: usize, target: usize) -> Self {
        Self {
            origin,
            target,
            elapsed: 0.0,
        }
    }

    fn def() -> &'static LaunchSequenceDef {
        &crate::data::game_data().seed_ship.launch
    }

    /// Beat boundaries as running totals, in the order they play.
    fn beats() -> [(LaunchBeat, f32); 4] {
        let def = Self::def();
        let countdown = def.countdown_seconds.max(0.0);
        let ascent = countdown + def.ascent_seconds.max(0.0);
        let transit = ascent + def.transit_seconds.max(0.0);
        let arrival = transit + def.arrival_seconds.max(0.0);
        [
            (LaunchBeat::Countdown, countdown),
            (LaunchBeat::Ascent, ascent),
            (LaunchBeat::Transit, transit),
            (LaunchBeat::Arrival, arrival),
        ]
    }

    /// How long the whole sequence runs.
    pub fn total_seconds() -> f32 {
        Self::beats()[3].1
    }

    /// When a beat starts, measured from ignition being ordered. Lets a caller
    /// drop the sequence straight onto a beat without knowing the durations.
    pub fn beat_start(beat: LaunchBeat) -> f32 {
        Self::beats()
            .iter()
            .take_while(|(candidate, _)| *candidate != beat)
            .last()
            .map(|(_, end)| *end)
            .unwrap_or(0.0)
    }

    pub fn origin(&self) -> usize {
        self.origin
    }

    /// The name of the world being spent, for the lines that name it.
    pub fn origin_name(&self) -> &'static str {
        crate::data::game_data().planet(self.origin).name.as_str()
    }

    pub fn target(&self) -> usize {
        self.target
    }

    /// Run the sequence in real time. It is deliberately not on the world
    /// clock: the world is not being simulated while this plays.
    pub fn advance(&mut self, delta_time: f32) {
        self.elapsed = (self.elapsed + delta_time.max(0.0)).min(Self::total_seconds());
    }

    /// Cut to the end. The player has seen it before.
    pub fn skip(&mut self) {
        self.elapsed = Self::total_seconds();
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= Self::total_seconds()
    }

    /// The beat playing now, or `None` once the sequence is over.
    pub fn beat(&self) -> Option<LaunchBeat> {
        if self.is_finished() {
            return None;
        }
        Self::beats()
            .iter()
            .find(|(_, end)| self.elapsed < *end)
            .map(|(beat, _)| *beat)
    }

    /// How far through the current beat, 0..1. Zero once finished.
    pub fn beat_fraction(&self) -> f32 {
        let Some(current) = self.beat() else {
            return 0.0;
        };
        let beats = Self::beats();
        let end = beats
            .iter()
            .find(|(beat, _)| *beat == current)
            .map(|(_, end)| *end)
            .unwrap_or(0.0);
        let start = beats
            .iter()
            .take_while(|(beat, _)| *beat != current)
            .last()
            .map(|(_, end)| *end)
            .unwrap_or(0.0);
        let span = end - start;
        if span <= 0.0 {
            return 1.0;
        }
        ((self.elapsed - start) / span).clamp(0.0, 1.0)
    }

    /// Seconds left on the countdown, for the number on the pad.
    pub fn countdown_remaining(&self) -> f32 {
        (Self::beats()[0].1 - self.elapsed).max(0.0)
    }

    /// What the swarm says over the current beat. The arrival beat speaks with
    /// the arriving world's own line, so it comes from the caller.
    pub fn line(&self, arrival_line: &str) -> String {
        let def = Self::def();
        let raw = match self.beat() {
            Some(LaunchBeat::Countdown) => def.countdown_line.as_str(),
            Some(LaunchBeat::Ascent) => def.ascent_line.as_str(),
            Some(LaunchBeat::Transit) => def.transit_line.as_str(),
            Some(LaunchBeat::Arrival) | None => arrival_line,
        };
        raw.replace("{origin}", self.origin_name())
    }

    pub fn skip_hint() -> &'static str {
        Self::def().skip_hint.as_str()
    }
}

#[cfg(test)]
mod tests;
