use std::time::Instant;

use crate::data::GameConfig;
use crate::engine::{route_over_network, BuildingType, DroneState, GridPos, ResourceType};
use crate::state::{Campaign, PlanetState, TICK_SECONDS};

fn congested_fixture() -> PlanetState {
    let mut state = PlanetState::new(2, 99, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 30);
    for x in 1..=10 {
        let pos = GridPos::new(core.x + x, core.y);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.grid.place_building(pos, BuildingType::Conduit);
    }
    let target = GridPos::new(core.x + 10, core.y);
    let route = route_over_network(&state.grid, core, target).expect("fixture route");
    for _ in 0..256 {
        let id = state.drones.spawn_drone(core);
        state.drones.get_drone_mut(id).unwrap().dispatch(
            target,
            route.clone(),
            1.0,
            ResourceType::Minerals,
        );
    }
    state.grid.update_power_grid();
    state
}

#[test]
fn hundreds_of_drones_on_a_congested_network_stay_under_budget() {
    let mut state = congested_fixture();
    let start = Instant::now();
    for _ in 0..120 {
        state.step(TICK_SECONDS, false);
    }
    assert!(
        start.elapsed().as_secs_f32() < 5.0,
        "congested 256-drone fixture exceeded 5 seconds"
    );
    assert_eq!(state.drones.total_count(), 256);
    assert!(state
        .drones
        .drones()
        .iter()
        .all(|drone| drone.state != DroneState::Error));
}

#[test]
fn background_world_transitions_stay_under_budget() {
    let mut campaign = Campaign::new(GameConfig::default(), 123);
    for index in [0, 1, 3, 4] {
        assert!(campaign.colonize(index));
    }
    let start = Instant::now();
    for _ in 0..30 {
        campaign.update_background(1.0);
    }
    campaign.update_shipments(30.0);
    assert!(
        start.elapsed().as_secs_f32() < 5.0,
        "all-world background fixture exceeded 5 seconds"
    );
    assert_eq!(campaign.colonized_flags(), [true, true, true, true, true]);
}

#[test]
fn largest_map_route_stays_under_budget() {
    let mut state = PlanetState::new(4, 101, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 30);
    let target = GridPos::new(state.grid.width as i32 - 2, core.y);
    for x in (core.x + 1)..target.x {
        let pos = GridPos::new(x, core.y);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.grid.place_building(pos, BuildingType::Conduit);
    }
    state.grid.update_power_grid();

    let start = Instant::now();
    for _ in 0..500 {
        route_over_network(&state.grid, core, target).expect("largest-map fixture route");
    }
    assert!(
        start.elapsed().as_secs_f32() < 5.0,
        "largest-map routing fixture exceeded 5 seconds"
    );
}
