use super::*;
use crate::data::GameConfig;

/// One full drone load at the shipped drill rate: 10 minerals at 5/s.
const FILL_SECONDS: f32 = 2.0;

/// A Core with a conduit run east and a drill at the far end of it.
fn state_with_run(length: i32) -> (PlanetState, GridPos, GridPos) {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 24);
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    state.unlock_building(BuildingType::Conduit);
    state.unlock_building(BuildingType::PowerNode);

    for step in 1..=length {
        let pos = GridPos::new(core.x + step, core.y);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        // A repeater every five tiles keeps a long run powered; power nodes
        // carry traffic too, so the route length is unaffected.
        let piece = if step % 5 == 0 {
            BuildingType::PowerNode
        } else {
            BuildingType::Conduit
        };
        state.select_building(piece);
        assert!(state.try_place_building(pos), "run piece at step {step}");
    }

    let drill = GridPos::new(core.x + length + 1, core.y);
    {
        // Ordinary ground under the drill. These tests are about routes
        // and dispatch; letting them start on whatever deposit the world
        // generator happened to lay there makes them measure geology.
        let tile = state.grid.get_mut(drill).unwrap();
        tile.terrain = crate::engine::TerrainType::Empty;
        tile.ore_richness = 1.0;
    }
    state.select_building(BuildingType::Drill);
    assert!(state.try_place_building(drill));
    state.grid.update_power_grid();

    (state, core, drill)
}

/// Loads delivered by a single drill over `seconds` of fixed-step
/// simulation, with the storage cap lifted out of the way.
fn loads_delivered(state: &mut PlanetState, seconds: f32) -> u32 {
    state.resources.minerals = 0.0;
    state.config.resources.base_mineral_cap = 100_000.0;

    let ticks = (seconds / crate::state::TICK_SECONDS) as u32;
    for _ in 0..ticks {
        state.step(crate::state::TICK_SECONDS, false);
    }

    (state.resources.minerals / state.drones.drone_capacity).round() as u32
}

/// Ore banked, rather than whole loads. Rounding to loads is too coarse to
/// see a difference that is a few seconds of drone speed.
fn ore_delivered(state: &mut PlanetState, seconds: f32) -> u32 {
    state.resources.minerals = 0.0;
    state.config.resources.base_mineral_cap = 100_000.0;

    let ticks = (seconds / crate::state::TICK_SECONDS) as u32;
    for _ in 0..ticks {
        state.step(crate::state::TICK_SECONDS, false);
    }
    state.resources.minerals.round() as u32
}

/// Snapshot of the whole harvest loop at the fixed timestep: drill cycle,
/// dispatch, travel out over the network, delivery, and the walk home. If
/// the tick length, drone speed, drill cycle or route cost changes, this
/// number moves.
#[test]
fn alloy_is_carried_to_whatever_eats_it_rather_than_always_to_the_core() {
    let (mut state, core, _drill) = state_with_run(4);
    // A Smelter on the run, and a Server Bank beside the Core that wants
    // what the Smelter makes.
    let smelter = GridPos::new(core.x + 2, core.y - 1);
    let bank = GridPos::new(core.x, core.y - 1);
    for (pos, kind) in [
        (smelter, BuildingType::Smelter),
        (bank, BuildingType::ServerBank),
    ] {
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        state.unlock_building(kind);
        state.select_building(kind);
        assert!(
            state.try_place_building(pos),
            "could not place at {:?}",
            pos
        );
    }
    state.grid.update_power_grid();

    let delivery = state
        .delivery_for(smelter, core, ResourceType::Alloy)
        .expect("somewhere for the alloy to go");
    assert_eq!(
        delivery.0, bank,
        "alloy went to {:?} rather than to the thing that eats it",
        delivery.0
    );
}

#[test]
fn a_resource_nothing_wants_still_goes_home_to_the_core() {
    let (state, core, drill) = state_with_run(4);
    // Biomass has no consumer building at all.
    let delivery = state
        .delivery_for(drill, core, ResourceType::Biomass)
        .expect("a route home");
    assert_eq!(delivery.0, core);
}

