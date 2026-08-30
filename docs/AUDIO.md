# Audio integration

Nanite Swarm ships without audio files. Gameplay emits the data-driven
`AudioEvent` values in `src/state/audio.rs`; `src/audio.rs` consumes those
values and renders the sound bank procedurally at startup, rather than adding
file paths to simulation code.

The backend is `macroquad::audio` through the shared toolkit's
`SoundManager`, with deterministic WAVs from `macroquad_toolkit::synth`. Kira
remains a possible native enhancement, but it would add a dependency, a second
platform path, and a larger autoplay surface for browser builds. The interface
keeps one-shot events, menu/gameplay/collapse music state, swarm scale, and
effective SFX/music volumes independent of the backend.

Browser audio begins after a user gesture. The menu and gameplay buttons are
the safe activation points; `GameAudio` defers playback until that gesture
rather than starting audio during `Game::new`.

The Audio tab in Settings exposes master, SFX, and music volume steppers plus
a `Test signal` button. The preview uses the same `UiConfirm` event as other
interface confirmations, so it verifies the active volume without coupling
the settings view directly to the audio backend.
