# TODO — Nanite Swarm

Open implementation work from the 2026-07-14 code audit against `gdd.md`.
This list is limited to work an implementation agent can complete in the
repository: Rust code, game data, UI, generated in-game assets, tests, and
build/release automation. Product decisions, external account operations,
legal work, marketing, manual playtesting, and subjective art direction are
intentionally out of scope.

## Core loop and win condition

- [x] Show the research requirement for every sealed Seed Ship stage in the ship-screen tech tree.
- [x] Render the launch departure over the current base, including the ship clearing the world before transit.
- [x] Record the source of each collapse, including the building or network failure that caused it.
- [x] Show the collapse source in the warning banner and post-collapse summary.
- [x] Apply source-specific collapse feedback when a local building failure differs from a broad grid collapse.
- [x] Add distinct Core stage 3 and 4 sprite assets and wire them to the saved stage data.

## Logistics puzzle

- [x] Replace the fixed partial-load tile threshold with a threshold derived from route length, drill rate, and pad fill time.
- [x] Add a real per-tile drone queue for congested conduit junctions.
- [x] Render queued drones and their order clearly on the map.
- [x] Add short-lived route reservations or equivalent look-ahead so a dispatch wave distributes across available paths.
- [x] Add a network revision counter and invalidate cached drone paths only when the network changes.
- [x] Profile path validation before enabling the revision-based optimization on every route.
- [x] Warn when a producer is placed on open ground without a connected conduit route.
- [x] Update deposit visuals as bonus ore depletes, including the transition from rich ground to ordinary ground.
- [x] Add a clear bridge silhouette and edge treatment so a bridge is readable over void or water.
- [x] Show an in-game placement hint when a selected building can bridge a gap.

## Interplanetary meta-layer

- [x] Add a configurable cap to each destination world's pending pod queue.
- [x] Define and display the overflow state when a destination queue reaches its cap.
- [x] Warn about a full landing pad in the mass-driver order panel, not only a missing pad.
- [x] Allow a standing order to target a specific landing pad on the destination world.
- [x] Add a solar-map panel for editing the standing orders of any world in the campaign.
- [x] Show the selected remote world's current order, cargo, destination, and delivery status.
- [x] Add schedule/cooldown data to the mass-driver order model.
- [x] Add priority data and deterministic tie-breaking to the mass-driver order model.
- [x] Add a surplus-only shipping mode that leaves a configured reserve on the source world.
- [x] Rebuild each world's unlocked-building set from campaign research so stale unlocks are removed.
- [x] Define named planet-feature data in `assets/planets.json`.
- [x] Generate named features deterministically from the planet definition and map seed.
- [x] Display a feature's name and bounds in the map or inspector.

## Planet hazards

- [x] Add an uncovered-building overlay that identifies network and upkeep buildings outside hazard-counter coverage.
- [x] Add spatial hazard fields or regions to the planet definition schema.
- [x] Apply spatial acid and cold modifiers during simulation.
- [x] Render the active hazard field with a legend on the map.
- [x] Split acid wear and dust wear into separate simulation quantities.
- [x] Give acid and dust distinct map tints and inspector readouts.
- [x] Add positive per-planet building and power constraints to `assets/planets.json`.
- [x] Validate per-planet constraints at data load and report invalid combinations with source context.
- [x] Enforce positive planet constraints during building availability and placement.
- [x] Explain a planet-specific constraint in the build and research UI.
- [x] Add Server Bank heat generation and storage to the simulation.
- [x] Implement water-tile cooling for buildings that support it.
- [x] Expose heat, cooling, and overheating effects in the Server Bank inspector.

## Research

- [x] Add a planet-condition field to research nodes for per-planet branches.
- [x] Filter unavailable research nodes by the active world's condition.
- [x] Explain the planet requirement when a branch is unavailable.
- [x] Track the research sources contributing to each resolved stat.
- [x] Show contributing techs from the stat sheet or stat inspector.
- [x] Move the dust stall threshold, efficiency steps, speed penalty, and leak threshold into research/config data.
- [x] Resolve those dust-response values through the modifier system.
- [x] Move collapse thresholds into the modifier system.
- [x] Add deterministic tests for modified dust response and collapse thresholds.

## Content and progression

- [x] Extend `RecipeDef` to support two or more carried inputs.
- [x] Add the required hopper and delivery state for multiple carried inputs.
- [x] Update routing and recipe completion logic for multiple carried inputs.
- [x] Add tests for partial delivery, missing inputs, and recipe completion with multiple carried inputs.
- [x] Scale producer collection capacity with crew size instead of giving every producer one fixed collection slot.
- [x] Verify that `swarm_dispatch` changes collection throughput without duplicating unrelated producer effects.
- [x] Add progress tracking for completed and expired directives to the Records screen.
- [x] Add scrolling to the Records achievement grid when its content exceeds the viewport.
- [x] Add descriptions for every buildable building in the building data.
- [x] Add tutorial copy for the remaining building and map-interaction steps.

## Simulation architecture