fn two_smelter_run() -> (PlanetState, GridPos, GridPos, GridPos, GridPos) {
    let (mut state, core, drill) = state_with_run(6);
    let near = GridPos::new(core.x + 2, core.y - 1);
    let far = GridPos::new(core.x + 4, core.y - 1);
    for pos in [near, far] {
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        assert!(state.grid.place_building(pos, BuildingType::Smelter));
    }
    state.grid.update_power_grid();
    (state, core, drill, near, far)
}

#[test]
fn the_leaner_processor_is_fed_before_the_nearer_one() {
    let (mut state, core, drill, near, far) = two_smelter_run();
    state.input_hoppers.insert(
        (near.x, near.y),
        [(ResourceType::Minerals, 12.0)].into_iter().collect(),
    );

    let delivery = state
        .delivery_for(drill, core, ResourceType::Minerals)
        .expect("processor delivery");
    assert_eq!(delivery.0, far);
}

#[test]
fn priority_processor_claims_cargo_before_a_leaner_standard_line() {
    let (mut state, core, drill, near, far) = two_smelter_run();
    state.input_hoppers.insert(
        (near.x, near.y),
        [(ResourceType::Minerals, 12.0)].into_iter().collect(),
    );
    state
        .grid
        .get_mut(near)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .input_priority = true;

    let delivery = state
        .delivery_for(drill, core, ResourceType::Minerals)
        .expect("processor delivery");
    assert_eq!(delivery.0, near);
    assert_ne!(delivery.0, far);
}

#[test]
fn cargo_in_flight_counts_as_hopper_supply_for_dispatch() {
    let (mut state, core, drill, near, far) = two_smelter_run();
    let id = state.drones.spawn_drone(drill);
    state.drones.get_drone_mut(id).unwrap().dispatch(
        near,
        vec![near],
        10.0,
        ResourceType::Minerals,
    );

    let delivery = state
        .delivery_for(drill, core, ResourceType::Minerals)
        .expect("processor delivery");
    assert_eq!(delivery.0, far);
}

#[test]
fn a_drill_on_a_deposit_cuts_more_than_one_on_ordinary_ground() {
    /// Ore banked in ten seconds by a drill on ground of this richness.
    fn banked(richness: f32) -> u32 {
        let (mut state, _core, drill) = state_with_run(0);
        state.grid.get_mut(drill).unwrap().ore_richness = richness;
        ore_delivered(&mut state, 10.0)
    }

    let ordinary = banked(1.0);
    assert!(banked(2.0) > ordinary, "a deposit was worth nothing");
    assert!(banked(0.5) < ordinary, "lean ground was worth the same");
}

#[test]
fn a_deposit_is_cut_down_towards_ordinary_ground_and_stops_there() {
    let (mut state, _core, drill) = state_with_run(0);
    state.grid.get_mut(drill).unwrap().ore_richness = 2.0;
    // A depletion rate that would run well past the floor if it could.
    state.config.ore.depletion_per_unit = 0.05;

    ore_delivered(&mut state, 120.0);

    let left = state.grid.get(drill).unwrap().ore_richness;
    assert_eq!(
        left, 1.0,
        "the deposit stopped somewhere other than ordinary ground"
    );
}

#[test]
fn ordinary_ground_is_not_worn_out_by_being_worked() {
    let (mut state, _core, drill) = state_with_run(0);
    state.grid.get_mut(drill).unwrap().ore_richness = 1.0;
    state.config.ore.depletion_per_unit = 0.05;

    ore_delivered(&mut state, 120.0);

    assert_eq!(state.grid.get(drill).unwrap().ore_richness, 1.0);
}

