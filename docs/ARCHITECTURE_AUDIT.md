# Architecture audit

Audit date: 2026-08-20

## Changes made

- `src/main.rs` was a 661-line coordinator containing construction, every
  screen transition, simulation timing, persistence, and capture-sensitive
  behavior. Construction and the platform loop remain in `main.rs`; frame
  orchestration is now in `src/main/update.rs`, with one helper per screen
  phase.
- `src/state/simulation.rs` mixed fixed-step timing, processor recipes, server
  production, heat, dust, biomass, and collapse consequences. The fixed-step
  coordinator remains in that file, while recipes, upkeep, and collapse own
  their mechanics in `state/simulation/`.
- Removed `src/data/loader.rs`, a project-local generic JSON wrapper that
  duplicated `macroquad_toolkit::data_loader`. Game data now calls the shared
  loader directly, leaving fallback and parsing behavior in one place.
- Moved the performance fixture tests from an inline implementation module to
  `src/state/performance/tests.rs`, matching the repository's unit-test
  placement rule.
- Removed an unused recipe helper, fixed the redundant planetary hover branch,
  made `GridPos::to_index` value-based, and made strict Clippy clean.

## Remaining debt

- `Game` is still the application-level coordinator and owns campaign,
  research, settings, audio, capture, and persistence state. It is now isolated
  from the binary entrypoint, but a future feature pass could extract a small
  session/persistence service if those responsibilities grow again.
- `data/defs.rs` still performs semantic validation against engine, directive,
  and state vocabularies. This is safe and well-covered today, but it keeps the
  data layer coupled to gameplay enums. Splitting validation into domain-owned
  validators would be the next architectural step; it is intentionally not
  part of this behavior-preserving pass.
- Several rendering and fixture files remain near the 600-line restructure
  signal (`capture_scenes.rs`, `flow_render.rs`, `terrain_render.rs`, and the
  larger test suites), but all remain below the 800-line hard limit. They should
  be split by scene family or rendering model before the next major feature.
- `GameData` is initialized through a process-global `OnceLock`, which matches
  the current Macroquad startup model but makes isolated multi-game instances
  difficult to test. A dependency-injected data context would be cleaner if
  the project ever needs multiple simultaneous sessions.
