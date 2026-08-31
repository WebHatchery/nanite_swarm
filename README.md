# Nanite Swarm

Nanite Swarm is an automation strategy game about building a self-replicating machine network that consumes planetary resources and expands across worlds.

The swarm is powerful, but every expansion changes the terrain, stresses the grid, and creates new logistical problems.

## Gameplay

- Place buildings on a tile grid to harvest and process resources.
- The opening tutorial now carries the player from the first drill through the
  Efficient Drills and Power Efficiency prerequisites to the researched and
  built Mass Driver that opens the campaign's next world; research lessons
  pulse and center the next neural node on touch-sized views.
- The opening lesson belongs to the starting world; newly colonized worlds
  keep the arrival notice and visible controls without replaying steps that
  may be forbidden by their own terrain and power rules.
- Hostile arrivals name the next visible RESEARCH or BUILD control for their
  authored power route in the landing banner and build palette, following any
  research prerequisites instead of leaving the dossier's warning behind.
- After the power route is standing, the same handoff continues to the
  world's hazard response: Ceramic Plating on acid worlds and Heater Nodes on
  cold worlds.
- Completing the opening lesson points to the visible SHIP control for the
  Seed Ship yard, then MAP for choosing the next world.
- Connect structures through conduits and power networks.
- Use drones to move minerals and keep production flowing.
- Route ore and alloy through multi-input precision assemblers to make flight-grade components.
- Harvest terrain for short-term gain with lasting consequences.
- Preserve forests to buffer nearby pollution, harvest them for biomass, or
  convert them into stronger dust filters. Select an intact forest to see its
  green buffer radius on the map.
- Surveyed planetary features such as landing craters, metal belts, and ice
  rifts can be selected on their revealed tiles to read their local dossier;
  the Iron Belt grants +50% drill output, the Acid Sea raises local machinery
  corrosion by 80%, the Ring Shadow deepens local cold by 40%, the Ice Rift
  improves nearby water cooling by 50%, and the Landing Crater returns 25% more
  terrain harvest on its tiles.
- Research upgrades and prepare for interplanetary expansion.
- Manage dust, power failures, and resource bottlenecks.
- Dust-response research changes the live production, drone, routing, and
  power thresholds—not just the research readout.
- Read each colonized world's live output, power balance, workforce, stockpiles,
  and alert cause from the Solar System dossier before travelling there.
- The dossier also shows remote Data, biomass, Alloy, and Components stockpiles
  plus their live generation/harvest rates, so a world can be managed without
  first travelling. Remote storage saturation is marked as `STORE <pool>` in
  the world list and named as `FULL ORE`, `FULL DATA`, or another affected pool
  in the dossier when a live producer has nowhere to put its output.
- The Solar System dossier also previews each world's construction bans and
  infrastructure requirements before the Seed Ship leaves, including the
  authored Geothermal and Fusion power routes for hostile worlds.
- A colonized world's dossier also names its standing freight route, nominal
  cargo rate, and live route status, so remote supply chains can be checked
  from the Worlds view before opening Shipping controls.
- The same remote dossier shows each world's current and next Core stage plus
  its first remaining Core milestone, so strategic growth remains actionable
  even when the swarm is managing another world.
- Colonized Solar System nodes carry a `C1-C5` progress marker, with the
  Space Elevator tether and Planetary Ring silhouette echoed on the map.
- The touch-sized planet list repeats the same `C1-C5` cue when a compact
  layout hides the orbit diagram.
- At Space Elevator and Planetary Ring stages, the map grows a live tether and
  orbital ring around the Core; the final stage also projects a quieter ring
  into the planetary backdrop so late evolution is visible beyond its tile.
- The completion report adds a world-by-world Core ledger, keeping each final
  stage and standing structure count visible after the last launch.
- Records also shows the campaign-wide lifetime resource sinks while the run
  is still in progress, including power draw, research Data, and factory input.

## Goal

Grow the nanite core from a small foothold into a planetary-scale swarm without collapsing your own infrastructure.

## Controls

