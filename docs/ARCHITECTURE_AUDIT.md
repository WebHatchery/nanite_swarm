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
- Moved duplicate-ID, cross-file reference, and definition-shape checks into
  `src/data/validation.rs`; the application layer now only sequences the data,
  engine, directive, and state validators.
- Split the factory and campaign-ending screenshot fixtures into child modules;
  the scene dispatcher fell from 792 to 620 lines before the next capture
  family can make the source-size limit a problem.
- Split the desktop command-bar graph renderer into `bottom_bar/graph.rs`,
  reducing the command-bar coordinator from 773 to 555 lines and adding direct
  coverage for its mode-specific scaling and readouts.
- Split the cross-world research contract from the campaign test coordinator
  into `src/state/campaign/tests/research.rs`, keeping the parent suite below
  the source-size restructure signal as research and background simulation grow.
- Split relocation and undo behavior from `src/state/actions.rs` into
  `src/state/actions/relocation.rs`, leaving the action coordinator at 567
  lines before the next construction feature.
- Split PlanetState map interaction mutation from `src/state/progress.rs` into
  `progress/interaction.rs`, and moved world construction constraints,
  hazard-field math, and resource ceilings into `progress/constraints.rs`.
- Split PlanetState records, graph telemetry modes, blueprints, undo entries,
  and research progress from `src/state/game_state.rs` into
  `game_state/records.rs` and `game_state/research.rs`, leaving the state
  aggregate focused on world fields and lifecycle construction.
- Split PlanetState world generation and data reattachment from
  `state/game_state.rs` into `game_state/construction.rs`, leaving the state
  aggregate focused on its serialized fields and runtime stat refresh.
- Split the loaded balance contract validation from `src/data/game_config.rs`
  into `game_config/validation.rs`, leaving the configuration module focused on
  schema types and defaults while preserving `GameConfig::validate()`.
- Split save rotation, corruption recovery, and slot-isolation coverage from
  `state/persistence/tests.rs` into `persistence/tests/recovery.rs`, leaving
  the parent suite focused on migration and round-trip contracts.
- Split arrival guidance and map-interaction coverage from
  `state/progress/tests.rs` into `progress/tests/guidance.rs` and
  `progress/tests/interaction.rs`, leaving rate, capacity, and timer coverage
  together in the parent suite.
- Split factory, freight, upkeep, smelting, and congestion capture staging from
  `capture_scenes/scenes.rs` into `scenes/industrial.rs`, leaving the dispatcher
  focused on reset state and the remaining scene families.
- Split individual building-card rendering from
  `screens/planetary_view/hud/build_palette.rs` into `build_palette/rows.rs`,
  leaving palette layout and quick-action state changes in the coordinator.
- Split procedural terrain and building tile generation from
  `graphics_gen/tiles.rs` into `tiles/terrain.rs` and `tiles/buildings.rs`,
  leaving the generator coordinator focused on output helpers and dispatch.
- Split planetary Mass Driver and Landing Pad behavior plus campaign freight
  orders from `state/shipping.rs` into `shipping/planet.rs` and
  `shipping/campaign.rs`, leaving cargo records and shared helpers in the
  coordinator.
- Split the loaded data records from `data/defs.rs` into building/terrain,
  research, Seed Ship, planet, and campaign-contract modules, leaving
  `GameData` focused on loading, indexing, and engine-tree conversion.
- Split the touch-sized top-bar renderer from `top_bar.rs` into
  `top_bar/mobile.rs`, leaving desktop metrics and notice layout independent
  from future mobile navigation changes.
- Split desktop and mobile offline, arrival, and collapse banners from
  `top_bar.rs` into `top_bar/notices.rs`, leaving the coordinator focused on
  resource cards, navigation, and the persistent desktop status row.
- Split Solar System planet rows, dossiers, and the legend from
  `interplanetary_view.rs` into `interplanetary_view/planet_panel.rs`, leaving
  the map coordinator focused on orbit layout, routes, and transitions.
- Split freight-lane drawing, pod interpolation, route hit testing, and
  shipment labels from `shipping_panel.rs` into
  `interplanetary_view/shipping_panel/routes.rs`, leaving the panel focused on
  standing-order controls and blocker copy.
- Split shared route readiness, landing-pad targeting, and reserve status copy
  from `shipping_panel.rs` into `shipping_panel/route_status.rs`, preserving
  the screen-level helpers used by planet rows and the mobile dossier.
- Split animated factory depth markers and focus-target overlays from
  `factory_deck.rs` into `factory_deck/art.rs`, leaving the control-room
  coordinator focused on layout, progress, and touch actions.
- Split animated building effects from `terrain_render/tiles.rs` into
  `terrain_render/tiles/motion.rs`, leaving visible tile traversal and
  placement-preview rendering in the tile coordinator.
