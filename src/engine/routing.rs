//! Logistics routing over the conduit network.
//!
//! Drones do not fly. They walk the network the player laid down: conduits,
//! power nodes and the Core carry traffic, everything else does not. A route
//! exists only if an unbroken chain of those tiles connects the two endpoints,
//! which is what makes conduit layout — not straight-line distance — the thing
//! the player is actually optimising.

use super::{Grid, GridPos};
use macroquad_toolkit::grid::bfs_path as toolkit_bfs_path;

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

/// Shortest route from `from` to `to` across the conduit network.
///
/// Returns the tiles to walk, excluding `from` and including `to`, or `None`
/// when the network is broken between them. An empty route means the drone is
/// already there.
pub fn route_over_network(grid: &Grid, from: GridPos, to: GridPos) -> Option<Vec<GridPos>> {
    if from == to {
        return Some(Vec::new());
    }

    toolkit_bfs_path(
        from.to_tile_pos(),
        to.to_tile_pos(),
        false,
        |pos| GridPos::from_tile_pos(pos).in_bounds(grid.width, grid.height),
        |pos| {
            let pos = GridPos::from_tile_pos(pos);
            pos == from || pos == to || tile_carries_traffic(grid, pos)
        },
    )
    .map(|path| {
        path.into_iter()
            .skip(1)
            .map(GridPos::from_tile_pos)
            .collect()
    })
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
