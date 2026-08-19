# Audio integration

Nanite Swarm currently ships without audio files. Gameplay emits the
data-driven `AudioEvent` values in `src/state/audio.rs`; a future backend only
needs to consume those values and choose a sound, rather than adding file
paths to simulation code.

The intended first backend is `macroquad::audio`: it is already part of the
runtime and is the smallest WASM-compatible option. Kira remains a possible
native enhancement, but it would add a dependency, a second platform path,
and a larger autoplay surface for browser builds. The interface therefore
keeps one-shot events, menu/gameplay/collapse music state, swarm scale, and
effective SFX/music volumes independent of either backend.

Browser audio must begin after a user gesture. The menu and gameplay buttons
are the safe activation points; an implementation should defer playback until
that gesture rather than starting audio during `Game::new`.