- Split the no-structure terrain inspector branch from `inspector_panel.rs`
  into `inspector_panel/terrain.rs`, keeping terrain rewards, feature copy,
  and harvest/filter actions together behind a typed inspection context.
- Split alert priority, recovery-button policy, flow detection, and pending
  interaction checks from `bottom_bar.rs` into `bottom_bar/alerts.rs`, keeping
  desktop and mobile alert behavior on one shared policy surface.
- Split campaign-wide research synchronization, offline research spending,
  and legacy research adoption from `campaign.rs` into `campaign/research.rs`,
  leaving travel, autosave, and directive orchestration in the campaign shell.
- Split shared mobile status and compact resource-readout helpers from
  `top_bar.rs` into `top_bar/readouts.rs`, leaving desktop card drawing and
  navigation controls in the top-bar coordinator.
- Split the Seed Ship's drill-to-Smelter production-chain tests from
  `state/seed_ship/tests.rs` into `seed_ship/tests/production.rs`, leaving
  stage payment, research gates, launch, and achievement contracts together.
- Split standing-route, reserve, shipment-label, and freight hit-test coverage
  from `interplanetary_view/tests.rs` into `tests/shipping.rs`, leaving map,
  dossier, and remote-alert contracts in the parent suite.
- Split severed-route, rerouting, repair, collapse recovery, and route-length
  coverage from `state/logistics/tests/network.rs` into `network/recovery.rs`,
  leaving throughput and congestion contracts in the parent suite.
- Split battery timing, event fast-forward, research threshold, and
  achievement/toast lifecycle coverage from `state/progress/tests.rs` into
  `progress/tests/timing.rs`, leaving economy and rate contracts in the parent.
- Split planet-state invariants and shipment invariants from
  `state/campaign/validation.rs` into `validation/planet.rs` and
  `validation/shipment.rs`, leaving load orchestration and directive checks in
  the parent validator.
- Split the terrain harvest and forest-filter action tests into
  `state/actions/tests/terrain.rs`, keeping the parent action suite below the
  hard source-size limit as terrain tradeoffs continue to grow.
- Split relocation, cargo-preservation, and undo coverage from
  `state/actions/tests.rs` into `actions/tests/relocation.rs`, keeping action
  mutation cases separate from the movement contract they verify.
- Split terrain harvest rewards, harvesting, and forest-filter conversion from
  `state/actions.rs` into `actions/harvest.rs`, leaving construction and
  processor control actions in the coordinator.
- Split standing-order cycling and reserve-policy coverage into
  `state/shipping/tests/orders.rs`, keeping the freight test coordinator below
  the hard source-size limit as remote-route behavior continues to grow.
- Split landing, overflow, pad recovery, and cross-world arrival coverage into
  `state/shipping/tests/arrivals.rs`, leaving route setup and inspector checks
  in the parent freight suite.
- Split network routing, drone recovery, and congestion accounting into
  `state/logistics/routes.rs`, leaving the logistics coordinator focused on
  producer dispatch before the next network feature expands it.
- Split Solar System telemetry types and world snapshots into
  `state/campaign/telemetry.rs`, leaving campaign progression and travel
  orchestration below the source-size restructure signal.
- Split Campaign construction, deterministic planet generation, and runtime
  data reattachment into `state/campaign/construction.rs`, leaving campaign
  progression and travel methods in the main coordinator.
- Split Solar System telemetry and remote-world alert coverage into
  `state/campaign/tests/telemetry.rs`, keeping the campaign test coordinator
  below the source-size restructure signal as map snapshots grow.
- Split planetary hazard, shielding, and climate coverage into
  `state/campaign/tests/worlds.rs`, leaving campaign travel, autosave, and
  background-time contracts in the parent.
- Split Solar System snapshot assembly and freight-map action handling into
  `src/main/update/interplanetary.rs`, leaving the frame coordinator focused on
  phase dispatch, local screens, and shared simulation timing.
- Split the touch-sized Solar System world dossier from `interplanetary_view/mobile.rs`
  into `interplanetary_view/mobile/world_info.rs`, leaving portrait navigation,
  tabs, and list layout independent from the growing dossier variants.
- Split standing-order controls, route warnings, and remote-order rows from
  `interplanetary_view/shipping_panel.rs` into `shipping_panel/order_editor.rs`,
  leaving the parent focused on scroll/layout orchestration and the in-flight
  manifest.
- Split Neural Network connections, node visuals, progress rings, and pointer
  selection from `research_view.rs` into `research_view/nodes.rs`, leaving the
  research coordinator focused on layout, info panels, and screen actions.
- Split deterministic terrain, ore, and landing-site generation from
  `engine/grid_engine/grid.rs` into `grid/generation.rs`, leaving the grid
  core focused on tile access, building occupancy, and pathfinding.
