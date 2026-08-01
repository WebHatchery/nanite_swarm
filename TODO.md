# TODO — Nanite Swarm

Open work from the 2026-07-14 code audit against `gdd.md`. The single-planet
automation loop is solid; the interplanetary pitch, the win condition, and the
logistics puzzle at the heart of the GDD are the gaps.

## Core loop and win condition

The Seed Ship exists: four stages declared in `assets/seed_ship.json`, its own screen behind the SHIP button, and a commit toggle that pours production into the yard at a capped intake, so it can only be paid for with sustained output. Launching it is the only way to reach an untouched world, and it is consumed doing so (rationale in `gdd.md` §4).

- A launch now plays out: a countdown on the pad, the ship clearing the world it was built on, transit, and an arrival vignette that names the new world and gives it its line from `planets.json`. The beats, their lengths and their prose are `seed_ship.json` data (`launch`), and any key cuts it short. What it does not do is show the *base* being left — the sequence is drawn against a bare limb of the world rather than over the map the player just spent hours on.
- Each standing stage now works for the world it stands on — the Cradle feeds the drills, the Spine adds a drone per drill, the Payload sharpens data — declared as modifiers in `seed_ship.json` and lost when the ship launches with them. The ship is also on the map now: committing raises a gantry over the Core, the hull grows inside it with `built_fraction`, a finished ship pulses, and a launched one leaves bare ground. It is a schematic silhouette rather than art, it occludes whatever is built directly above the Core, and the launch vignette does not show it leaving.
- Every stage past the first is now gated on research declared in `seed_ship.json` — the Spine on Efficient Drills, the Payload on Advanced Research, the Ignition Charge on the Mass Driver — and a blocked yard sits idle rather than banking resources against a stage nobody can build yet. The ship screen says which tech it is waiting on. What the screen still will not say is what the *sealed* stages further down need, so the tree gives no advance warning of what to research next.
The campaign ends: every world taken and a finished ship with nowhere to send it shows SYSTEM CONSUMED with what the run consumed, once, and the player can carry on in the finished system. There is deliberately no failure state (rationale in `gdd.md` §5b).

- Nothing after the ending changes. A finished campaign plays exactly like an unfinished one, so there is no reason to keep the save. New-game-plus, a harder second system, or a score to beat would all give it one.
- Collapse stays a setback, never a death (decided 2026-08-01). It is still a flat 20-second shutdown and a data penalty on every world at every stage, so it stings hardest exactly when the player can least afford it and barely registers later; it wants scaling with the size of what collapsed.
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

- Left-behind worlds keep working: they run the same simulation in coarse one-second steps without visuals, and the solar map lists what each has stockpiled. One seam remains — nothing runs while the map, the menu, or the launch sequence is open, so time only passes on the surface.
- Turn Mass Drivers into gameplay: a building, export schedules, transit time, and receiving landing pads. The tech now gates Seed Ship launches, so it is no longer a flag that does nothing, but nothing is exported over it yet.
- Research belongs to the campaign, and every world adopts it — including one colonized after the fact. Each `PlanetState` still keeps a copy, because a world needs its own answer (it can refuse a building the swarm has researched), but the campaign is the only writer. What is *not* shared yet: `unlocked_buildings` is rebuilt per world from that research and never pruned, so a building unlocked and then somehow un-researched would stay unlocked.
- Worlds are now defined in `assets/planets.json` (size, terrain weights, banned buildings, arrival line) and the map reads the same file, so identity lives in one place. What is still uniform: every world generates from one flat distribution with a cleared centre, so none of them have landmarks, regions, or a shape worth reading.

## Planet hazards

Hazards are per-world data (`planets.json`). Acid rain corrodes anything carrying the network, riding the existing dust-to-stall chain so a neglected Venus run eventually breaks; the cold takes a share of drone speed. The counters are buildings placed on the map — Shield Generators and Heater Nodes, each holding 90% of one hazard off within four tiles — so covering a sprawling base is a layout and power problem. Ceramic Plating remains a small global backstop.

- Coverage is drawn for whatever upkeep building is selected or being placed, and wear is tinted onto the map, so a corroding run is visible from across the base. What is still missing is the reverse view: nothing shows which buildings are *uncovered*, which is the question you actually ask when a base sprawls.
- A hazard is uniform across a world. Acid squalls that move, or cold that bites hardest far from the Core, would give the map a shape to plan around.
- Acid and dust share one wear tint, because they share one number in the sim. If they should read differently on the map they need to be different quantities first. A frozen drone still just looks slow.
- Per-planet constraints are still only a ban list plus hazards. The positive half is missing: infinite geothermal, fusion-only worlds, generators that exist on one world and nowhere else.
- Add the Heat mechanic the GDD gives Server Banks; water tiles carry a "may provide cooling" comment with no logic behind it.

## Research

Techs declare their effects as modifiers in `research.json` (`engine::modifiers`), validated at load; the stringly-typed `unlocked_techs.contains(...)` reads are gone.

