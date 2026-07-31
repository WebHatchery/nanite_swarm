# TODO — Nanite Swarm

Open work from the 2026-07-14 code audit against `gdd.md`. The single-planet
automation loop is solid; the interplanetary pitch, the win condition, and the
logistics puzzle at the heart of the GDD are the gaps.

## Core loop and win condition

- Build the Seed Ship: multi-stage megastructure, large resource sinks, launch sequence, per-planet victory (absent from code).
- Add campaign-complete and terminal-failure states; `GamePhase` has only MainMenu/Playing/Research/Interplanetary/Settings.
- Decide whether infrastructure collapse can actually end a run, and design the difficulty curve around it — today it is a 20-second timeout with a data penalty.
- Make the five core stages mechanical, not just visual: tie Crash-Lander/Fortress/Space Elevator/Planetary Ring (GDD §5) to research and throughput milestones with new capabilities each.

## The logistics puzzle

- **Decide the pillar**: either route drones along the conduit network with throughput limits and real crossings (the GDD's "Spaghetti"), or formally redesign around free-flying drones with congestion. Conduits are power-only today; drones BFS over raw terrain and ignore conduits, buildings, and each other. Content and balance downstream both depend on this.
- Re-path drones when the grid changes, emit the existing unused `DroneEvent::PathBlocked`, and surface a broken-route error flag as the GDD promises.
- Make congestion and throughput a scaling pressure; today it is one drone per drill with no interaction.
- Add ore deposits with richness and depletion — drills currently output a hardcoded 10 minerals/cycle from any tile, so placement is spatially meaningless.
- Make Bridge tiles real; they are a bool flag that does not even transmit power.

## Interplanetary meta-layer

- Persist planet state across travel — `main.rs` discards the current planet and generates a fresh random one, inverting the GDD's "Planet 1 does not disappear".
- Simulate colonized planets in the background (scheduled tick or summary) so left-behind worlds keep producing.
- Turn Mass Drivers into gameplay: a building, export schedules, transit time, and receiving landing pads, instead of a tech flag plus 100 minerals.
- Persist `colonized_planets`, `current_planet_index`, and the active directive; they live on `Game` and are lost on reload.
- Give each planet its own generation rules; all five call the same `PlanetState::new(24, 24, seed)` with difficulty as flavour text.

## Planet hazards

- Zone 2 (Venus): acid rain degrading standard conduits, Ceramic Plating and Shield Generator counters, void-heavy volcanic terrain.
- Zone 3 (Cryo): freeze slowing drones 50%, Heater Nodes along the network, no solar or wind.
- Add per-planet power constraints (no solar on Venus, infinite geothermal, fusion-only cryo worlds) — this is what makes each planet a distinct puzzle.
- Add the Heat mechanic the GDD gives Server Banks; water tiles carry a "may provide cooling" comment with no logic behind it.

## Research

- Fix five no-op techs — `efficient_drills`, `drone_capacity`, `power_efficiency`, `advanced_research`, `neural_expansion` are never read outside the tree UI.
- Build a declared modifier/stat system driven from `research.json`, replacing the stringly-typed `unlocked_techs.contains(...)` pattern that caused the no-ops.
- Expand the tree well past 15 nodes, with per-planet branches and hazard counters as research.

## Content

- Add production chains: intermediate products, recipes, and processing buildings. Minerals currently teleport into one global pool, so there is nothing to optimise.
- Grow the building set beyond 10 across processing, logistics, hazard counters, and megastructure parts.
- Tier the resource set beyond Minerals/Energy/Data/Biomass to support chains and mass-driver strategy.
- Larger and more varied maps with per-planet generators and landmark features (needs the camera work below).
- Replace the four `tier % 4` directives and four hardcoded achievements with a real objective/milestone system and a full achievement set.
- Write the GDD's "indifferent optimizer" tone into directives, research descriptions, and planet-arrival vignettes; it appears nowhere in game text today.

## Simulation architecture

- Move to a fixed timestep with an accumulator; `get_frame_time()` feeds the sim directly, so behaviour is frame-rate dependent and diverges from the 60s offline chunks.
- Pull simulation out of `screens/` so planetary, interplanetary, and background-planet simulation share one engine.
- Tick-quantise timers and float accumulation, then add deterministic snapshot tests for terrain harvest, power failure, drone routing, research unlocks, and collapse thresholds.
- Move hardcoded balance constants into validated JSON fixtures — drill output is `10.0/cycle` while the HUD shows the unused config value `2.0`, and `conduit_throughput` and `core_power_consumption` are dead config fields.
- Validate data at load with real error messages; `game_data().building(id)` panics on a missing id.

## Save system

- Add autosave on interval, on quit, and on travel; today saving is manual or on entering the menu.
- Add a save schema version and migration path before the first balance patch mangles old saves.
- Add multiple slots and corruption recovery with backup rotation.
- Persist meta-state: colonized planets, current planet, directive, settings, and tutorial progress.
- Harden offline progress with a clock-tamper guard and hard caps, and turn the offline banner into an earnings report.

## UX and UI

- Add camera pan and zoom; none exists at a fixed 28px grid, and it blocks any map larger than 24x24.
- Add pause and game speed — the HUD advertises "PAUSE Space" and shows speed buttons whose return values are ignored.
- Wire or remove the decorative controls: PAN, ZOOM, BOX SELECT, BUILD, DEMOLISH hints are unwired and the main-menu Quit button is a no-op.
- Replace the six-condition text checklist with a real persisted tutorial with highlighting and interactive gating.
- Add a notification/toast system; achievements currently unlock with no feedback beyond a counter.
- Make settings work and persist — `ui_scale` is stored but never applied, audio sliders drive nothing, and the Settings struct is never saved.
- Add genre-standard build tools: demolish mode, drag-demolish, relocation, blueprint stamps, undo.
- Add production statistics — rates, consumption, and net-flow graphs; the bottom-bar graph is decorative.

## Audio

- Build audio from zero: there is no audio system, no sound files, and no `macroquad::audio` import. Needs menu and gameplay music layers responding to swarm scale and collapse, an ambient bed, and SFX for placement, demolition, harvest, drone delivery, research, directives, the collapse alarm, achievements, and UI. Evaluate `macroquad::audio` against `kira` early given WASM autoplay and latency constraints.

## Art and presentation

- Animate the procedural sprite set: rotating drills, blinking servers, turbine spin — everything is static today.
- Do a cohesive style and palette pass, add screen juice (collapse shake, harvest impact), and give each planet its own atmosphere.
- Build core stages 3 and 4 visuals: the Space Elevator tether and the background Planetary Ring, the GDD's signature image.
- Produce marketing art: key art, logo, store capsules, trailer. Only `catalog_thumbnail.png` exists.

## Engineering quality

- Expand test coverage past the single offline-hibernation test: power flood-fill and repeater range, drone pathfinding and re-path, placement rules, harvest consequences, research effects, save round-trip and migration, offline sim, collapse thresholds. Wire the headless capture harness scenes into CI.
- Stress-test drone routing and conduit networks on congested maps and across interplanetary transitions.
- Add opt-in telemetry and crash reporting for balance funnels and errors beyond the manual bug-report widget.
- Set performance budgets and profile for multi-planet background sim, larger maps, and hundreds of drones, especially on WASM.

## Release

Direction decided 2026-07-14: premium PC game at $5, itch.io first, Steam only if
itch shows traction. Mobile is dropped, superseding the GDD's "PC & Mobile" framing.

- itch.io release engineering: page setup, a `butler` push pipeline wired into `publish.ps1`, and in-game version stamping so bug reports are traceable.
- Decide the free-web-build question — a free full WASM build on the WebHatchery portal undercuts a $5 price. Take it down, freeze it as a first-planet demo, or host the demo on the itch page.
- Remove the Ko-fi widget and bug-report branding from the paid distribution.
- Produce itch page assets: 630x500 cover, screenshots and GIFs, description copy, optionally a trailer.
- Retune the 4-hour battery and hibernation model for desktop session patterns; it was designed for mobile check-in retention.
- Externalise strings into a table before content expansion doubles their count; full localization can slip past the itch launch.
- Accessibility: colourblind-safe shapes for state now encoded only in colour, working text scaling, a reduced-motion option, remappable keys.
- Run a closed alpha and balance iteration loop; the game has never been balanced beyond the author's own play.
- Define the QA matrix (Windows versions x GPUs, plus browsers if a web demo ships) and save-compat testing per patch.
- Set up legal and payout basics: privacy policy once telemetry exists, EULA, itch tax setup.
- Run an itch devlog cadence with GIF-forward social posts, and define up front the traction threshold that triggers the Steam decision.
- Steam (contingent): Steamworks achievements, cloud saves, rich presence, store page and wishlist campaign, a Next Fest demo, and controller/Deck support.
