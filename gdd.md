# GAME DESIGN DOCUMENT: TOTAL CONVERSION

*Genre: Idle / Logistics / Puzzle Strategy*  
*Platform: PC & Mobile (Hybrid)*  
*Theme: Sci-Fi, Nanite Swarm, Planetary Consumption*

## 1. High Concept

> "The Universe is just raw material waiting to be processed."

The player controls a self-replicating Artificial Intelligence designed to consume planetary bodies. Starting with a single "Core" on a finite grid, the player must harvest resources, build a logistics network, and evolve their central structure until they have consumed enough of the planet to build a Seed Ship and launch to the next world.

Unlike standard idle games, you do not lose progress when you move on. You build an Interplanetary Supply Chain, using the resources of conquered worlds to fuel the colonization of harder, more hostile planets.

## 2. Core Gameplay Loop

### The "Fast Start" (No Clicker Phase)

The game respects the player's time. There is no manual "clicking on rocks."

- **Deployment:** You land on a finite map with a starter kit of resources.
- **Connection:** You place a Drill and connect it to the Core via a Conduit.
- **Automation:** Immediately, small Rover Drones exit the Drill and carry ore to the Core.
- **Expansion:** You use resources to extend the grid, reaching new nodes and "eating" the terrain.
- **Evolution:** You research new tech to solve logistical bottlenecks.
- **Ascension:** You build the Mega-Structure, launch, and begin the cycle on a new planet.

## 3. Mechanics & Systems

### The Logistics Puzzle (The "Spaghetti")

The difficulty comes from spatial management, not just resource costs.

- **Strict Piping:** Resource pipes (Conveyors) cannot overlap.
- **The Challenge:** Crossing a Water line with an Iron line requires planning or expensive "Bridge" tiles.
- **Power Grid:** Energy spreads to adjacent buildings automatically but requires "Repeater Nodes" to travel long distances.
- **Drone Swarm:** Resources are not invisible numbers; they are physical items carried by small Worker Drones along your pipes. If a pipe is broken, the drones stop and wave an error flag.

#### Decision: drones route on the network (2026-08-01)

The pillar was open between routing drones along the conduit network and
redesigning around free-flying drones with congestion. **The network wins.**
Conduits are the logistics graph, not just a power cable: a delivery only
happens if an unbroken run of Conduit / Power Node / Core tiles joins a Drill to
the Core, and travel time is the length of that run, not the straight-line
distance.

The reasoning:

- Free flight deletes the puzzle. With drones crossing open ground the only
  spatial decision left is "build the drill nearer the Core", and the map stops
  being a thing you route through.
- Three of the campaign's hazards only exist because the network is physical:
  Acid Rain dissolving pipes, Heater Nodes placed *along* the pipe run, and
  Venus's Void gaps that force Bridges. Free flight makes all three inert.
- Congestion pressure is easier to make legible on a graph than in open space.
  Throughput belongs to a conduit segment the player can see and widen; a
  free-flight jam is invisible and unfixable.
- It is the cheaper build. Power connectivity already flood-fills the same
  graph, so routing reuses the structure the game has instead of adding a
  steering/avoidance layer.

What follows from it: conduit segments carry a throughput limit (congestion is a
property of the run, not the drone count); Bridges become real crossings rather
than a cosmetic flag; and terrain matters because it dictates where a run can
physically go.

### The Terrain Dilemma (Cannibalization)

The map is composed of tiles with distinct properties. The player must choose between Harvesting or Utilizing.

#### Mountains

- **Option A (Harvest):** Strip-mine the mountain for a massive one-time Iron bonus. Consequence: The tile becomes "Rough Ground" (difficult to build on).
- **Option B (Utilize):** Leave the mountain alone. Place a Wind Turbine on it for +200% Energy efficiency.

#### Forests

- **Option A:** Consume for Biomass/Carbon.
- **Option B:** Keep as a buffer against pollution/heat mechanics.

### The Synapse (Research)

Research is not a list; it is a visual representation of the AI's growing intelligence.

- **Visuals:** A glowing neural network. As you unlock nodes, the "brain" lights up and expands.
- **Currency:** Research requires Data. You must build Server Banks (which consume Power and generate Heat) to generate Data.

## 4. Progression: The Solar Campaign

The game takes place across a Solar System. Each planet introduces new physics and hazards.

### Zone 1: The Red Planet (Mars-like)

- **Constraint:** Low Solar Power. High Wind.
- **Hazard:** Dust Accumulation. Buildings slowly lose efficiency over time. Player must research "Self-Cleaning Servos" or deploy Sweeper Drones.

### Zone 2: The Pressure Cooker (Venus-like)

- **Constraint:** Solar is impossible (thick clouds). Geothermal is infinite.
- **Hazard:** Acid Rain. Standard pipes dissolve. Requires "Ceramic Plating" or "Shield Generators."
- **Terrain:** Highly volcanic. Lots of unbuildable "Void" gaps that require Bridges.

### Zone 3: The Cryo Giants (Saturn/Titan-like)

- **Constraint:** No Sun. No Wind. Nuclear/Fusion power only.
- **Hazard:** The Freeze. Drones move 50% slower. Player must build "Heater Nodes" along the pipe network to keep fluids moving.

### Interplanetary Logistics (The Meta-Layer)

When the player moves to Planet 2, Planet 1 does not disappear.

- **Mass Drivers:** The player can set up an export schedule. (e.g., "Launch 500 Steel/Minute from Mars to Saturn").
- **The Strategy:** Use the abundant resources of early, easy planets to brute-force the difficult puzzles of later planets.

## 5. Visuals & User Experience

### The Evolving Core

The central base (The Player) changes visually as the Tech Tree fills up.

- **Stage 1:** A simple Crash-Lander pod.
- **Stage 2:** A sprawling Fortress / Factory Hub.
- **Stage 3:** A Space Elevator (Tether extends into the sky).
- **Stage 4:** A Planetary Ring. The structure encircles the entire planet in the background.

### Offline Mechanics

- **The Battery:** The AI has a 4-hour internal battery.
- **Behavior:** The game simulates at 100% speed for 4 hours after the player logs off. After 4 hours, it enters "Hibernation" (10% speed). This encourages daily check-ins without punishing busy players.

### Controls

- **Smart Drag:** Click a building, drag a line to the destination. The game auto-places the pipes along the grid.

## 6. Narrative Tone

- **Perspective:** "The Consumer."
- **Vibe:** Indifferent, Efficient, Industrial.
- **Theme:** The transition from a chaotic natural world to a perfectly ordered, synthetic machine. You are not evil; you are just optimizing.