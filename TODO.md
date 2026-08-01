# TODO — Nanite Swarm

Open work from the 2026-07-14 code audit against `gdd.md`. The single-planet
automation loop is solid; the interplanetary pitch, the win condition, and the
logistics puzzle at the heart of the GDD are the gaps.

## Core loop and win condition

The Seed Ship exists: four stages declared in `assets/seed_ship.json`, its own screen behind the SHIP button, and a commit toggle that pours production into the yard at a capped intake, so it can only be paid for with sustained output. Launching it is the only way to reach an untouched world, and it is consumed doing so (rationale in `gdd.md` §4).

- The launch is instant and silent. Give it the sequence the GDD asks for: a countdown, the ship leaving, an arrival vignette on the new world.
- Give the stages consequences beyond cost — new capabilities, a changed skyline, a reason to reach stage three other than reaching stage four.
- Tie the ship to research: every stage is buildable from turn one, so the tree and the megastructure never meet.
- Add campaign-complete and terminal-failure states; `GamePhase` now has a SeedShip screen but still no ending.
- Decide whether infrastructure collapse can actually end a run, and design the difficulty curve around it — today it is a 20-second timeout with a data penalty.
- Make the five core stages mechanical, not just visual: tie Crash-Lander/Fortress/Space Elevator/Planetary Ring (GDD §5) to research and throughput milestones with new capabilities each.

## The logistics puzzle

Pillar decided 2026-08-01: **drones route along the conduit network** (rationale
in `gdd.md` §3). Drones now walk Conduit/Power Node/Core tiles only, stop and
flag when a run is cut, and resume when it is repaired.

A network tile passes `conduit_capacity` drones at full speed; past that they share it, so a shared trunk slows everything routed through it. Saturated tiles are outlined on the map and raise a bottom-bar alert. A drill's crew size is a stat, so `swarm_dispatch` research puts a second drone on every drill.

- A drill still dispatches at most one drone per tick, and only when a whole load is waiting, so a crew of two only helps on runs longer than the drill's fill time. Dispatching a partial load when a drone is idle and the run is long would make the crew worth having everywhere.
- Congestion slows drones but never queues them: they pass through each other on a full tile rather than waiting. A real queue would make a junction readable at a glance.
- Route cost ignores traffic — `route_over_network` still picks the shortest path even when it is the saturated one. Weighting by load would let drones spread across parallel runs by themselves.
- Re-validating every in-flight drone's remaining path each tick is O(drones x path length); add a network revision counter and re-check only when the grid changes if it shows up in profiling.
- A stalled drone shows an error flag and a HUD counter, but nothing points at *where* the break is; highlight the severed run on the map.
- Add ore deposits with richness and depletion — every drill cuts the same `drill_output_rate` from any tile, so placement is spatially meaningless.
- Make Bridge tiles real; they are a bool flag that does not even transmit power.

## Interplanetary meta-layer

`state::Campaign` owns all five worlds, the current index, and the directive; travel keeps every planet exactly as it was left, and the save carries the lot.

- Simulate colonized planets in the background (scheduled tick or summary) so left-behind worlds keep producing. `Campaign` holds them all, so the missing piece is a cheap off-screen model, not the plumbing.
- Turn Mass Drivers into gameplay: a building, export schedules, transit time, and receiving landing pads. The tech now gates Seed Ship launches, so it is no longer a flag that does nothing, but nothing is exported over it yet.
- Hoist research to the campaign. Every `PlanetState` carries its own `research` copy and only the current planet's is authoritative, kept in step by `sync_research_to_planet` on travel — a left-behind world's stat sheet is stale, which will matter the moment background simulation lands.
- Worlds are now defined in `assets/planets.json` (size, terrain weights, banned buildings, arrival line) and the map reads the same file, so identity lives in one place. What is still uniform: every world generates from one flat distribution with a cleared centre, so none of them have landmarks, regions, or a shape worth reading.

## Planet hazards

