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
fn a_void_gap_is_crossed_by_a_bridge_and_by_nothing_else() {
    // Core, two conduits, a gap of void, then the drill.
    let mut grid = Grid::new(7, 3);
    grid.reveal_around(GridPos::new(3, 0), 32);
    let core = GridPos::new(0, 0);
    let gap = GridPos::new(3, 0);
    let drill = GridPos::new(5, 0);
    grid.get_mut(gap).unwrap().terrain = crate::engine::TerrainType::Void;
    grid.place_building(core, BuildingType::Core);
    for x in [1, 2, 4] {
        assert!(grid.place_building(GridPos::new(x, 0), BuildingType::Conduit));
    }
    grid.place_building(drill, BuildingType::Drill);

    // Nothing can be laid across the void, so there is no route.
    assert!(!grid.can_place_building(gap, BuildingType::Conduit));
    assert!(route_over_network(&grid, drill, core).is_none());

    // A bridge is the crossing, not a licence to build one.
    assert!(grid.place_building(gap, BuildingType::Bridge));
    let route = route_over_network(&grid, drill, core).expect("bridged run");
    assert!(route.contains(&gap), "the route skipped the bridge");
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
