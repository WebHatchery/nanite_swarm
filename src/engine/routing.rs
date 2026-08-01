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
mod tests {
    use super::*;
    use crate::engine::BuildingType;

    /// Core at (0, 0), a conduit run east, and a drill at the end of it.
    fn line_network(length: i32) -> (Grid, GridPos, GridPos) {
        let mut grid = Grid::new(length as u32 + 3, 3);
        grid.reveal_around(GridPos::new(0, 0), 64);
        let core = GridPos::new(0, 0);
        grid.place_building(core, BuildingType::Core);
        for x in 1..=length {
            grid.place_building(GridPos::new(x, 0), BuildingType::Conduit);
        }
        let drill = GridPos::new(length + 1, 0);
        grid.place_building(drill, BuildingType::Drill);
        (grid, core, drill)
    }

    /// Two runs from the drill to the Core: a short one along the top and a
    /// longer one that dips down a row and comes back.
    fn two_runs() -> (Grid, GridPos, GridPos, Vec<GridPos>, Vec<GridPos>) {
        let mut grid = Grid::new(8, 4);
        grid.reveal_around(GridPos::new(3, 1), 64);
        let core = GridPos::new(0, 0);
        let drill = GridPos::new(5, 0);
        grid.place_building(core, BuildingType::Core);
        grid.place_building(drill, BuildingType::Drill);

        let short: Vec<GridPos> = (1..=4).map(|x| GridPos::new(x, 0)).collect();
        let long: Vec<GridPos> = [
            GridPos::new(5, 1),
            GridPos::new(4, 1),
            GridPos::new(3, 1),
            GridPos::new(2, 1),
            GridPos::new(1, 1),
            GridPos::new(0, 1),
        ]
        .into_iter()
        .collect();
        for pos in short.iter().chain(long.iter()) {
            grid.place_building(*pos, BuildingType::Conduit);
        }
        (grid, core, drill, short, long)
    }

    #[test]
    fn a_tile_under_its_limit_costs_one_step_and_a_crowded_one_costs_more() {
        assert_eq!(traffic_cost(0, 2.0, 1.5), 1.0);
        assert_eq!(
            traffic_cost(2, 2.0, 1.5),
            1.0,
            "at capacity is still a step"
        );
        assert_eq!(traffic_cost(4, 2.0, 1.5), 1.0 + 2.0 * 1.5);
        // A penalty of nothing is how the old behaviour is spelled.
        assert_eq!(traffic_cost(9, 2.0, 0.0), 1.0);
    }

    #[test]
    fn the_shortest_run_is_taken_while_it_is_clear() {
        let (grid, core, drill, short, _) = two_runs();
        let route = route_over_network(&grid, drill, core).unwrap();
        for pos in &short {
            assert!(route.contains(pos), "{:?} was not on the route", pos);
        }
    }

    #[test]
    fn a_saturated_trunk_is_worth_going_around() {
        let (grid, core, drill, short, long) = two_runs();
        // Four drones on a run that passes two: the detour is now cheaper.
        let route = route_over_network_weighted(&grid, drill, core, |pos| {
            if short.contains(&pos) {
                traffic_cost(4, 2.0, 1.5)
            } else {
                1.0
            }
        })
        .unwrap();
        for pos in &long {
            assert!(route.contains(pos), "{:?} was not on the detour", pos);
        }
        for pos in &short {
            assert!(!route.contains(pos), "{:?} was still used", pos);
        }
    }

    #[test]
    fn a_busy_trunk_that_is_still_the_only_way_home_is_used_anyway() {
        // Congestion makes a tile expensive, never impassable: a single run
        // carrying everything must not strand the swarm.
        let (grid, core, drill) = line_network(4);
        let route = route_over_network_weighted(&grid, drill, core, |_| traffic_cost(20, 2.0, 1.5));
        assert_eq!(route.unwrap().len(), 5);
    }

    #[test]
    fn route_over_network_follows_the_conduit_run() {
        let (grid, core, drill) = line_network(4);
        let route = route_over_network(&grid, drill, core).unwrap();
        assert_eq!(
            route,
            vec![
                GridPos::new(4, 0),
                GridPos::new(3, 0),
                GridPos::new(2, 0),
                GridPos::new(1, 0),
                core,
            ]
        );
    }

    #[test]
    fn route_over_network_is_none_when_the_run_is_broken() {
        let (mut grid, core, drill) = line_network(4);
        grid.remove_building(GridPos::new(2, 0));
        assert!(route_over_network(&grid, drill, core).is_none());
    }

    #[test]
    fn route_over_network_ignores_open_ground_shortcuts() {
        // A U-shaped run: open ground would be a 2-tile hop, the network is 6.
        let mut grid = Grid::new(5, 5);
        grid.reveal_around(GridPos::new(2, 2), 64);
        let core = GridPos::new(0, 0);
        grid.place_building(core, BuildingType::Core);
        for pos in [
            GridPos::new(0, 1),
            GridPos::new(0, 2),
            GridPos::new(1, 2),
            GridPos::new(2, 2),
            GridPos::new(2, 1),
        ] {
            grid.place_building(pos, BuildingType::Conduit);
        }
        let drill = GridPos::new(2, 0);
        grid.place_building(drill, BuildingType::Drill);

        let route = route_over_network(&grid, drill, core).unwrap();
        assert_eq!(route.len(), 6);
    }

    #[test]
    fn route_over_network_travels_through_power_nodes() {
        let mut grid = Grid::new(5, 3);
        grid.reveal_around(GridPos::new(2, 0), 64);
        let core = GridPos::new(0, 0);
        grid.place_building(core, BuildingType::Core);
        grid.place_building(GridPos::new(1, 0), BuildingType::PowerNode);
        let drill = GridPos::new(2, 0);
        grid.place_building(drill, BuildingType::Drill);
        assert_eq!(
            route_over_network(&grid, drill, core),
            Some(vec![GridPos::new(1, 0), core])
        );
    }

    #[test]
    fn a_dust_stalled_conduit_breaks_the_route() {
        let (mut grid, core, drill) = line_network(4);
        grid.get_mut(GridPos::new(2, 0))
            .unwrap()
            .building
            .as_mut()
            .unwrap()
            .dust = 100.0;
        assert!(route_over_network(&grid, drill, core).is_none());
    }

    #[test]
    fn route_over_network_returns_empty_for_the_same_tile() {
        let (grid, core, _) = line_network(1);
        assert_eq!(route_over_network(&grid, core, core), Some(Vec::new()));
    }

    #[test]
    fn a_drill_adjacent_to_the_core_needs_no_conduit() {
        let mut grid = Grid::new(4, 4);
        grid.reveal_around(GridPos::new(1, 1), 8);
        let core = GridPos::new(1, 1);
        grid.place_building(core, BuildingType::Core);
        let drill = GridPos::new(2, 1);
        grid.place_building(drill, BuildingType::Drill);
        assert_eq!(route_over_network(&grid, drill, core), Some(vec![core]));
    }

    #[test]
    fn buildings_that_do_not_carry_traffic_are_not_a_route() {
        let mut grid = Grid::new(5, 3);
        grid.reveal_around(GridPos::new(2, 0), 64);
        let core = GridPos::new(0, 0);
        grid.place_building(core, BuildingType::Core);
        // A Storage block is not pipe; it does not join the network.
        grid.place_building(GridPos::new(1, 0), BuildingType::Storage);
        let drill = GridPos::new(2, 0);
        grid.place_building(drill, BuildingType::Drill);
        assert!(route_over_network(&grid, drill, core).is_none());
    }
}
