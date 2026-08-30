use super::*;

#[test]
fn every_simulation_event_has_a_distinct_cue_and_gain() {
    let events = [
        AudioEvent::Placement,
        AudioEvent::Demolition,
        AudioEvent::Harvest,
        AudioEvent::Delivery,
        AudioEvent::Research,
        AudioEvent::Directive,
        AudioEvent::Collapse,
        AudioEvent::Achievement,
        AudioEvent::UiConfirm,
        AudioEvent::UiBack,
    ];

    for event in events {
        assert!(event_gain(event) > 0.0);
        assert!(cue_for(event).cooldown() > 0.0);
    }
    let cues: std::collections::HashSet<_> = events.into_iter().map(cue_for).collect();
    assert_eq!(cues.len(), events.len());
}

#[test]
fn music_gain_keeps_menu_quiet_and_collapse_present() {
    assert!(music_gain(MusicState::Menu, 0.0) < music_gain(MusicState::Collapse, 0.0));
    assert!(music_gain(MusicState::Gameplay, 1.0) > music_gain(MusicState::Gameplay, 0.0));
    assert!(music_gain(MusicState::Collapse, 0.0) > 0.0);
    assert!(music_gain(MusicState::Gameplay, 10.0) <= 0.5);
}

#[test]
fn every_cue_renders_nonempty_audio() {
    for cue in AudioCue::SFX {
        let bytes = render_wav(&voices_for(cue), &config(), cue_seed(cue));
        assert!(bytes.len() > 44, "{cue:?} did not render audio");
    }
    for (state, cue) in AudioCue::MUSIC {
        let bytes = render_wav(&music_voices(state), &config(), cue_seed(cue));
        assert!(bytes.len() > 44, "{state:?} did not render audio");
    }
}
