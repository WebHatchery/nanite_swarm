# TODO — Nanite Swarm

Open implementation work from the 2026-07-14 code audit against `gdd.md`.
This list is limited to work an implementation agent can complete in the
repository: Rust code, game data, UI, generated in-game assets, tests, and
build/release automation. Product decisions, external account operations,
legal work, marketing, manual playtesting, and subjective art direction are
intentionally out of scope.

## Core loop and win condition

- [ ] Show the research requirement for every sealed Seed Ship stage in the ship-screen tech tree.
- [ ] Render the launch departure over the current base, including the ship clearing the world before transit.
- [ ] Record the source of each collapse, including the building or network failure that caused it.
- [ ] Show the collapse source in the warning banner and post-collapse summary.
- [ ] Apply source-specific collapse feedback when a local building failure differs from a broad grid collapse.
- [ ] Add distinct Core stage 3 and 4 sprite assets and wire them to the saved stage data.

## Logistics puzzle

- [ ] Replace the fixed partial-load tile threshold with a threshold derived from route length, drill rate, and pad fill time.
- [ ] Add a real per-tile drone queue for congested conduit junctions.
- [ ] Render queued drones and their order clearly on the map.
- [ ] Add short-lived route reservations or equivalent look-ahead so a dispatch wave distributes across available paths.
- [ ] Add a network revision counter and invalidate cached drone paths only when the network changes.
- [ ] Profile path validation before enabling the revision-based optimization on every route.
- [ ] Warn when a producer is placed on open ground without a connected conduit route.
- [ ] Update deposit visuals as bonus ore depletes, including the transition from rich ground to ordinary ground.
- [ ] Add a clear bridge silhouette and edge treatment so a bridge is readable over void or water.
- [ ] Show an in-game placement hint when a selected building can bridge a gap.

## Interplanetary meta-layer

- [ ] Add a configurable cap to each destination world's pending pod queue.
- [ ] Define and display the overflow state when a destination queue reaches its cap.
- [ ] Warn about a full landing pad in the mass-driver order panel, not only a missing pad.
- [ ] Allow a standing order to target a specific landing pad on the destination world.
- [ ] Add a solar-map panel for editing the standing orders of any world in the campaign.
- [ ] Show the selected remote world's current order, cargo, destination, and delivery status.
- [ ] Add schedule/cooldown data to the mass-driver order model.
- [ ] Add priority data and deterministic tie-breaking to the mass-driver order model.
- [ ] Add a surplus-only shipping mode that leaves a configured reserve on the source world.
- [ ] Rebuild each world's unlocked-building set from campaign research so stale unlocks are removed.
- [ ] Define named planet-feature data in `assets/planets.json`.
- [ ] Generate named features deterministically from the planet definition and map seed.
- [ ] Display a feature's name and bounds in the map or inspector.

## Planet hazards

- [ ] Add an uncovered-building overlay that identifies network and upkeep buildings outside hazard-counter coverage.
- [ ] Add spatial hazard fields or regions to the planet definition schema.
- [ ] Apply spatial acid and cold modifiers during simulation.
- [ ] Render the active hazard field with a legend on the map.
- [ ] Split acid wear and dust wear into separate simulation quantities.
- [ ] Give acid and dust distinct map tints and inspector readouts.
- [ ] Add positive per-planet building and power constraints to `assets/planets.json`.
- [ ] Validate per-planet constraints at data load and report invalid combinations with source context.
- [ ] Enforce positive planet constraints during building availability and placement.
- [ ] Explain a planet-specific constraint in the build and research UI.
- [ ] Add Server Bank heat generation and storage to the simulation.
- [ ] Implement water-tile cooling for buildings that support it.
- [ ] Expose heat, cooling, and overheating effects in the Server Bank inspector.

## Research

- [ ] Add a planet-condition field to research nodes for per-planet branches.
- [ ] Filter unavailable research nodes by the active world's condition.
- [ ] Explain the planet requirement when a branch is unavailable.
- [ ] Track the research sources contributing to each resolved stat.
- [ ] Show contributing techs from the stat sheet or stat inspector.
- [ ] Move the dust stall threshold, efficiency steps, speed penalty, and leak threshold into research/config data.
- [ ] Resolve those dust-response values through the modifier system.
- [ ] Move collapse thresholds into the modifier system.
- [ ] Add deterministic tests for modified dust response and collapse thresholds.

## Content and progression

- [ ] Extend `RecipeDef` to support two or more carried inputs.
- [ ] Add the required hopper and delivery state for multiple carried inputs.
- [ ] Update routing and recipe completion logic for multiple carried inputs.
- [ ] Add tests for partial delivery, missing inputs, and recipe completion with multiple carried inputs.
- [ ] Scale producer collection capacity with crew size instead of giving every producer one fixed collection slot.
- [ ] Verify that `swarm_dispatch` changes collection throughput without duplicating unrelated producer effects.
- [ ] Add progress tracking for completed and expired directives to the Records screen.
- [ ] Add scrolling to the Records achievement grid when its content exceeds the viewport.
- [ ] Add descriptions for every buildable building in the building data.
- [ ] Add tutorial copy for the remaining building and map-interaction steps.

## Simulation architecture