#[test]
fn a_pad_that_is_already_full_is_not_charged_for_ore_it_never_took() {
    let (mut state, _core, drill) = state_with_run(0);
    state.grid.get_mut(drill).unwrap().ore_richness = 2.0;
    state.config.ore.depletion_per_unit = 0.01;
    // Only the drills run, so nothing ever leaves the pad: it fills and
    // the drill has to stop being charged for ore it cannot cut.
    for _ in 0..600 {
        state.update_drills(crate::state::TICK_SECONDS);
    }

    let left = state.grid.get(drill).unwrap().ore_richness;
    let pad = state
        .output_buffers
        .get(&(drill.x, drill.y))
        .copied()
        .unwrap_or(0.0);
    let taken = (2.0 - left) / 0.01;
    assert!(
        (taken - pad).abs() < 0.5,
        "charged for {} ore with {} on the pad",
        taken,
        pad
    );
}

#[test]
fn partial_loads_lift_what_a_long_run_actually_delivers() {
    /// Ore banked over a fixed stretch, with partial dispatch on or off.
    fn delivered(allow_partial: bool) -> f32 {
        let (mut state, _, _) = state_with_run(10);
        state.research.unlocked_techs.push("swarm_dispatch".into());
        state.refresh_stats();
        // Room to bank the lot: at the shipped cap the pool clamps and
        // every arrangement looks identical.
        state.config.resources.base_mineral_cap = 1_000_000.0;
        if !allow_partial {
            // A threshold no run can reach is how "never" is spelled.
            state.config.buildings.partial_load_min_route = 10_000.0;
        }
        let before = state.resources.minerals;
        for _ in 0..1_800 {
            state.step(crate::state::TICK_SECONDS, false);
        }
        state.resources.minerals - before
    }

    let waiting = delivered(false);
    let sending = delivered(true);
    assert!(
        sending > waiting,
        "partial loads delivered {} against {} for waiting",
        sending,
        waiting
    );
}

#[test]
fn a_long_run_sends_a_part_full_drone_rather_than_leave_it_standing() {
    let (mut state, core, drill) = state_with_run(8);
    let load = state.drones.drone_capacity;
    // Two thirds of a load on the pad and a drone idle beside it.
    state.output_buffers.insert((drill.x, drill.y), load * 0.67);

    state.dispatch_producers(core);

    let carrying: f32 = state
        .drones
        .drones()
        .iter()
        .filter(|drone| drone.state != DroneState::Idle)
        .map(|drone| drone.carrying)
        .sum();
    assert!(
        carrying > 0.0 && carrying < load,
        "nothing left with a part load: {}",
        carrying
    );
    // And the pad was debited exactly what left, not a whole load.
    let left = state
        .output_buffers
        .get(&(drill.x, drill.y))
        .copied()
        .unwrap_or(0.0);
    assert!(left.abs() < 0.001, "the pad still holds {}", left);
}

#[test]
fn a_short_run_still_waits_for_a_full_load() {
    let (mut state, core, drill) = state_with_run(2);
    let load = state.drones.drone_capacity;
    state.output_buffers.insert((drill.x, drill.y), load * 0.67);

    state.dispatch_producers(core);

    assert!(
        state
            .drones
            .drones()
            .iter()
            .all(|drone| drone.state == DroneState::Idle),
        "a drone left on a run short enough to wait out"
    );
}

#[test]
fn a_pad_with_barely_anything_on_it_is_not_worth_the_walk() {
    let (mut state, core, drill) = state_with_run(8);
    let load = state.drones.drone_capacity;
    state.output_buffers.insert((drill.x, drill.y), load * 0.1);

    state.dispatch_producers(core);

    assert!(state
        .drones
        .drones()
        .iter()
        .all(|drone| drone.state == DroneState::Idle));
}

#[test]
fn a_full_load_goes_out_however_short_the_run() {
    let (mut state, core, drill) = state_with_run(2);
    let load = state.drones.drone_capacity;
    state.output_buffers.insert((drill.x, drill.y), load);

    state.dispatch_producers(core);

    assert!(state
        .drones
        .drones()
        .iter()
        .any(|drone| drone.carrying >= load));
}

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

