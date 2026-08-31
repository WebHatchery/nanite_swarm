//! Delivery and processor-priority logistics checks.
use super::*;

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
fn an_iron_belt_drill_cuts_the_authored_local_bonus() {
    fn cut_on_belt(on_belt: bool) -> f32 {
        let mut state = PlanetState::new(0, 42, GameConfig::default());
        let core = state.grid.find_core().expect("Mercury Core");
        let drill = GridPos::new(core.x + 1, core.y);
        if let Some(tile) = state.grid.get_mut(drill) {
            tile.terrain = crate::engine::TerrainType::Empty;
            tile.ore_richness = 1.0;
            tile.revealed = true;
        }
        state.features[0].bounds = if on_belt {
            (0.45, 0.45, 0.20, 0.20)
        } else {
            (0.0, 0.0, 0.05, 0.05)
        };
        state.select_building(BuildingType::Drill);
        assert!(state.try_place_building(drill));
        state.grid.update_power_grid();
        state.update_drills(1.0);
        state
            .output_buffers
            .get(&(drill.x, drill.y))
            .copied()
            .unwrap_or(0.0)
    }

    let ordinary = cut_on_belt(false);
    let belt = cut_on_belt(true);
    assert!((belt - ordinary * 1.5).abs() < 0.001);
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
