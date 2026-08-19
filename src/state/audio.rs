//! Data-driven audio events.
//!
//! The shipped build deliberately has no sound files yet. Keeping event names
//! in the simulation means a future backend can bind macroquad::audio or kira
//! without teaching gameplay about file paths, autoplay, or channel volume.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioEvent {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicState {
    Menu,
    Gameplay,
    Collapse,
}

#[derive(Debug, Clone, Copy)]
pub struct AudioMix {
    pub music_state: MusicState,
    pub swarm_scale: f32,
    pub sfx_volume: f32,
    pub music_volume: f32,
}

impl Default for AudioMix {
    fn default() -> Self {
        Self {
            music_state: MusicState::Menu,
            swarm_scale: 0.0,
            sfx_volume: 1.0,
            music_volume: 0.8,
        }
    }
}

impl AudioMix {
    pub fn for_planet(
        state: &super::game_state::PlanetState,
        sfx_volume: f32,
        music_volume: f32,
    ) -> Self {
        Self {
            music_state: if state.power_collapse_shutdown > 0.0 {
                MusicState::Collapse
            } else {
                MusicState::Gameplay
            },
            swarm_scale: (state.grid.total_buildings() as f32 / 40.0).clamp(0.0, 1.0),
            sfx_volume: sfx_volume.clamp(0.0, 1.0),
            music_volume: music_volume.clamp(0.0, 1.0),
        }
    }
}

impl super::game_state::PlanetState {
    pub fn emit_audio(&mut self, event: AudioEvent) {
        self.audio_events.push(event);
    }

    pub fn take_audio_events(&mut self) -> Vec<AudioEvent> {
        std::mem::take(&mut self.audio_events)
    }
}
