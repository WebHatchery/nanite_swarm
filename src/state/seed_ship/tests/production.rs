use super::*;

/// A drill beside the Core and a Smelter on its other side, both powered.
fn state_with_smelter() -> (PlanetState, GridPos, GridPos) {
    let mut state = state();
    state.resources.alloy = 0.0;
    let core = state.grid.find_core().unwrap();
    let drill = GridPos::new(core.x + 1, core.y);
    let smelter = GridPos::new(core.x, core.y + 1);
    for pos in [drill, smelter] {
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.grid.reveal_around(pos, 1);
    }
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(smelter));
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));
    state.grid.update_power_grid();
    (state, drill, smelter)
}

/// The whole chain in one go: ore is carried to a Smelter, refined into
/// alloy, and the alloy is carried on to the Core by the Smelter's own
/// drones. Nothing teleports at either end.
#[test]
fn ore_is_carried_in_refined_and_the_alloy_carried_out() {
    let (mut state, _drill, smelter) = state_with_smelter();

    // Long enough for a full load of alloy to be made and then delivered.
    for _ in 0..400 {
        state.step(0.1, false);
    }

    assert!(state.resources.alloy > 0.0, "no alloy reached the Core");
    assert!(state.alloy_rate() > 0.0);
    // Ore reached the hopper rather than the global pool...
    assert!(state.input_buffers.contains_key(&(smelter.x, smelter.y)));
    // ...and the Smelter has its own crew to carry the alloy out.
    assert!(!state.drones.drones_at(smelter).is_empty());
}

#[test]
fn alloy_waits_on_the_smelter_pad_until_a_drone_takes_it() {
    let (mut state, _drill, smelter) = state_with_smelter();

    // Short enough that a full load has not been collected yet.
    for _ in 0..100 {
        state.step(0.1, false);
    }

    let on_pad = state
        .output_buffers
        .get(&(smelter.x, smelter.y))
        .copied()
        .unwrap_or(0.0);
    assert!(on_pad > 0.0, "the Smelter refined nothing");
    assert_eq!(
        state.resources.alloy, 0.0,
        "alloy reached the pool without being carried"
    );
}

#[test]
fn a_smelter_nobody_delivers_to_stays_idle_however_full_the_pool_is() {
    let mut state = state();
    state.resources.alloy = 0.0;
    state.resources.minerals = 100_000.0;
    // Powered, but with no drill anywhere to send it anything.
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x, core.y + 1);
    state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(pos));
    state.grid.update_power_grid();

    for _ in 0..100 {
        state.step(0.1, false);
    }
    assert_eq!(state.resources.alloy, 0.0);
}

#[test]
fn ore_goes_to_the_smelter_before_it_goes_to_the_pool() {
    let (mut state, _drill, smelter) = state_with_smelter();
    state.resources.minerals = 0.0;
    state.config.resources.base_mineral_cap = 100_000.0;

    // One delivery's worth of time.
    for _ in 0..40 {
        state.step(0.1, false);
    }

    let refined = state
        .output_buffers
        .get(&(smelter.x, smelter.y))
        .copied()
        .unwrap_or(0.0);
    assert!(refined > 0.0, "nothing reached the smelter");
    assert_eq!(
        state.resources.minerals, 0.0,
        "ore went to the pool while the smelter had room"
    );
}

#[test]
fn an_unpowered_smelter_refines_nothing() {
    let mut state = state();
    state.resources.alloy = 0.0;
    // Far from the Core, so it never gets power.
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 6, core.y + 6);
    state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(pos));
    state.grid.update_power_grid();

    for _ in 0..100 {
        state.step(0.1, false);
    }
    assert_eq!(state.resources.alloy, 0.0);
    assert_eq!(state.alloy_rate(), 0.0);
}

#[test]
fn a_smelter_with_no_minerals_left_simply_slows_down() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::Smelter);
    state.select_building(BuildingType::Smelter);
    assert!(state.try_place_building(pos));
    state.grid.update_power_grid();

    state.resources.minerals = 0.0;
    state.resources.alloy = 0.0;
    for _ in 0..100 {
        state.step(0.1, false);
    }

    assert_eq!(state.resources.alloy, 0.0);
    assert!(state.resources.minerals >= 0.0, "minerals went negative");
}
