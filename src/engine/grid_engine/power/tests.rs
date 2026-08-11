use super::super::building_type::BuildingType;
use super::super::grid::Grid;
use super::super::grid_pos::GridPos;

fn grid_with_core(width: u32, height: u32) -> (Grid, GridPos) {
    let mut grid = Grid::new(width, height);
    let core_pos = GridPos::new(width as i32 / 2, height as i32 / 2);
    // Reveal the whole grid so placement in these tests is never blocked by fog of war.
    grid.reveal_around(core_pos, width + height);
    grid.place_building(core_pos, BuildingType::Core);
    (grid, core_pos)
}

#[test]
fn conduit_adjacent_to_core_is_powered() {
    let (mut grid, core_pos) = grid_with_core(6, 6);
    let conduit_pos = GridPos::new(core_pos.x + 1, core_pos.y);
    grid.place_building(conduit_pos, BuildingType::Conduit);
    grid.update_power_grid();
    assert!(
        grid.get(conduit_pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .powered
    );
}

#[test]
fn building_beyond_repeater_range_is_unpowered_without_a_node() {
    let (mut grid, core_pos) = grid_with_core(20, 4);
    // Lay a chain of conduits far past the repeater range with no PowerNode.
    let far_x = core_pos.x + grid.repeater_range as i32 + 3;
    for x in (core_pos.x + 1)..=far_x {
        grid.place_building(GridPos::new(x, core_pos.y), BuildingType::Conduit);
    }
    grid.update_power_grid();
    let far_pos = GridPos::new(far_x, core_pos.y);
    assert!(
        !grid
            .get(far_pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .powered
    );
}

#[test]
fn power_node_repeater_extends_range() {
    let (mut grid, core_pos) = grid_with_core(30, 4);
    let range = grid.repeater_range as i32;
    for x in (core_pos.x + 1)..core_pos.x + range {
        grid.place_building(GridPos::new(x, core_pos.y), BuildingType::Conduit);
    }
    let node_pos = GridPos::new(core_pos.x + range, core_pos.y);
    grid.place_building(node_pos, BuildingType::PowerNode);
    let far_x = node_pos.x + range;
    for x in (node_pos.x + 1)..=far_x {
        grid.place_building(GridPos::new(x, core_pos.y), BuildingType::Conduit);
    }
    grid.update_power_grid();
    let far_pos = GridPos::new(far_x, core_pos.y);
    assert!(
        grid.get(far_pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .powered
    );
}

#[test]
fn dust_stalled_building_does_not_transmit_power() {
    let (mut grid, core_pos) = grid_with_core(6, 6);
    let conduit_pos = GridPos::new(core_pos.x + 1, core_pos.y);
    let beyond_pos = GridPos::new(core_pos.x + 2, core_pos.y);
    grid.place_building(conduit_pos, BuildingType::Conduit);
    grid.place_building(beyond_pos, BuildingType::Conduit);
    grid.get_mut(conduit_pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 100.0;
    grid.update_power_grid();
    assert!(
        !grid
            .get(beyond_pos)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .powered
    );
}

#[test]
fn the_core_generates_and_a_connected_drill_consumes() {
    let (mut grid, core_pos) = grid_with_core(6, 6);
    let drill_pos = GridPos::new(core_pos.x + 1, core_pos.y);
    grid.place_building(drill_pos, BuildingType::Drill);
    grid.update_power_grid();
    // Core generates power, drill consumes it once connected.
    assert!(grid.total_power_generation() > 0.0);
    assert!(grid.total_power_consumption() > 0.0);
}
