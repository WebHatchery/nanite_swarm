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
mod tests {
    use super::*;

    #[test]
    fn the_beats_play_in_order_and_then_it_is_over() {
        let mut sequence = LaunchSequence::new(0, 1);
        let mut seen = Vec::new();
        for _ in 0..2_000 {
            if let Some(beat) = sequence.beat() {
                if seen.last() != Some(&beat) {
                    seen.push(beat);
                }
            }
            sequence.advance(0.05);
        }
        assert_eq!(
            seen,
            vec![
                LaunchBeat::Countdown,
                LaunchBeat::Ascent,
                LaunchBeat::Transit,
                LaunchBeat::Arrival,
            ]
        );
        assert!(sequence.is_finished());
        assert_eq!(sequence.beat(), None);
    }

    #[test]
    fn a_launch_nobody_wants_to_watch_can_be_cut_short() {
        let mut sequence = LaunchSequence::new(0, 1);
        sequence.advance(0.5);
        assert!(!sequence.is_finished());
        sequence.skip();
        assert!(sequence.is_finished());
        assert_eq!(sequence.beat(), None);
    }

    #[test]
    fn each_beat_runs_from_nothing_to_all_of_it() {
        let mut sequence = LaunchSequence::new(0, 1);
        // The very first frame of the first beat.
        assert!(sequence.beat_fraction() < 0.05);
        // One frame short of the countdown ending is nearly all of that beat.
        sequence.advance(LaunchSequence::beats()[0].1 - 0.01);
        assert_eq!(sequence.beat(), Some(LaunchBeat::Countdown));
        assert!(sequence.beat_fraction() > 0.95);
        // And the next beat starts over from the bottom.
        sequence.advance(0.02);
        assert_eq!(sequence.beat(), Some(LaunchBeat::Ascent));
        assert!(sequence.beat_fraction() < 0.05);
    }

    #[test]
    fn the_world_being_spent_is_named_in_the_line_that_spends_it() {
        let mut sequence = LaunchSequence::new(0, 1);
        sequence.advance(LaunchSequence::beats()[0].1 + 0.1);
        let line = sequence.line("unused");
        assert_eq!(sequence.beat(), Some(LaunchBeat::Ascent));
        assert!(
            line.contains(sequence.origin_name()),
            "ascent line said: {}",
            line
        );
        assert!(!line.contains("{origin}"), "placeholder left in: {}", line);
    }

    #[test]
    fn the_arriving_world_speaks_for_itself() {
        let mut sequence = LaunchSequence::new(0, 1);
        sequence.advance(LaunchSequence::beats()[2].1 + 0.1);
        assert_eq!(sequence.beat(), Some(LaunchBeat::Arrival));
        assert_eq!(
            sequence.line("Venus does not want this."),
            "Venus does not want this."
        );
    }

    #[test]
    fn the_countdown_counts_down() {
        let mut sequence = LaunchSequence::new(0, 1);
        let start = sequence.countdown_remaining();
        assert!(start > 0.0, "the pad hold has no length");
        sequence.advance(1.0);
        assert!((start - sequence.countdown_remaining() - 1.0).abs() < 0.001);
        sequence.skip();
        assert_eq!(sequence.countdown_remaining(), 0.0);
    }
}