Every required action has a visible tap/click control. Tap a building card and
then the grid to build; with CONDUIT selected, drag across the grid to lay a
route or tap individual tiles. Smart Drag also lets you drag from an existing
building to a destination to extend its conduit route. Drag elsewhere to pan,
pinch or tap the visible `-` / `+` zoom controls, and use the bottom command bar for selection, demolition,
blueprints, lossless relocation, undo, speed, and pause. RELOCATE moves a
structure with its staged cargo and drill crew intact. Tap FOCUS to collapse both
sidebars for route planning, then tap PANELS to restore them. When freight is
moving, its map key identifies the colour and silhouette of every live cargo.
On phones, tap `CORE` beside `HELP` to recenter the map after panning.
The in-flight shipping ledger names both the source and destination world for
each pod under a `full route` heading, so simultaneous routes remain
distinguishable.
Tap CANCEL to abandon a build, relocation, box-selection, or pending purge
confirmation before the next map tap.
Tap the Core to inspect its current evolution stage, the next stage, and the
remaining milestone requirements holding that evolution back.
Select a revealed tile inside a named planetary feature to read its feature
description alongside the terrain details; unrevealed tiles keep that
information hidden. The feature header names any local production or hazard
effect before machinery is placed. Harvest previews include active research
and local feature bonuses, so the reward shown beside HARVEST matches the tap.
Placed structures keep their local feature effect in the inspector header.
Mountain Wind Turbines keep their `+100%` efficiency payoff there as well.
If a dragged conduit route spends its last resources, the route pauses and
reports the exact mineral and energy cost of the next segment once per gesture.
The bottom graph records observed ore, alloy, and parts flow; tap its visible
FLOW button to cycle through live power and Data production/consumption. Warning
triangles and the Operations count identify powered processors that are missing a feed.
The top resource cards show live rates from active drills, the Core and Server
Banks, and biomass harvesters. Every stored resource also shows its live cap;
when production has nowhere to go, the rate changes to FULL so the swarm can
expand storage or redirect the line.
Remote maintenance rows also name `MAINT DUST`, `MAINT ACID`, or `MAINT MIXED`.
The desktop Operations panel's Achievements row is touch-sized as well as
mouse-clickable, so Records remains reachable without a physical keyboard.
Records keeps achievements, directive progress, and the durable event log in
one touch-scrolling view; its footer names both lists as scroll targets.
On compact Research, Seed Ship, and Solar System screens, transient notices
stay in that durable log so they do not cover controls or scrollable panels.
The selected-structure Power row follows the standing building's live draw or
generation, including dust, overclock, standby, focus, and research effects;
build previews continue to show nominal power.
The Factory Deck control room reports whether the selected focus is live,
limited, waiting on feed or dispatch space, paused by standby or dust, offline,
or aimed at a deck with no machines yet.
When the deck is open, its focused machines carry a pulsing map frame so the
scope of the production trade is visible even when the control-room panel is
covering the center of the world; the frame dims offline machines and changes
to warning or error colours when feed, standby, dust, or dispatch pressure is
the local blocker. The deck footer carries the same LIVE, WAIT, FULL, and
OFFLINE legend in-game. A limited focus also exposes TRACE to cycle through
the affected machine and recenter the map on it.
When the bottom alert card reports a logistics problem, tap its JUMP button to
cycle through the affected structures or network tiles; power-only problems
keep the FLOW button for the network diagnosis overlay.
The same JUMP path points to buildings where dust has started reducing output
or corrosion is nearing a network failure, and the Solar System dossier marks
those remote worlds MAINT.
The desktop alert card names the same local cause as `UPKEEP DUST`,
`UPKEEP ACID`, or `UPKEEP MIXED` before JUMP focuses the first recovery.
On phones, that control keeps the cause compact as `JUMP DUST`, `JUMP ACID`,
or `JUMP MIXED` when upkeep is the active recovery.