- Zone 2 (Venus): acid rain degrading standard conduits, Ceramic Plating and Shield Generator counters, void-heavy volcanic terrain.
- Zone 3 (Cryo): freeze slowing drones 50%, Heater Nodes along the network, no solar or wind.
- Per-planet constraints so far are only a ban list — no wind on Venus or Saturn, no harvesters where nothing grows. The positive half is missing: infinite geothermal, fusion-only worlds, generators that only exist somewhere.
- Add the Heat mechanic the GDD gives Server Banks; water tiles carry a "may provide cooling" comment with no logic behind it.

## Research

Techs declare their effects as modifiers in `research.json` (`engine::modifiers`), validated at load; the stringly-typed `unlocked_techs.contains(...)` reads are gone.

- Expand the tree well past 15 nodes, with per-planet branches and hazard counters as research.
- The research view shows a node's prose description but not the stats it moves; render the declared modifiers so the tree explains itself.
- Add stats for the values still fixed in Rust: drone speed, repeater range, collapse thresholds, harvest yields.

## Content

- Add production chains: intermediate products, recipes, and processing buildings. A drill now buffers ore until a drone takes it, but everything still lands in one global mineral pool.
- Grow the building set beyond 10 across processing, logistics, hazard counters, and megastructure parts.
- Tier the resource set beyond Minerals/Energy/Data/Biomass to support chains and mass-driver strategy.
- Larger and more varied maps with landmark features. Sizes are per-world data now (20x20 to 26x26) and the camera does not cap them, but the generator has no notion of a feature.
- Replace the four `tier % 4` directives and four hardcoded achievements with a real objective/milestone system and a full achievement set. The Power Surplus directive still uses one number as both the power threshold and the seconds it must be held, so the two scale together by accident.
- Write the GDD's "indifferent optimizer" tone into directives and research descriptions. Arrival lines have it (`planets.json`), and the Seed Ship stages do; everything else still reads like a spreadsheet.

## Simulation architecture

The sim runs on a fixed 1/30s timestep with an accumulator (`PlanetState::advance`), capped at 6 catch-up steps; research and directives advance on exactly the time the planet simulated.

- Offline catch-up still steps at a coarser 1s, because four hours at the live tick rate is 432,000 steps on load. Unify it when the offline model becomes an earnings report (see Save system).
- Extend the deterministic snapshot tests past harvest throughput: power failure, research unlocks, and collapse thresholds still have no pinned numbers.
- Move the remaining hardcoded balance constants into JSON — dust rates, sweeper/filter radii, and collapse timings are Rust consts, and `conduit_throughput` and `core_power_consumption` are still dead config fields.
- Validate the rest of the data at load the way research modifiers now are; `game_data().building(id)` still panics on a missing id with no context.

## Save system

- Add autosave on interval, on quit, and on travel; today saving is manual or on entering the menu.
- The save is versioned (`SaveGame`, version 1) and reads the old unversioned single-planet save, but there is no migration *framework* — the next shape change needs a real per-version upgrade path rather than a second fallback branch.
- Add multiple slots and corruption recovery with backup rotation.
- Persist the meta-state the campaign does not hold: settings and tutorial progress are still lost on reload.
- Harden offline progress with a clock-tamper guard and hard caps, and turn the offline banner into an earnings report.

## UX and UI

- Add pause and game speed — the HUD advertises "PAUSE Space" and shows speed buttons whose return values are ignored.
- Wire or remove the controls that are still decoration: BOX SELECT and the BUILD/DEMOLISH hints do nothing, and the main-menu Quit button is a no-op. (PAN and ZOOM now work: middle-drag and wheel, per-planet camera.)
- Replace the six-condition text checklist with a real persisted tutorial with highlighting and interactive gating.
- Add a notification/toast system; achievements currently unlock with no feedback beyond a counter, and finishing a Seed Ship stage passes silently.
- The right-hand panel stack is full at 720p — a fifth panel overflows the four existing ones, which is why the Seed Ship got its own screen. Any new readout needs the stack's internal layouts made height-aware first.
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

- Expand test coverage: research effects, save migration, and placement rules still have none. Wire the headless capture harness scenes (`mainmenu`, `research`, `logistics`) into CI.
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
