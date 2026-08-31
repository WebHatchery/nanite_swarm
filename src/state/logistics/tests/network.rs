//! Network integrity, throughput, and congestion checks.
use super::*;

mod recovery;

#[test]
fn an_unbroken_run_has_nothing_to_point_at() {
    let (state, _, _) = state_with_run(4);
    assert!(state.severed_network().is_empty());
}

#[test]
fn cutting_a_run_points_at_everything_past_the_cut() {
    let (mut state, core, _) = state_with_run(5);
    let cut = GridPos::new(core.x + 3, core.y);
    state.grid.remove_building(cut);
    state.grid.update_power_grid();

    let severed = state.severed_network();
    // The two pieces beyond the cut, and nothing on the Core's side.
    assert!(severed.contains(&GridPos::new(core.x + 4, core.y)));
    assert!(severed.contains(&GridPos::new(core.x + 5, core.y)));
    assert!(!severed.contains(&GridPos::new(core.x + 1, core.y)));
    assert!(!severed.contains(&GridPos::new(core.x + 2, core.y)));
    assert!(!severed.contains(&core), "the Core cut itself off");
}

#[test]
fn a_conduit_choked_with_dust_is_named_as_the_break_itself() {
    let (mut state, core, _) = state_with_run(4);
    let choked = GridPos::new(core.x + 2, core.y);
    if let Some(building) = state
        .grid
        .get_mut(choked)
        .and_then(|tile| tile.building.as_mut())
    {
        building.dust = 100.0;
    }
    assert!(state
        .grid
        .get(choked)
        .unwrap()
        .building
        .as_ref()
        .unwrap()
        .is_dust_stalled());

    let severed = state.severed_network();
    assert!(
        severed.contains(&choked),
        "the choked tile was not pointed at: {:?}",
        severed
    );
    assert!(severed.contains(&GridPos::new(core.x + 3, core.y)));
    assert!(!severed.contains(&GridPos::new(core.x + 1, core.y)));
}