#[test]
fn a_drill_with_no_pipe_to_the_core_never_dispatches() {
    let (mut state, _core, drill) = state_with_run(3);
    // Cut the run right next to the drill.
    state
        .grid
        .remove_building(GridPos::new(drill.x - 1, drill.y));
    state.grid.update_power_grid();

    state.update_logistics(FILL_SECONDS * 4.0, false);
    assert_eq!(state.drones.drones()[0].state, DroneState::Idle);
}

#[test]
fn cutting_the_run_stops_a_drone_in_flight_without_losing_its_cargo() {
    let (mut state, _core, drill) = state_with_run(4);
    state.update_logistics(FILL_SECONDS, false);
    assert_eq!(state.drones.drones()[0].state, DroneState::MovingToCore);

    state
        .grid
        .remove_building(GridPos::new(drill.x - 3, drill.y));
    state.grid.update_power_grid();
    state.update_logistics(0.0, false);

    let drone = &state.drones.drones()[0];
    assert_eq!(drone.state, DroneState::Error);
    assert!(drone.carrying > 0.0);
    assert_eq!(state.stalled_drone_count(), 1);
}

#[test]
fn a_cut_run_is_routed_around_when_a_second_run_exists() {
    let (mut state, core, drill) = state_with_run(4);
    // A parallel run one row south, joined at both ends.
    state.select_building(BuildingType::Conduit);
    for x in core.x..=drill.x {
        let pos = GridPos::new(x, core.y + 1);
        state.grid.get_mut(pos).unwrap().terrain = crate::engine::TerrainType::Empty;
        assert!(state.try_place_building(pos));
    }
    state.grid.update_power_grid();
    state.update_logistics(FILL_SECONDS, false);
    assert_eq!(state.drones.drones()[0].state, DroneState::MovingToCore);

    state
        .grid
        .remove_building(GridPos::new(drill.x - 2, drill.y));
    state.grid.update_power_grid();
    state.update_logistics(0.0, false);

    let drone = &state.drones.drones()[0];
    assert_eq!(drone.state, DroneState::MovingToCore);
    assert_eq!(drone.path.last(), Some(&core));
    assert_eq!(state.stalled_drone_count(), 0);
}

#[test]
fn repairing_the_run_sends_a_stalled_drone_on_its_way_again() {
    let (mut state, core, drill) = state_with_run(4);
    state.update_logistics(FILL_SECONDS, false);
    let cut = GridPos::new(drill.x - 3, drill.y);
    state.grid.remove_building(cut);
    state.grid.update_power_grid();
    state.update_logistics(0.0, false);
    assert_eq!(state.drones.drones()[0].state, DroneState::Error);

    state.select_building(BuildingType::Conduit);
    assert!(state.try_place_building(cut));
    state.update_logistics(0.0, false);

    let drone = &state.drones.drones()[0];
    assert_eq!(drone.state, DroneState::MovingToCore);
    assert_eq!(drone.path.last(), Some(&core));
    assert!(drone.carrying > 0.0);
}

#[test]
fn drones_stranded_by_a_power_collapse_recover_once_it_passes() {
    let (mut state, _core, _drill) = state_with_run(3);
    state.update_logistics(FILL_SECONDS, false);
    state.trigger_power_collapse();
    assert_eq!(state.drones.drones()[0].state, DroneState::Error);
    assert_eq!(state.drones.drones()[0].carrying, 0.0);

    // The collapse blocks the sim; once it lifts, drones head home.
    state.power_collapse_shutdown = 0.0;
    for _ in 0..40 {
        state.update_logistics(0.1, false);
        state.drones.update(0.1);
    }
    assert_ne!(state.drones.drones()[0].state, DroneState::Error);
}

#[test]
fn a_longer_run_takes_a_longer_route_than_a_direct_one() {
    let (mut short_run, _, _) = state_with_run(1);
    let (mut long_run, _, _) = state_with_run(6);
    short_run.update_logistics(FILL_SECONDS, false);
    long_run.update_logistics(FILL_SECONDS, false);

    assert!(long_run.drones.drones()[0].path.len() > short_run.drones.drones()[0].path.len());
}
