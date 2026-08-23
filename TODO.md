# Outstanding work — Nanite Swarm

Last verified: 2026-08-23.

The implementation items from the 2026-07-14 GDD audit are complete. There
are no known gameplay, save-system, UI, audio, release, or source-size gate
blockers. The remaining work is architectural follow-up identified by the
2026-08-20 architecture audit.

## Architecture follow-up

- [ ] Extract a focused session/persistence service from `Game` if further
  features increase its ownership of campaign, research, settings, audio,
  capture, or persistence state.
- [ ] Split the near-limit files before the next major feature adds to them:
  `src/capture_scenes.rs` (691 lines),
  `src/screens/planetary_view/flow_render.rs` (698 lines),
  `src/screens/planetary_view/terrain_render.rs` (692 lines),
  `src/data/defs.rs` (708 lines),
  `src/state/logistics/tests.rs` (729 lines), and
  `src/state/simulation/tests.rs` (682 lines).
- [ ] Replace the process-global `GameData` `OnceLock` with an injected data
  context if the game needs multiple simultaneous or independently tested
  sessions.