#[test]
fn a_second_run_takes_the_load_the_first_one_cannot() {
    // A short run and a longer parallel one, both reaching the drill.
    let (mut state, core, drill) = state_with_run(4);
    let detour: Vec<GridPos> = (0..=5)
        .map(|step| GridPos::new(core.x + step, core.y + 1))
        .collect();
    for pos in &detour {
        state.grid.get_mut(*pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.select_building(BuildingType::Conduit);
        assert!(state.try_place_building(*pos), "detour piece at {:?}", pos);
    }
    state.grid.update_power_grid();

    // With everything clear, the drill routes down the short run.
    let clear = state
        .delivery_for(drill, core, ResourceType::Minerals)
        .expect("a route home");
    assert!(
        clear.1.iter().all(|pos| pos.y == core.y),
        "the clear route left the short run: {:?}",
        clear.1
    );

    // Now saturate it. The same drill should prefer the longer way round.
    for step in 1..=4 {
        state.traffic.insert((core.x + step, core.y), 6);
    }
    let crowded = state
        .delivery_for(drill, core, ResourceType::Minerals)
        .expect("a route home");
    assert!(
        crowded.1.iter().any(|pos| pos.y == core.y + 1),
        "the crowded route stayed on the saturated run: {:?}",
        crowded.1
    );
    assert!(
        crowded.1.len() > clear.1.len(),
        "the detour was not actually longer"
    );
}

#[test]
fn a_drill_beside_the_core_delivers_four_loads_in_ten_seconds() {
    let (mut state, _core, _drill) = state_with_run(0);
    assert_eq!(loads_delivered(&mut state, 10.0), 4);
}

/// The point of the pillar: the same drill on the end of a long run
/// delivers less, because the round trip now costs more than the cycle.
#[test]
fn a_drill_at_the_end_of_a_long_run_delivers_less() {
    let (mut near, _, _) = state_with_run(1);
    let (mut far, _, _) = state_with_run(9);
    assert!(loads_delivered(&mut far, 30.0) < loads_delivered(&mut near, 30.0));
}

/// Put `count` drones on the same tile, mid-route, and let the traffic
/// pass settle.
fn crowd_one_tile(state: &mut PlanetState, count: usize, tile: GridPos) {
    for _ in 0..count {
        let id = state.drones.spawn_drone(tile);
        let drone = state.drones.get_drone_mut(id).unwrap();
        drone.dispatch(tile, vec![tile], 1.0, ResourceType::Minerals);
    }
    state.update_traffic();
}

#[test]
fn a_drill_works_alone_until_research_says_otherwise() {
    let (mut state, _core, drill) = state_with_run(2);
    assert_eq!(state.drone_crew_size(), 1);
    state.step(FILL_SECONDS, false);
    assert_eq!(state.drones.drones_at(drill).len(), 1);
}

fn drill_output_after_research(researched: bool) -> f32 {
    let (mut state, _, drill) = state_with_run(0);
    if researched {
        state
            .research
            .unlocked_techs
            .push("self_cleaning_servos".to_string());
        state.refresh_stats();
    }
    state
        .grid
        .get_mut(drill)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 26.0;
    state.update_drills(1.0);
    state
        .output_buffers
        .get(&(drill.x, drill.y))
        .copied()
        .unwrap_or(0.0)
}

#[test]
fn self_cleaning_servos_moves_the_live_drill_efficiency_threshold() {
    let plain = drill_output_after_research(false);
    let researched = drill_output_after_research(true);

    assert_eq!(plain, 4.5, "the default threshold should reduce the drill");
    assert_eq!(
        researched, 5.0,
        "the researched threshold should keep this drill at full output"
    );
}

#[test]
fn self_cleaning_servos_moves_the_live_drone_speed_threshold() {
    let (mut plain, _, drill) = state_with_run(2);
    let (mut researched, _, researched_drill) = state_with_run(2);
    researched
        .research
        .unlocked_techs
        .push("self_cleaning_servos".to_string());
    researched.refresh_stats();

    for (state, drill_pos) in [(&mut plain, drill), (&mut researched, researched_drill)] {
        state
            .grid
            .get_mut(drill_pos)
            .unwrap()
            .building
            .as_mut()
            .unwrap()
            .dust = 51.0;
        state.update_logistics(FILL_SECONDS, false);
    }

    assert_eq!(plain.drones.drones()[0].speed, 5.6);
    assert_eq!(researched.drones.drones()[0].speed, 8.0);
}

#[test]
fn acid_neutralizers_keep_a_high_dust_run_connected() {
    let (mut state, core, drill) = state_with_run(4);
    state
        .research
        .unlocked_techs
        .push("acid_neutralizers".to_string());
    state.refresh_stats();

    let choked = GridPos::new(core.x + 2, core.y);
    state
        .grid
        .get_mut(choked)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 105.0;
    state.refresh_power_grid();

    assert!(state.resolved_dust_response().stall_threshold > 105.0);
    assert!(
        state
            .grid
            .get(choked)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .powered
    );
    assert!(state
        .delivery_for(drill, core, ResourceType::Minerals)
        .is_some());
}

#[test]
fn swarm_dispatch_puts_a_second_drone_on_every_drill() {
    let (mut state, _core, drill) = state_with_run(2);
    state
        .research
        .unlocked_techs
        .push("swarm_dispatch".to_string());
    state.refresh_stats();
    assert_eq!(state.drone_crew_size(), 2);

    // The crew turns up without the drill being rebuilt.
    state.step(FILL_SECONDS, false);
    assert_eq!(state.drones.drones_at(drill).len(), 2);
}

#[test]
fn a_second_drone_lifts_throughput_on_a_run_too_long_for_one() {
    let alone = {
        let (mut state, _, _) = state_with_run(9);
        loads_delivered(&mut state, 60.0)
    };
    let crewed = {
        let (mut state, _, _) = state_with_run(9);
        state
            .research
            .unlocked_techs
            .push("swarm_dispatch".to_string());
        state.refresh_stats();
        loads_delivered(&mut state, 60.0)
    };
    assert!(
        crewed > alone,
        "one drone delivered {alone}, two delivered {crewed}"
    );
}

#[test]
fn traffic_below_capacity_costs_nothing() {
    let (mut state, core, _) = state_with_run(2);
    state.drones.drones_mut().iter_mut().for_each(|drone| {
        drone.state = DroneState::Idle;
    });
    crowd_one_tile(&mut state, 2, core);

    assert_eq!(state.congested_tiles(), 0);
    assert!(!state.is_congested(core));
    for drone in state.drones.drones() {
        if drone.state == DroneState::MovingToCore {
            assert_eq!(drone.speed, state.drones.drone_speed);
        }
    }
}

#[test]
fn a_crowded_tile_slows_everything_crossing_it() {
    let (mut state, core, _) = state_with_run(2);
    state.drones.drones_mut().iter_mut().for_each(|drone| {
        drone.state = DroneState::Idle;
    });
    let capacity = state.config.buildings.conduit_capacity;
    crowd_one_tile(&mut state, 6, core);

    assert!(state.is_congested(core));
    assert_eq!(state.congested_tiles(), 1);

    let expected = state.drones.drone_speed * (capacity / 6.0);
    for drone in state.drones.drones() {
        if drone.state == DroneState::MovingToCore {
            assert!(
                (drone.speed - expected).abs() < 1e-3,
                "{} vs {expected}",
                drone.speed
            );
        }
    }
}

#[test]
fn traffic_only_counts_drones_that_are_actually_moving() {
    let (mut state, core, _) = state_with_run(2);
    state.drones.drones_mut().iter_mut().for_each(|drone| {
        drone.state = DroneState::Idle;
    });
    crowd_one_tile(&mut state, 6, core);
    assert!(state.is_congested(core));

    // Park them all: the jam clears.
    for drone in state.drones.drones_mut() {
        drone.state = DroneState::Idle;
    }
    state.update_traffic();
    assert_eq!(state.congested_tiles(), 0);
    assert_eq!(state.drones.drones()[0].speed, state.drones.drone_speed);
}

/// Two drills sharing one run, measured over a minute at a given tile
/// capacity.
fn shared_trunk_throughput(capacity: f32) -> u32 {
    let (mut state, _core, drill) = state_with_run(8);
    state.config.buildings.conduit_capacity = capacity;
    // The spur drill gets ordinary ground too, so this measures the
    // trunk and not which drill landed on a deposit.
    if let Some(tile) = state.grid.get_mut(GridPos::new(drill.x - 1, drill.y - 1)) {
        tile.ore_richness = 1.0;
    }
    // A second drill hanging off the same run.
    let spur = GridPos::new(drill.x - 1, drill.y - 1);
    state.grid.get_mut(spur).unwrap().terrain = crate::engine::TerrainType::Empty;
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(spur));
    state.grid.update_power_grid();
    ore_delivered(&mut state, 60.0)
}