- Split box selection, blueprint stamping, and selected-processor policy from
  `state/actions.rs` into `state/actions/batch.rs`, leaving the parent focused
  on single-building placement, routing, and demolition.
- Split orbit positions, planet marks, hover targets, and telemetry markers
  from `interplanetary_view.rs` into `interplanetary_view/solar_map.rs`,
  leaving the screen coordinator focused on panels, routes, and dossiers.
- Split Core stage and milestone readouts from `hud/inspector_panel.rs` into
  `inspector_panel/core_status.rs`, leaving the structure inspector focused on
  building and terrain presentation.
- Split desktop research-node details from `research_view.rs` into
  `research_view/desktop.rs`, leaving the coordinator focused on shared layout,
  viewport interaction, and screen actions.
- Split JSON round-trip persistence coverage from `state/persistence/tests.rs`
  into `tests/roundtrip.rs`, leaving the parent focused on migrations, offline
  progress, and malformed-save behavior.
- Split placement, conduit-route, and demolition coverage from
  `state/actions/tests.rs` into `actions/tests/construction.rs`, leaving the
  parent focused on processor action checks and shared fixture setup.
- Split factory focus status, Core progression, and resource-chip readouts from
  `factory_deck.rs` into `factory_deck/readouts.rs`, leaving the deck renderer
  focused on control-room geometry and interaction.
- Split shared tick advancement, event fast-forwarding, campaign systems, and
  completion/autosave checks from `main/update.rs` into `main/update/simulation.rs`,
  leaving the frame coordinator focused on screen-phase dispatch.
- Split the shared capture-session reset from `capture_scenes/scenes.rs` into
  `capture_scenes/scenes/reset.rs`, leaving the scene coordinator focused on
  dispatching each named screenshot fixture.
- Split general mobile, Core, Seed Ship, recovery, and camera capture staging
  from `capture_scenes/scenes.rs` into `capture_scenes/scenes/world.rs`, leaving
  the coordinator focused on routing the remaining scene families.
- Split flow route and processor-node drawing from `flow_render/model.rs` into
  `flow_render/model/drawing.rs`, leaving the flow model focused on live
  logistics calculations and ledger inputs.
- Split simulation runtime coverage into focused `runtime/clock.rs`,
  `runtime/collapse.rs`, `runtime/tutorial.rs`, and `runtime/upkeep.rs` suites,
  leaving `simulation/tests/runtime.rs` as the shared fixture boundary.
- Split progression-state coverage into `progress/tests/offline.rs`,
  `progress/tests/effects.rs`, and `progress/tests/capacity.rs`, leaving the
  parent suite focused on shared fixtures and smaller interaction families.
- Split traffic-pressure and shared-trunk coverage from
  `logistics/tests/network.rs` into `network/congestion.rs`, leaving the
  parent focused on route integrity, dispatch, and hazard behavior.
- Split building texture, topology, glow, motion, and power-state drawing from
  `terrain_render/tiles.rs` into `tiles/building.rs`, leaving the tile loop
  focused on terrain, feature marks, and hover/placement overlays.
- Split drill buffering, producer discovery, partial-load policy, and drone
  dispatch from `state/logistics.rs` into `state/logistics/producers.rs`,
  leaving route selection and network recovery in the parent.
- Split pause/speed/next-event controls and offline catch-up from
  `state/progress.rs` into `progress/clock.rs` and `progress/offline.rs`,
  leaving live rates, hazards, and presentation readouts in the parent.
- Split short-phone, regular-phone, and full-height world dossier rendering
  from `interplanetary_view/mobile/world_info.rs` into `world_info/compact.rs`,
  `world_info/medium.rs`, and `world_info/tall.rs`, leaving the parent as the
  prepared context and viewport dispatcher.
- Split the desktop help overlay from `hud/bottom_bar.rs` into
  `bottom_bar/help.rs`, leaving the command bar focused on controls, clock
  actions, and the live status panels.
- Split the desktop command-bar renderer from `hud/bottom_bar.rs` into
  `bottom_bar/desktop.rs`, leaving the parent focused on action vocabulary and
  responsive path dispatch while the desktop module owns its layout.

## Remaining debt

- `Game` is still the application-level coordinator and owns campaign,
  research, settings, audio, capture, and persistence state. It is now isolated
  from the binary entrypoint, but a future feature pass could extract a small
  session/persistence service if those responsibilities grow again.
- The domain validators still depend on their gameplay vocabularies, by design:
  engine, directive, and state rules are checked by the modules that execute
  them. Cross-file definition integrity now stays in the data domain.
- Several rendering and test files remain near the 600-line restructure signal
  (`flow_render.rs`, `terrain_render.rs`, and the larger test suites), but all
  remain below the 800-line hard limit. They should be split by rendering model
  or responsibility before the next major feature.
