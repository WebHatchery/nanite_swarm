//! Logistics routing over the conduit network.
//!
//! Drones do not fly. They walk the network the player laid down: conduits,
//! power nodes and the Core carry traffic, everything else does not. A route
//! exists only if an unbroken chain of those tiles connects the two endpoints,
//! which is what makes conduit layout — not straight-line distance — the thing
//! the player is actually optimising.

use super::{Grid, GridPos};
use macroquad_toolkit::pathfinding::{find_path_with, Heuristic, Pos};

/// Whether a drone may travel through this tile.
///
/// Endpoints (a drill, a Core) are handled by the caller; this is about the
/// tiles in between. A fully dust-stalled conduit is treated as broken pipe.
pub fn tile_carries_traffic(grid: &Grid, pos: GridPos) -> bool {
    grid.get(pos)
        .and_then(|tile| tile.building.as_ref())
        .map(|building| building.carries_traffic() && !building.is_dust_stalled())
        .unwrap_or(false)
}

/// Shortest route from `from` to `to` across the conduit network, counting
/// every tile the same.
///
/// Returns the tiles to walk, excluding `from` and including `to`, or `None`
/// when the network is broken between them. An empty route means the drone is
/// already there.
pub fn route_over_network(grid: &Grid, from: GridPos, to: GridPos) -> Option<Vec<GridPos>> {
    route_over_network_weighted(grid, from, to, |_| 1.0)
}

/// The same route, with each tile costing whatever `tile_cost` says it does.
///
/// This is what makes a second parallel run worth laying: a saturated trunk
/// can be made to cost more than the detour around it, and drones spread
/// across the network by themselves rather than all queueing on the shortest
/// path. A cost below one tile is clamped away — the search leans on costs
/// never being cheaper than a step for its estimates to hold.
pub fn route_over_network_weighted(
    grid: &Grid,
    from: GridPos,
    to: GridPos,
    tile_cost: impl Fn(GridPos) -> f32,
) -> Option<Vec<GridPos>> {
    if from == to {
        return Some(Vec::new());
    }

    find_path_with(
        Pos::new(from.x, from.y),
        Pos::new(to.x, to.y),
        grid.width as usize,
        grid.height as usize,
        |pos| {
            let pos = GridPos::new(pos.x, pos.y);
            pos == from || pos == to || tile_carries_traffic(grid, pos)
        },
        |pos| tile_cost(GridPos::new(pos.x, pos.y)).max(1.0),
        Heuristic::Manhattan,
        false,
    )
    .map(|path| {
        path.waypoints
            .into_iter()
            .skip(1)
            .map(|pos| GridPos::new(pos.x, pos.y))
            .collect()
    })
}

/// What crossing a tile is worth to a router, given how many drones are
/// already headed across it.
///
/// At or under capacity a tile is worth exactly one step. Every drone past the
/// limit adds `penalty`, so a five-tile trunk carrying twice what it should can
/// be worth a ten-tile detour, and the detour stops looking worth it again the
/// moment the trunk clears.
pub fn traffic_cost(load: u32, capacity: f32, penalty: f32) -> f32 {
    let over = (load as f32 - capacity.max(1.0)).max(0.0);
    1.0 + over * penalty.max(0.0)
}

#[cfg(test)]
mod tests;
