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