- [x] Replace per-second offline stepping with an aggregated deterministic earnings calculation.
- [x] Define the offline report fields for ore, alloy, Data, power, and elapsed world time.
- [x] Show the aggregated earnings report when a campaign resumes.
- [x] Add deterministic snapshot coverage for power failure and recovery.
- [x] Add deterministic snapshot coverage for research unlocks and modifiers.
- [x] Add deterministic snapshot coverage for collapse scaling and recovery.
- [x] Move pad depth and hopper depth from Rust constants into `logistics` in `game_config.json`.
- [x] Validate building and recipe references at data load.
- [x] Validate planet and terrain references at data load.
- [x] Validate directive and achievement references at data load.
- [x] Validate Seed Ship and Core stage references at data load.
- [x] Include the source file and identifier in missing-data errors instead of panicking without context.
- [x] Extract the generic seeded value-noise field into `macroquad-toolkit` and keep terrain vocabulary in this project.
- [x] Add deterministic stress fixtures for congested conduit networks and hundreds of drones.
- [x] Extend routing stress fixtures across interplanetary travel and background-world transitions.
- [x] Measure background simulation cost across all worlds during travel and menu transitions.
- [x] Measure routing and rendering cost on the largest supported map.
- [x] Add automated performance thresholds for the measured simulation and routing fixtures.
- [x] Add placement-rule tests for terrain, research, hazards, bridges, and banned buildings.
- [x] Wire the `mainmenu`, `research`, and `logistics` headless capture scenes into CI.

## Save system

- [x] Investigate close/quit hooks for each supported target and document which targets can autosave on window close.
- [x] Add the supported target-specific close-save implementation where a reliable hook exists.
- [x] Increase recovery depth beyond one rotated backup.
- [x] Show which backup generation was used during recovery.
- [x] Create a per-version save migration registry and migration runner.
- [x] Move the current unversioned-save conversion into the migration runner.
- [x] Add player-visible campaign slots to the save model.
- [x] Add slot selection, creation, deletion, and switching to the menu UI.
- [x] Add tests proving that one slot cannot overwrite another slot's campaign.
- [x] Add an offline-clock tamper guard with a bounded fallback delta.
- [x] Add hard caps for offline elapsed time and offline resource gains.
- [x] Include elapsed time, capped time, and each resource gain in the offline report.

## UX and UI

- [x] Persist a preferred simulation speed separately from the per-planet pause state.
- [x] Add a touch-visible speed control above 4× with an explicit maximum.
- [x] Define the next-interesting-event data needed for fast-forwarding.
- [x] Add a touch-visible control that advances to the next supported interesting event.
- [x] Implement BOX SELECT with a visible drag gesture.
- [x] Implement BUILD MENU with a visible tap target.
- [x] Wire the main-menu Quit action on supported desktop targets and remove it where the target cannot quit.
- [x] Highlight the map tiles or endpoints required by tutorial steps that involve drawing a route.
- [x] Store toast history per campaign rather than per world.
- [x] Persist toast history and restore it into the Records log.
- [x] Make the right-hand panel stack calculate height-aware layouts before adding another panel.
- [x] Remove or disable audio sliders until an audio backend is present so no settings control is inert.
- [x] Add key-remapping controls and persist the mappings.
- [x] Add plain-language explanations for each settings option.
- [x] Implement building relocation as a touch-completable action.
- [x] Implement saving a selected set of buildings as a blueprint stamp.
- [x] Implement placing a blueprint stamp with per-building validation and clear failures.
- [x] Add undo history for placement, relocation, and demolition.
- [x] Add power production and consumption series to the bottom-bar graph.
- [x] Add alloy production and consumption series to the bottom-bar graph.
- [x] Add Data production and consumption series to the bottom-bar graph.
- [x] Add production-versus-consumption series to the graph.
- [x] Persist graph samples in the campaign save and restore them on load.

## Presentation and in-game assets

- [x] Animate rotating drills without changing simulation timing.
- [x] Animate blinking servers without changing simulation timing.
- [x] Animate turbine spin without changing simulation timing.
- [x] Add collapse shake behind the reduced-motion setting.
- [x] Add harvest-impact effects behind the reduced-motion setting.
- [x] Add planet-specific atmosphere overlays driven by planet data.
- [x] Add an in-game version string to the main menu and bug-report payload.

## Audio integration

- [x] Evaluate `macroquad::audio` and `kira` against the project's WASM autoplay and latency constraints.
- [x] Add a data-driven audio event interface for gameplay and UI events.
- [x] Connect placement, demolition, harvest, and delivery events to the audio interface.
- [x] Connect research, directive, collapse, achievement, and UI events to the audio interface.
- [x] Add menu and gameplay music states driven by swarm scale and collapse state.
- [x] Add an ambient gameplay layer and connect the existing audio settings to its volume controls.

## Release engineering

- [x] Stamp the build version into the game and generated release metadata.
- [x] Remove the Ko-fi widget and bug-report branding from paid-distribution builds.
- [x] Externalize player-facing strings into a table while preserving the current default language.
- [x] Add shape or pattern cues for all state indicators currently encoded only by color.
- [x] Verify text scaling at the supported minimum and maximum sizes.
- [x] Add a reduced-motion setting and honor it for animation, shake, and screen effects.
- [x] Add automated save-compatibility tests for every supported save version.