- Expand the tree well past 15 nodes, with per-planet branches and hazard counters as research.
- The research view shows a node's prose description but not the stats it moves; render the declared modifiers so the tree explains itself.
- Add stats for the values still fixed in Rust: drone speed, repeater range, collapse thresholds, harvest yields.

## Content

The first chain runs end to end and nothing teleports: a drill piles ore on its pad, its crew carries it to the nearest Smelter with room, the Smelter refines it and piles alloy on its own pad, and its own crew carries that to the Core. Both halves are the same code — a producer, a load, and whatever wants it — so a third link is data.

- Alloy has no consumer building, so it always routes to the Core. The moment something eats alloy, `dispatch_producers` will pick it the same way it picks a Smelter, but nothing does yet.
- `biomass_in` still draws from the global pool, because no recipe uses it. The next recipe that does needs the hopper to hold more than ore.
- Every producer's crew is one drone (two with `swarm_dispatch`), so a Smelter fed by three drills has the same collection capacity as one fed by one.
- One recipe, one product. Deeper chains (alloy plus biomass into something) and more processing buildings are the content this unlocks.
- `RecipeDef` is a fixed struct of named fields, so a third input needs a code change rather than a data change. It wants to be a map of resource id to amount once there are more than two.
- Grow the building set beyond 12 across processing, logistics, and megastructure parts. Hazard counters exist now; nothing else on that list does.
- Tier the resource set further. Alloy is the first refined product; mass-driver strategy still has nothing worth shipping between worlds.
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

The campaign autosaves every minute of world time, on travel, on launch, and on the way to the menu, and says so in the bottom bar. Every save rotates the previous one into a backup, and a load that cannot read the main save falls back to it and says so. A failed write raises an alert rather than passing silently.

- Nothing catches the window closing, so the last minute of an idle session is still lost if the player alt-F4s. macroquad has no close hook to hang this on; it may need a platform-specific one or a much shorter interval.
- Recovery is one save deep. A campaign that has been quietly broken for several minutes has already rotated the good copy out.
- The save is versioned (`SaveGame`, version 1) and reads the old unversioned single-planet save, but there is no migration *framework* — the next shape change needs a real per-version upgrade path rather than a second fallback branch.
- Add multiple *player-visible* slots. Rotation and recovery exist (one backup, automatic), but there is still one campaign and no way to keep a second run or go back more than one save.
- Persist the meta-state the campaign does not hold: settings and tutorial progress are still lost on reload.
- Harden offline progress with a clock-tamper guard and hard caps, and turn the offline banner into an earnings report.

## UX and UI

Space pauses, the bottom bar's speed buttons work, and both ride the fixed timestep: speed scales how much world time a second of real time buys, never how long a step is.

- Speed and pause are per-planet runtime state and reset on load, which is right for pause and arguably wrong for a preferred speed.
- There is no fast-forward past four times, and no way to skip to the next interesting moment — an idle game eventually wants both.
- Wire or remove what is still decoration: BOX SELECT (Shift+Drag) and BUILD MENU (B) do nothing, and the main-menu Quit button is a no-op. PAN, ZOOM, PAUSE and DEMOLISH all work now.
The tutorial is five steps in `assets/tutorial.json`, each with a goal the simulation checks (build a thing, research a thing, connect a thing). It persists across saves, says what to do rather than which step you are on, highlights the building it is asking for in the palette, and toasts each step as it lands. The panel is the tutorial while it runs and the directive after.

- No gating: the tutorial suggests, it never blocks. That is probably right for this game, but it means a player who ignores it entirely gets no more help than one who follows it.
- Nothing points at the *map*. A step that wants a conduit run drawn between two tiles can highlight the Conduit card but not the tiles.
Toasts (the toolkit's `NotificationManager`) announce achievements, finished research, newly available buildings, and each Seed Ship stage. They fade in real time, so they keep fading while the world is paused.

- Directives still complete silently, and the tutorial advances without saying so — both predate the toast stack and should use it.
- There is no history: a toast missed while the player was on the research screen is gone. A scrollable log would also give the "indifferent optimizer" voice somewhere to accumulate.
- The right-hand panel stack is full at 720p — a fifth panel overflows the four existing ones, which is why the Seed Ship got its own screen. Any new readout needs the stack's internal layouts made height-aware first.
- Toasts are only drawn on the planetary view, so anything that fires while the research, ship or map screen is open is missed entirely.
Settings load at startup, apply as they are changed, and are written to disk, so text scale, fullscreen and the FPS overlay survive a restart. The autosave cadence comes from `autosave_interval` rather than a constant.

- The audio sliders still drive nothing; that waits on the audio system (see Audio).
- Only display settings are applied live. There is no key remapping, and nothing in the settings screen explains what any of it does.
- Add the rest of the genre-standard build tools: relocation, blueprint stamps, undo. Demolish mode and drag-demolish work (X or the palette button; drag tears down a run), but every demolition is final and refunds half.
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
