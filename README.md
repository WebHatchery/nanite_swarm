# Nanite Swarm

Nanite Swarm is an automation strategy game about building a self-replicating machine network that consumes planetary resources and expands across worlds.

The swarm is powerful, but every expansion changes the terrain, stresses the grid, and creates new logistical problems.

## Gameplay

- Place buildings on a tile grid to harvest and process resources.
- Connect structures through conduits and power networks.
- Use drones to move minerals and keep production flowing.
- Route ore and alloy through multi-input precision assemblers to make flight-grade components.
- Harvest terrain for short-term gain with lasting consequences.
- Research upgrades and prepare for interplanetary expansion.
- Manage dust, power failures, and resource bottlenecks.

## Goal

Grow the nanite core from a small foothold into a planetary-scale swarm without collapsing your own infrastructure.

## Controls

Every required action has a visible tap/click control. Tap a building card and
then the grid to build; drag the map to pan, pinch or use the visible zoom
controls to zoom, and use the bottom command bar for selection, demolition,
blueprints, relocation, undo, speed, and pause. Tap FOCUS to collapse both
sidebars for route planning, then tap PANELS to restore them. When freight is
moving, its map key identifies the colour and silhouette of every live cargo.
The bottom graph records observed ore, alloy, and parts flow; warning triangles
and the Operations count identify powered processors that are missing a feed.
Tap FLOW to trace live material routes and recipe-buffer health over the map.
Research Adaptive Clocking, then select a processor and tap BOOST for 1.5x work at 1.75x power and 1.8x dust.
Tap FLOW to open live supply routes and the factory ledger: processor health, the dominant missing input, and observed ore/alloy/components rates.
Use BOX SELECT to mark a factory block, then tap BOOST N or NORMAL N to change every selected processor together.
Tap AUTO CLOCK to let the swarm boost only fed processors with spare power and low dust, normalizing them when those margins disappear.

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
- `TODO.md` — the open work, from the 2026-07-14 code audit against the GDD.