When a remote freight order is selected, switching to WORLDS keeps that
source's dossier selected. If its route needs a receiving pad, the dossier
names the destination row to tap; if the driver is the blocker, it names the
source row instead. On desktop, the edited source row also carries an accent
outline so the order remains anchored while the orbit map is in view.
When it reports SAVE FAILED, tap RETRY to attempt the campaign save again.
On the title screen, SAVE appears only after New Game or Continue/Load, so an
existing slot cannot be overwritten before a campaign is selected.
The slot selector marks the chosen slot as SAVED or EMPTY before you switch it.
When it reports BACKUP RECOVERED, tap SAVE NOW to establish a fresh primary
save after recovery.
Impossible world or freight references in a save are rejected before
simulation, so the rotated backup can be used instead of crashing on load.
Malformed world grids are rejected the same way when their dimensions no
longer match their stored tiles.
Persisted drones with broken routes or invalid flight values are rejected
before their first movement update as well.
Each saved world must retain exactly one Core, the anchor used to rebuild its
power and logistics state.
World configuration, resource pools, machine wear, ground values, research
progress, and staged cargo are checked before simulation too; derived building
unlocks are rebuilt from the campaign research mirror when the save loads.
When it reports LOW BATTERY, tap FLOW to inspect the power network before the
swarm continues in hibernation.
While away, the foreground world continues its selected research job and a
committed Seed Ship yard at the same capped rates as live play; the return
banner reports the Data spent and any completed ship stages.
Tap the event-coded NEXT button to jump to the next recovery, research-unlock,
or mass-driver timer while keeping the whole campaign clock in sync.
The Solar System map recentres and scales its orbit rings on narrow windows;
the planet list remains a touch-sized travel and launch control.
Established freight orders stay visible as cargo-coloured lanes with direction
markers between the worlds, even while every pod is on the ground.
On desktop, tap a freight lane to select its source order in Shipping; planet
nodes retain their travel and launch action.
Remote standing orders show their destination, cargo rate, and the next
fixable route blocker, so an offline driver or missing landing pad can be found
without opening each order in turn; the same two-line ledger remains readable
inside the portrait Shipping drawer.
If a route is pinned to one landing pad and that pad is demolished or loses
power, the route is marked unavailable until the Landing pad control is cycled.
Pods already in flight on that route follow the recovered pad choice once the
route is cycled, preserving cargo that was waiting above the world.
Its header distinguishes a Mass Driver that is not built from one that is
built but offline, and explains when a finished Seed Ship still needs a live
driver.
Tap an untouched world once to prepare a launch, then tap its highlighted
`CONFIRM` row again to confirm; tap `CANCEL` to clear the armed launch.
Travel to an already-colonized world remains one tap.
At compact browser sizes, the resource cards abbreviate and the build/right
panels use short summaries so touch controls remain separated and visible.
At short desktop heights or tablet widths, Solar System uses full-width Worlds
and Shipping panels so the dossier, route controls, and footer do not collide.
After a world change, the mobile header briefly names the arrival as
`ARRIVED <WORLD>` while the desktop banner keeps the full arrival line.
Portrait world dossiers repeat the selected world's alert code in the status
line, so a remote maintenance cause stays visible after tapping its row.
On portrait screens, BUILD and INFO open one full-width drawer at a time;
MAP returns to the live grid between actions.
The mobile gameplay header also keeps the active research job's DATA progress
visible beneath the status line, including a warning colour while collapse
recovery has research locked.
Research uses a separate control row and full-width detail drawer on phones;
drag the neural tree with a mouse or touch to pan it, then tap `-` / `+` to
zoom;
the Seed Ship yard compresses each stage into a readable card, keeps the
diversion control visible, and opens the neural network directly when a stage
  needs research. Tap Back there to return to the yard. The portrait Solar System
  screen separates Worlds
  and Shipping into touch-sized tabs, with the freight controls scrollable below.
  Shipping distinguishes a missing Landing Pad from a built but offline one, so
  held pods explain whether the destination needs construction or power; the
  standing order also shows its nominal cargo throughput per minute. Tap
  Shipping reserve to cycle the source floor through OFF, 100, 250, 500, and
  1000 units; surplus routes will not drain a world down to that floor. The
  setting follows remote standing orders as well as the local route.
  The title and Settings screens also switch to single-column phone layouts so
  the command stack, display toggles, audio controls, and pause-key control
  remain on-screen.
  At short desktop heights, the Seed Ship yard scrolls its stages while keeping
  RESEARCH and DIVERT/HALT controls visible above the footer.
The Audio tab includes a visible Test signal control for checking the current
volume mix without entering a campaign.
Deleting a save slot requires a second tap on its explicitly labelled
confirmation state.
The mobile FLOW ledger and Factory Deck use exclusive, full-width overlays;
HELP reserves the command bar so diagnostics and recovery controls stay clear.
The ending report compresses to two short lines and keeps both choices inside
very short phone viewports.

Factory controls:

- Tap FLOW for live routes, recipe-buffer gauges, bottlenecks, installed
  capacity, utilization, priority badges, and traffic saturation.
- Tap DECK for the Factory Control Room: see how far the base has reached from
  Surface to Orbital, choose Balanced, Ore, Alloy, or Parts focus, and read the
  live production/power trade in one compact overlay. The deck also shows the
  current Core stage, next stage, and live milestone counters. The DECK control
  remains visible in the compact command bar so the overlay is always
  touch-closable.
- Research Adaptive Clocking, then tap BOOST for 1.5x work at 1.75x power
  and 1.8x dust, or tap AUTO CLOCK to manage healthy lines automatically.
- Use BOX SELECT to boost, prioritize, pause, resume, or safely purge a
  production district with visible touch controls.
- Tap STANDARD / PRIORITY to give one line first claim on routed inputs while
  preserving demand balancing among equal peers.
- Tap STANDBY to stop a line's production and base power draw without losing
  either buffer; tap RESUME to restart it.
- A processor on STANDBY cannot be boosted until it is resumed; bulk BOOST
  skips paused lines automatically.
- A full dispatch pad stops its recipe without consuming inputs. Tap PURGE PAD
  then PURGE AGAIN, or use the equivalent two-tap district purge.
- Processor floor art shows intake fill, product stacks, active packets,
  priority beacons, boost cadence, and red backpressure shutters.
- Research Buffer Lattices to expand every processor dispatch pad by 50%.
- A Seed Ship needs a powered Mass Driver on the world it leaves; research
  unlocks the rail, but the infrastructure still has to stand.
- Records reward boosted clusters, staged freight, priority lattices, and
  districts held in standby.

Keyboard shortcuts are optional:

- 1-5: select buildings.
- H: harvest terrain.
- R: research view.
- M: interplanetary map.
- Esc: main menu.
- F1: help overlay.

## Current Scope

Playable solar campaign with persistent worlds, routed multi-stage factories,
physical drone logistics and congestion, research, evolving Cores, hazardous
planet conditions, interplanetary freight, staged Seed Ships, collapse and
recovery, save slots, offline progress, and a campaign ending.

## Documentation

- `gdd.md` — the design document: the logistics puzzle, the terrain dilemma, the solar campaign, and the evolving core.
- `TODO.md` — the remaining architectural follow-up work.
