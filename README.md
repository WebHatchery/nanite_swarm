# Nanite Swarm

Nanite Swarm is an automation strategy game about building a self-replicating machine network that consumes planetary resources and expands across worlds.

The swarm is powerful, but every expansion changes the terrain, stresses the grid, and creates new logistical problems.

## Gameplay

- Place buildings on a tile grid to harvest and process resources.
- Connect structures through conduits and power networks.
- Use drones to move minerals and keep production flowing.
- Route ore and alloy through multi-input precision assemblers to make flight-grade components.
- Harvest terrain for short-term gain with lasting consequences.
- Preserve forests to buffer nearby pollution, harvest them for biomass, or
  convert them into stronger dust filters. Select an intact forest to see its
  green buffer radius on the map.
- Research upgrades and prepare for interplanetary expansion.
- Manage dust, power failures, and resource bottlenecks.
- Dust-response research changes the live production, drone, routing, and
  power thresholds—not just the research readout.
- Read each colonized world's live output, power balance, workforce, and alert
  cause from the Solar System dossier before travelling there.
- The dossier also shows remote Data and biomass stockpiles plus their live
  generation/harvest rates, so a world can be managed without first travelling.
- The Solar System dossier also previews each world's construction bans and
  infrastructure requirements before the Seed Ship leaves.

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
The bottom graph records observed ore, alloy, and parts flow; tap its visible
FLOW button to cycle through live power and Data production/consumption. Warning
triangles and the Operations count identify powered processors that are missing a feed.
The top resource cards show live rates from active drills, the Core and Server
Banks, and biomass harvesters.
The selected-structure Power row follows the standing building's live draw or
generation, including dust, overclock, standby, focus, and research effects;
build previews continue to show nominal power.
When the bottom alert card reports a logistics problem, tap its JUMP button to
cycle through the affected structures or network tiles; power-only problems
keep the FLOW button for the network diagnosis overlay.
The same JUMP path points to buildings where dust has started reducing output
or corrosion is nearing a network failure, and the Solar System dossier marks
those remote worlds MAINT.
When it reports SAVE FAILED, tap RETRY to attempt the campaign save again.
When it reports BACKUP RECOVERED, tap SAVE NOW to establish a fresh primary
save after recovery.
When it reports LOW BATTERY, tap FLOW to inspect the power network before the
swarm continues in hibernation.
Tap the event-coded NEXT button to jump to the next recovery, research-unlock,
or mass-driver timer while keeping the whole campaign clock in sync.
The Solar System map recentres and scales its orbit rings on narrow windows;
the planet list remains a touch-sized travel and launch control.
Its header distinguishes a Mass Driver that is not built from one that is
built but offline, and explains when a finished Seed Ship still needs a live
driver.
At compact browser sizes, the resource cards abbreviate and the build/right
panels use short summaries so touch controls remain separated and visible.
On portrait screens, BUILD and INFO open one full-width drawer at a time;
MAP returns to the live grid between actions.
Research uses a separate control row and full-width detail drawer on phones;
the Seed Ship yard compresses each stage into a readable card, keeps the
diversion control visible, and opens the neural network directly when a stage
needs research. Tap Back there to return to the yard. The portrait Solar System
screen separates Worlds
and Shipping into touch-sized tabs, with the freight controls scrollable below.
The title and Settings screens also switch to single-column phone layouts so
the command stack, display toggles, audio controls, and pause-key control
remain on-screen.
The Audio tab includes a visible Test signal control for checking the current
volume mix without entering a campaign.
Deleting a save slot requires a second tap on its explicitly labelled
confirmation state.
The mobile FLOW ledger and Factory Deck use exclusive, full-width overlays;
HELP reserves the command bar so diagnostics and recovery controls stay clear.

Factory controls:

- Tap FLOW for live routes, recipe-buffer gauges, bottlenecks, installed
  capacity, utilization, priority badges, and traffic saturation.
- Tap DECK for the Factory Control Room: see how far the base has reached from
  Surface to Orbital, choose Balanced, Ore, Alloy, or Parts focus, and read the
  live production/power trade in one compact overlay. The DECK control remains
  visible in the compact command bar so the overlay is always touch-closable.
- Research Adaptive Clocking, then tap BOOST for 1.5x work at 1.75x power
  and 1.8x dust, or tap AUTO CLOCK to manage healthy lines automatically.
- Use BOX SELECT to boost, prioritize, pause, resume, or safely purge a
  production district with visible touch controls.
- Tap STANDARD / PRIORITY to give one line first claim on routed inputs while
  preserving demand balancing among equal peers.
- Tap STANDBY to stop a line's production and base power draw without losing
  either buffer; tap RESUME to restart it.
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
