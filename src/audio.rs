//! Browser-safe audio for the swarm.
//!
//! The simulation only emits [`AudioEvent`] values. This module gives those
//! events a small procedural sound bank, so the game has audio without adding
//! binary assets or file paths to gameplay code. Playback is armed by the
//! first user gesture because browsers reject autoplay from `Game::new`.

use crate::state::{AudioEvent, AudioMix, MusicState};
use macroquad::audio::PlaySoundParams;
use macroquad_toolkit::audio::SoundManager;
use macroquad_toolkit::synth::{render_wav, SynthConfig, Voice, Wave};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum AudioCue {
    Placement,
    Demolition,
    Harvest,
    Delivery,
    Research,
    Directive,
    Collapse,
    Achievement,
    UiConfirm,
    UiBack,
    MenuMusic,
    GameplayMusic,
    CollapseMusic,
}

impl AudioCue {
    const SFX: [Self; 10] = [
        Self::Placement,
        Self::Demolition,
        Self::Harvest,
        Self::Delivery,
        Self::Research,
        Self::Directive,
        Self::Collapse,
        Self::Achievement,
        Self::UiConfirm,
        Self::UiBack,
    ];

    const MUSIC: [(MusicState, Self); 3] = [
        (MusicState::Menu, Self::MenuMusic),
        (MusicState::Gameplay, Self::GameplayMusic),
        (MusicState::Collapse, Self::CollapseMusic),
    ];

    fn cooldown(self) -> f32 {
        match self {
            Self::Placement | Self::Demolition | Self::Harvest => 0.08,
            Self::Delivery => 0.18,
            Self::Research | Self::Directive => 0.24,
            Self::Collapse => 1.5,
            Self::Achievement => 2.0,
            Self::UiConfirm | Self::UiBack => 0.10,
            Self::MenuMusic | Self::GameplayMusic | Self::CollapseMusic => 0.0,
        }
    }
}

fn config() -> SynthConfig {
    SynthConfig {
        sample_rate: 22_050,
        master_gain: 0.25,
    }
}

fn cue_for(event: AudioEvent) -> AudioCue {
    match event {
        AudioEvent::Placement => AudioCue::Placement,
        AudioEvent::Demolition => AudioCue::Demolition,
        AudioEvent::Harvest => AudioCue::Harvest,
        AudioEvent::Delivery => AudioCue::Delivery,
        AudioEvent::Research => AudioCue::Research,
        AudioEvent::Directive => AudioCue::Directive,
        AudioEvent::Collapse => AudioCue::Collapse,
        AudioEvent::Achievement => AudioCue::Achievement,
        AudioEvent::UiConfirm => AudioCue::UiConfirm,
        AudioEvent::UiBack => AudioCue::UiBack,
    }
}

fn event_gain(event: AudioEvent) -> f32 {
    match event {
        AudioEvent::Placement => 0.42,
        AudioEvent::Demolition => 0.48,
        AudioEvent::Harvest => 0.36,
        AudioEvent::Delivery => 0.52,
        AudioEvent::Research => 0.60,
        AudioEvent::Directive => 0.58,
        AudioEvent::Collapse => 0.78,
        AudioEvent::Achievement => 0.82,
        AudioEvent::UiConfirm | AudioEvent::UiBack => 0.34,
    }
}