#[test]
fn a_saturated_trunk_delivers_less_than_a_clear_one() {
    let clear = shared_trunk_throughput(100.0);
    let saturated = shared_trunk_throughput(0.5);
    assert!(
        saturated < clear,
        "saturated run delivered {saturated}, clear run delivered {clear}"
    );
}

#[test]
fn drills_dispatch_along_the_conduit_run() {
    let (mut state, core, drill) = state_with_run(3);
    state.update_logistics(FILL_SECONDS, false);

    let drone = &state.drones.drones()[0];
    assert_eq!(drone.state, DroneState::MovingToCore);
    assert_eq!(drone.path.len(), 4);
    assert_eq!(drone.path.last(), Some(&core));
    assert_eq!(drone.home, drill);
}

fn drone_speed_on_ring_shadow(on_shadow: bool) -> f32 {
    let mut state = PlanetState::new(4, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 12);
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    state.unlock_building(BuildingType::Conduit);
    state.features[0].bounds = if on_shadow {
        (0.40, 0.40, 0.30, 0.30)
    } else {
        (0.0, 0.0, 0.05, 0.05)
    };

    for step in 1..=4 {
        let pos = GridPos::new(core.x + step, core.y);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.select_building(BuildingType::Conduit);
        assert!(state.try_place_building(pos));
    }
    let drill = GridPos::new(core.x + 5, core.y);
    let drill_tile = state.grid.get_mut(drill).unwrap();
    drill_tile.terrain = crate::engine::TerrainType::Empty;
    drill_tile.ore_richness = 1.0;
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));
    state.grid.update_power_grid();

    state.update_logistics(FILL_SECONDS, false);
    let drone = &state.drones.drones()[0];
    assert_eq!(drone.state, DroneState::MovingToCore);
    assert!(!drone.path.is_empty());
    let local_multiplier = crate::state::game_state::feature_hazard_multiplier(
        &state.features,
        &state.data().planet(4).features,
        drone.path[drone.path_index],
        state.grid.width,
        state.grid.height,
        "cold",
    );
    assert_eq!(local_multiplier, if on_shadow { 1.4 } else { 1.0 });
    drone.speed
}

#[test]
fn ring_shadow_deepens_cold_on_its_route_without_affecting_ordinary_route_speed() {
    let ordinary = drone_speed_on_ring_shadow(false);
    let shadow = drone_speed_on_ring_shadow(true);
    let freeze = PlanetState::new(4, 42, GameConfig::default()).freeze_strength();
    let expected_ratio = (1.0 - freeze * 1.4) / (1.0 - freeze);

    assert!(
        shadow < ordinary,
        "Ring Shadow speed was {shadow}, ordinary route speed was {ordinary}"
    );
    assert!(
        (shadow - ordinary * expected_ratio).abs() < 0.001,
        "Ring Shadow speed was {shadow}, expected {expected_ratio}x ordinary speed {ordinary}"
    );
}