- [ ] Replace per-second offline stepping with an aggregated deterministic earnings calculation.
- [ ] Define the offline report fields for ore, alloy, Data, power, and elapsed world time.
- [ ] Show the aggregated earnings report when a campaign resumes.
- [ ] Add deterministic snapshot coverage for power failure and recovery.
- [ ] Add deterministic snapshot coverage for research unlocks and modifiers.
- [ ] Add deterministic snapshot coverage for collapse scaling and recovery.
- [ ] Move pad depth and hopper depth from Rust constants into `logistics` in `game_config.json`.
- [ ] Validate building and recipe references at data load.
- [ ] Validate planet and terrain references at data load.
- [ ] Validate directive and achievement references at data load.
- [ ] Validate Seed Ship and Core stage references at data load.
- [ ] Include the source file and identifier in missing-data errors instead of panicking without context.
- [ ] Extract the generic seeded value-noise field into `macroquad-toolkit` and keep terrain vocabulary in this project.
- [ ] Add deterministic stress fixtures for congested conduit networks and hundreds of drones.
- [ ] Extend routing stress fixtures across interplanetary travel and background-world transitions.
- [ ] Measure background simulation cost across all worlds during travel and menu transitions.
- [ ] Measure routing and rendering cost on the largest supported map.
- [ ] Add automated performance thresholds for the measured simulation and routing fixtures.
- [ ] Add placement-rule tests for terrain, research, hazards, bridges, and banned buildings.
- [ ] Wire the `mainmenu`, `research`, and `logistics` headless capture scenes into CI.

## Save system

- [ ] Investigate close/quit hooks for each supported target and document which targets can autosave on window close.
- [ ] Add the supported target-specific close-save implementation where a reliable hook exists.
- [ ] Increase recovery depth beyond one rotated backup.
- [ ] Show which backup generation was used during recovery.
- [ ] Create a per-version save migration registry and migration runner.
- [ ] Move the current unversioned-save conversion into the migration runner.
- [ ] Add player-visible campaign slots to the save model.
- [ ] Add slot selection, creation, deletion, and switching to the menu UI.
- [ ] Add tests proving that one slot cannot overwrite another slot's campaign.
- [ ] Add an offline-clock tamper guard with a bounded fallback delta.
- [ ] Add hard caps for offline elapsed time and offline resource gains.
- [ ] Include elapsed time, capped time, and each resource gain in the offline report.

## UX and UI

- [ ] Persist a preferred simulation speed separately from the per-planet pause state.
- [ ] Add a touch-visible speed control above 4× with an explicit maximum.
- [ ] Define the next-interesting-event data needed for fast-forwarding.
- [ ] Add a touch-visible control that advances to the next supported interesting event.
- [ ] Implement BOX SELECT with a visible drag gesture.
- [ ] Implement BUILD MENU with a visible tap target.
- [ ] Wire the main-menu Quit action on supported desktop targets and remove it where the target cannot quit.
- [ ] Highlight the map tiles or endpoints required by tutorial steps that involve drawing a route.
- [ ] Store toast history per campaign rather than per world.
- [ ] Persist toast history and restore it into the Records log.
- [ ] Make the right-hand panel stack calculate height-aware layouts before adding another panel.
- [ ] Remove or disable audio sliders until an audio backend is present so no settings control is inert.
- [ ] Add key-remapping controls and persist the mappings.
- [ ] Add plain-language explanations for each settings option.
- [ ] Implement building relocation as a touch-completable action.
- [ ] Implement saving a selected set of buildings as a blueprint stamp.
- [ ] Implement placing a blueprint stamp with per-building validation and clear failures.
- [ ] Add undo history for placement, relocation, and demolition.
- [ ] Add power production and consumption series to the bottom-bar graph.
- [ ] Add alloy production and consumption series to the bottom-bar graph.
- [ ] Add Data production and consumption series to the bottom-bar graph.
- [ ] Add production-versus-consumption series to the graph.
- [ ] Persist graph samples in the campaign save and restore them on load.

## Presentation and in-game assets

- [ ] Animate rotating drills without changing simulation timing.
- [ ] Animate blinking servers without changing simulation timing.
- [ ] Animate turbine spin without changing simulation timing.
- [ ] Add collapse shake behind the reduced-motion setting.
- [ ] Add harvest-impact effects behind the reduced-motion setting.
- [ ] Add planet-specific atmosphere overlays driven by planet data.
- [ ] Add an in-game version string to the main menu and bug-report payload.

## Audio integration

- [ ] Evaluate `macroquad::audio` and `kira` against the project's WASM autoplay and latency constraints.
- [ ] Add a data-driven audio event interface for gameplay and UI events.
- [ ] Connect placement, demolition, harvest, and delivery events to the audio interface.
- [ ] Connect research, directive, collapse, achievement, and UI events to the audio interface.
- [ ] Add menu and gameplay music states driven by swarm scale and collapse state.
- [ ] Add an ambient gameplay layer and connect the existing audio settings to its volume controls.

## Release engineering

- [ ] Stamp the build version into the game and generated release metadata.
- [ ] Remove the Ko-fi widget and bug-report branding from paid-distribution builds.
- [ ] Externalize player-facing strings into a table while preserving the current default language.
- [ ] Add shape or pattern cues for all state indicators currently encoded only by color.
- [ ] Verify text scaling at the supported minimum and maximum sizes.
- [ ] Add a reduced-motion setting and honor it for animation, shake, and screen effects.
- [ ] Add automated save-compatibility tests for every supported save version.