fn voices_for(cue: AudioCue) -> Vec<Voice> {
    match cue {
        AudioCue::Placement => vec![
            Voice::tone(0.0, 0.11, 330.0, 0.18)
                .glide(520.0)
                .wave(Wave::Triangle),
            Voice::tone(0.0, 0.045, 1800.0, 0.07)
                .wave(Wave::Noise)
                .attack(0.01),
        ],
        AudioCue::Demolition => vec![
            Voice::tone(0.0, 0.16, 280.0, 0.18)
                .glide(130.0)
                .wave(Wave::Square),
            Voice::tone(0.0, 0.13, 95.0, 0.12).wave(Wave::Sine),
        ],
        AudioCue::Harvest => vec![
            Voice::tone(0.0, 0.10, 440.0, 0.16)
                .glide(660.0)
                .wave(Wave::Triangle),
            Voice::tone(0.06, 0.12, 660.0, 0.14).wave(Wave::Sine),
        ],
        AudioCue::Delivery => vec![
            Voice::tone(0.0, 0.08, 392.0, 0.16).wave(Wave::Triangle),
            Voice::tone(0.07, 0.14, 587.0, 0.17).wave(Wave::Triangle),
            Voice::tone(0.15, 0.18, 784.0, 0.13).wave(Wave::Sine),
        ],
        AudioCue::Research => vec![
            Voice::tone(0.0, 0.16, 523.0, 0.14).wave(Wave::Triangle),
            Voice::tone(0.08, 0.20, 659.0, 0.14).wave(Wave::Triangle),
            Voice::tone(0.16, 0.27, 784.0, 0.13).wave(Wave::Sine),
        ],
        AudioCue::Directive => vec![
            Voice::tone(0.0, 0.14, 740.0, 0.16)
                .glide(1100.0)
                .wave(Wave::Sine),
            Voice::tone(0.10, 0.20, 1480.0, 0.08).wave(Wave::Triangle),
        ],
        AudioCue::Collapse => vec![
            Voice::tone(0.0, 0.42, 180.0, 0.20)
                .glide(70.0)
                .wave(Wave::Square),
            Voice::tone(0.0, 0.28, 75.0, 0.14).wave(Wave::Sine),
            Voice::tone(0.03, 0.10, 900.0, 0.07)
                .wave(Wave::Noise)
                .attack(0.01),
        ],
        AudioCue::Achievement => vec![
            Voice::tone(0.0, 0.13, 523.0, 0.15).wave(Wave::Triangle),
            Voice::tone(0.10, 0.15, 659.0, 0.15).wave(Wave::Triangle),
            Voice::tone(0.20, 0.18, 784.0, 0.16).wave(Wave::Triangle),
            Voice::tone(0.31, 0.34, 1046.0, 0.14).wave(Wave::Sine),
        ],
        AudioCue::UiConfirm => vec![Voice::tone(0.0, 0.08, 660.0, 0.15)
            .glide(880.0)
            .wave(Wave::Triangle)],
        AudioCue::UiBack => vec![Voice::tone(0.0, 0.10, 520.0, 0.14)
            .glide(330.0)
            .wave(Wave::Triangle)],
        AudioCue::MenuMusic | AudioCue::GameplayMusic | AudioCue::CollapseMusic => Vec::new(),
    }
}

fn music_voices(state: MusicState) -> Vec<Voice> {
    match state {
        MusicState::Menu => vec![
            Voice::tone(0.0, 7.8, 98.0, 0.07)
                .wave(Wave::Sine)
                .attack(0.18),
            Voice::tone(0.0, 7.8, 147.0, 0.035)
                .wave(Wave::Triangle)
                .attack(0.18),
            Voice::tone(1.6, 1.8, 392.0, 0.045)
                .wave(Wave::Triangle)
                .attack(0.12),
            Voice::tone(4.6, 2.4, 494.0, 0.04)
                .wave(Wave::Triangle)
                .attack(0.12),
        ],
        MusicState::Gameplay => vec![
            Voice::tone(0.0, 7.8, 73.4, 0.08)
                .wave(Wave::Sine)
                .attack(0.16),
            Voice::tone(0.0, 7.8, 110.0, 0.035)
                .wave(Wave::Triangle)
                .attack(0.16),
            Voice::tone(0.4, 0.45, 220.0, 0.035).wave(Wave::Triangle),
            Voice::tone(1.4, 0.45, 277.2, 0.035).wave(Wave::Triangle),
            Voice::tone(2.4, 0.45, 329.6, 0.035).wave(Wave::Triangle),
            Voice::tone(4.4, 0.45, 220.0, 0.035).wave(Wave::Triangle),
            Voice::tone(5.4, 0.45, 277.2, 0.035).wave(Wave::Triangle),
            Voice::tone(6.4, 0.45, 370.0, 0.035).wave(Wave::Triangle),
        ],
        MusicState::Collapse => vec![
            Voice::tone(0.0, 7.8, 55.0, 0.10)
                .wave(Wave::Sine)
                .attack(0.10),
            Voice::tone(0.0, 7.8, 82.4, 0.045)
                .wave(Wave::Square)
                .attack(0.10),
            Voice::tone(0.4, 0.6, 164.8, 0.04)
                .glide(130.8)
                .wave(Wave::Triangle),
            Voice::tone(2.5, 0.6, 164.8, 0.04)
                .glide(130.8)
                .wave(Wave::Triangle),
            Voice::tone(4.6, 0.6, 146.8, 0.04)
                .glide(110.0)
                .wave(Wave::Triangle),
        ],
    }
}

fn music_cue(state: MusicState) -> AudioCue {
    AudioCue::MUSIC
        .iter()
        .find_map(|(candidate, cue)| (*candidate == state).then_some(*cue))
        .expect("every music state has a cue")
}

fn music_gain(state: MusicState, swarm_scale: f32) -> f32 {
    match state {
        MusicState::Menu => 0.24,
        MusicState::Gameplay => 0.16 + swarm_scale.clamp(0.0, 1.0) * 0.34,
        MusicState::Collapse => 0.52,
    }
}

fn cue_seed(cue: AudioCue) -> u64 {
    match cue {
        AudioCue::Placement => 0xA001,
        AudioCue::Demolition => 0xA002,
        AudioCue::Harvest => 0xA003,
        AudioCue::Delivery => 0xA004,
        AudioCue::Research => 0xA005,
        AudioCue::Directive => 0xA006,
        AudioCue::Collapse => 0xA007,
        AudioCue::Achievement => 0xA008,
        AudioCue::UiConfirm => 0xA009,
        AudioCue::UiBack => 0xA00A,
        AudioCue::MenuMusic => 0xB001,
        AudioCue::GameplayMusic => 0xB002,
        AudioCue::CollapseMusic => 0xB003,
    }
}

/// The app-owned audio bank and its playback policy.
pub struct GameAudio {
    sounds: SoundManager<AudioCue>,
    cooldowns: HashMap<AudioCue, f32>,
    pending_events: Vec<AudioEvent>,
    active_music: Option<MusicState>,
    armed: bool,
}

impl GameAudio {
    /// Render and decode the bank during startup, but do not play anything.
    pub async fn load() -> Self {
        let mut sounds = SoundManager::new();
        for cue in AudioCue::SFX {
            let bytes = render_wav(&voices_for(cue), &config(), cue_seed(cue));
            let _ = sounds.load_sound_bytes(cue, &bytes).await;
        }
        for (state, cue) in AudioCue::MUSIC {
            let bytes = render_wav(&music_voices(state), &config(), cue_seed(cue));
            let _ = sounds.load_sound_bytes(cue, &bytes).await;
        }
        Self {
            sounds,
            cooldowns: HashMap::new(),
            pending_events: Vec::new(),
            active_music: None,
            armed: false,
        }
    }

    /// Apply the current mix and play events emitted by the previous frame.
    ///
    /// Events stay queued until a gesture has armed the browser audio context;
    /// this avoids losing the first meaningful cue after a menu click.
    pub fn update(&mut self, mix: AudioMix, events: Vec<AudioEvent>, user_gesture: bool, dt: f32) {
        self.sounds.sfx_volume = mix.sfx_volume;
        self.sounds.music_volume = mix.music_volume;
        for event in events {
            if self.pending_events.len() >= 48 {
                self.pending_events.remove(0);
            }
            self.pending_events.push(event);
        }
        for remaining in self.cooldowns.values_mut() {
            *remaining = (*remaining - dt).max(0.0);
        }
        if user_gesture {
            self.armed = true;
            self.sounds.visible = true;
        }
        if !self.armed {
            return;
        }

        self.sync_music(mix);
        for event in std::mem::take(&mut self.pending_events) {
            self.play_event(event, mix.swarm_scale);
        }
    }

    fn sync_music(&mut self, mix: AudioMix) {
        if self.active_music != Some(mix.music_state) {
            if let Some(previous) = self.active_music {
                self.sounds.stop_raw(music_cue(previous));
            }
            let cue = music_cue(mix.music_state);
            self.sounds.play_raw(
                cue,
                PlaySoundParams {
                    looped: true,
                    volume: (mix.music_volume * music_gain(mix.music_state, mix.swarm_scale))
                        .clamp(0.0, 1.0),
                },
            );
            self.active_music = Some(mix.music_state);
        } else {
            let cue = music_cue(mix.music_state);
            self.sounds.set_raw_volume(
                cue,
                mix.music_volume * music_gain(mix.music_state, mix.swarm_scale),
            );
        }
    }

    fn play_event(&mut self, event: AudioEvent, swarm_scale: f32) {
        let cue = cue_for(event);
        if self.cooldowns.get(&cue).copied().unwrap_or(0.0) > 0.0 {
            return;
        }
        let swarm_gain = 0.86 + swarm_scale.clamp(0.0, 1.0) * 0.14;
        self.sounds.play_sfx(cue, event_gain(event) * swarm_gain);
        self.cooldowns.insert(cue, cue.cooldown());
    }
}

#[cfg(test)]
mod tests;
