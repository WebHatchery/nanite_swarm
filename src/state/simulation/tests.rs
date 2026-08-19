use super::*;
use crate::data::GameConfig;
use crate::engine::GridPos;

fn state() -> PlanetState {
    PlanetState::new(2, 42, GameConfig::default())
}

/// A world with `count` conduits standing beside the Core, for the
/// collapse to have something to be the size of.
fn state_with_structures(count: i32) -> PlanetState {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    state.grid.reveal_around(core, 32);
    for step in 1..=count {
        let pos = GridPos::new(core.x + step % 8, core.y + 1 + step / 8);
        if let Some(tile) = state.grid.get_mut(pos) {
            tile.terrain = TerrainType::Empty;
        }
        state.grid.place_building(pos, BuildingType::Conduit);
    }
    state
}

#[test]
fn a_small_swarm_is_knocked_down_for_less_time_than_a_large_one() {
    let mut small = state_with_structures(2);
    let mut large = state_with_structures(80);
    assert!(small.collapse_scale() < large.collapse_scale());

    small.trigger_power_collapse();
    large.trigger_power_collapse();

    assert!(
        small.power_collapse_shutdown < large.power_collapse_shutdown,
        "small was down {}s and large {}s",
        small.power_collapse_shutdown,
        large.power_collapse_shutdown
    );
    assert!(small.research_lock_timer < large.research_lock_timer);
}

#[test]
fn the_shutdown_stays_between_the_two_ends_it_was_given() {
    let config = GameConfig::default();
    let (min, max) = (
        config.collapse.min_shutdown_seconds,
        config.collapse.max_shutdown_seconds,
    );
    for count in [0, 1, 30, 80, 400] {
        let mut state = state_with_structures(count);
        state.trigger_power_collapse();
        let down = state.power_collapse_shutdown;
        assert!(
            (min..=max).contains(&down),
            "{} structures went down for {}s",
            count,
            down
        );
        assert_eq!(state.power_collapse_length, down);
    }
}

#[test]
fn a_bigger_swarm_loses_a_bigger_share_of_what_it_was_holding() {
    let mut small = state_with_structures(2);
    let mut large = state_with_structures(80);
    small.resources.data = 1_000.0;
    large.resources.data = 1_000.0;

    small.trigger_power_collapse();
    large.trigger_power_collapse();

    assert!(
        small.resources.data > large.resources.data,
        "small kept {} and large kept {}",
        small.resources.data,
        large.resources.data
    );
    // And neither is wiped out: a collapse is a setback, never a death.
    assert!(large.resources.data > 0.0);
}

#[test]
fn the_throughput_graph_starts_with_nothing_to_say() {
    let state = state();
    assert!(state.throughput.buckets().is_empty());
    assert_eq!(state.throughput.last(), None);
}

#[test]
fn a_second_of_deliveries_becomes_one_point_on_the_graph() {
    let mut state = state();
    state.delivered_since_sample = 12.0;
    state.sample_throughput(1.0);

    assert_eq!(state.throughput.len(), 1);
    assert_eq!(state.throughput.last(), Some(12.0));
    // And the accumulator starts over rather than double-counting.
    assert_eq!(state.delivered_since_sample, 0.0);
}

#[test]
fn a_quiet_second_is_recorded_as_a_quiet_second_not_skipped() {
    let mut state = state();
    state.delivered_since_sample = 6.0;
    state.sample_throughput(1.0);
    state.sample_throughput(1.0);

    assert_eq!(state.throughput.len(), 2);
    assert_eq!(state.throughput.last(), Some(0.0));
    // The spike is still the peak, which is the point of keeping ranges.
    assert_eq!(state.throughput.max(), Some(6.0));
}

#[test]
fn a_long_stretch_of_world_time_lands_every_second_it_covers() {
    let mut state = state();
    state.delivered_since_sample = 5.0;
    // Four seconds in one go: one second of deliveries and three quiet.
    state.sample_throughput(4.0);
    assert_eq!(state.throughput.len(), 4);
    assert_eq!(state.throughput.max(), Some(5.0));
}

/// A powered building of `kind` beside the Core, with `hopper` of its
/// carried input already delivered to it.
fn processing_world(kind: BuildingType, hopper: f32) -> (PlanetState, GridPos) {
    let mut state = state();
    state.resources.minerals = 10_000.0;
    state.resources.energy = 10_000.0;
    state.config.resources.max_energy = 10_000.0;
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 2);
    if let Some(tile) = state.grid.get_mut(pos) {
        tile.terrain = TerrainType::Empty;
    }
    state.unlock_building(kind);
    state.select_building(kind);
    assert!(state.try_place_building(pos), "could not place {:?}", kind);
    state.grid.update_power_grid();
    state.input_buffers.insert((pos.x, pos.y), hopper);
    (state, pos)
}

#[test]
fn a_recipe_takes_its_carried_input_from_the_hopper_and_pays_out_on_the_pad() {
    let (mut state, pos) = processing_world(BuildingType::Smelter, 60.0);

    state.update_recipes(1.0);

    let hopper = state.input_buffers.get(&(pos.x, pos.y)).copied().unwrap();
    let pad = state.output_buffers.get(&(pos.x, pos.y)).copied().unwrap();
    assert!(hopper < 60.0, "the hopper was not drawn down");
    assert!(pad > 0.0, "nothing was made");
    // Alloy is something a drone can carry, so it waits to be collected
    // rather than appearing in the pool.
    assert_eq!(state.resources.alloy, 0.0);
}

#[test]
fn a_recipe_with_an_empty_hopper_does_nothing_however_full_the_pool_is() {
    let (mut state, pos) = processing_world(BuildingType::Smelter, 0.0);
    state.resources.minerals = 100_000.0;

    state.update_recipes(1.0);

    assert_eq!(
        state
            .output_buffers
            .get(&(pos.x, pos.y))
            .copied()
            .unwrap_or(0.0),
        0.0
    );
}

#[test]
fn the_assembler_needs_both_routed_inputs_before_it_makes_components() {
    let (mut state, pos) = processing_world(BuildingType::Assembler, 10.0);
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [(crate::engine::ResourceType::Alloy, 10.0)]
            .into_iter()
            .collect(),
    );

    state.update_recipes(1.0);
    assert_eq!(
        state
            .output_buffers
            .get(&(pos.x, pos.y))
            .copied()
            .unwrap_or(0.0),
        0.0,
        "alloy alone should not satisfy the ore hopper"
    );

    state
        .input_hoppers
        .get_mut(&(pos.x, pos.y))
        .unwrap()
        .insert(crate::engine::ResourceType::Minerals, 10.0);
    state.update_recipes(1.0);

    assert_eq!(
        state
            .output_buffers
            .get(&(pos.x, pos.y))
            .copied()
            .unwrap_or(0.0),
        0.5,
        "one second should produce the declared component rate"
    );
    assert_eq!(
        state.resources.components, 0.0,
        "components must wait for a drone"
    );
}

#[test]
fn graph_samples_measure_factory_work_that_really_happened() {
    let (mut state, pos) = processing_world(BuildingType::Assembler, 0.0);
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [
            (crate::engine::ResourceType::Minerals, 10.0),
            (crate::engine::ResourceType::Alloy, 10.0),
        ]
        .into_iter()
        .collect(),
    );
    state.update_recipes(1.0);
    state.sample_throughput(1.0);

    let sample = state.graph_samples.last().expect("one observed second");
    assert!((sample.minerals_consumed - 2.0).abs() < 0.001);
    assert!((sample.alloy_consumed - 1.0).abs() < 0.001);
    assert!((sample.components_produced - 0.5).abs() < 0.001);
    assert_eq!(state.observed_components_rate(), 0.5);
}

#[test]
fn overclocked_assembler_pays_for_half_again_as_much_real_work() {
    let (mut state, pos) = processing_world(BuildingType::Assembler, 0.0);
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [
            (crate::engine::ResourceType::Minerals, 20.0),
            (crate::engine::ResourceType::Alloy, 20.0),
        ]
        .into_iter()
        .collect(),
    );
    state
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .overclocked = true;

    state.update_recipes(1.0);

    assert_eq!(state.output_buffers.get(&(pos.x, pos.y)), Some(&0.75));
    let hoppers = state.input_hoppers.get(&(pos.x, pos.y)).unwrap();
    assert_eq!(
        hoppers.get(&crate::engine::ResourceType::Minerals),
        Some(&17.0)
    );
    assert_eq!(
        hoppers.get(&crate::engine::ResourceType::Alloy),
        Some(&18.5)
    );
}

#[test]
fn a_full_processor_pad_stops_output_without_eating_more_input() {
    let (mut state, pos) = processing_world(BuildingType::Smelter, 0.0);
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [(crate::engine::ResourceType::Minerals, 20.0)]
            .into_iter()
            .collect(),
    );
    let capacity = state.processor_pad_capacity();
    state.output_buffers.insert((pos.x, pos.y), capacity);

    state.update_recipes(1.0);

    assert_eq!(state.output_buffers.get(&(pos.x, pos.y)), Some(&capacity));
    assert_eq!(
        state.input_hoppers[&(pos.x, pos.y)][&crate::engine::ResourceType::Minerals],
        20.0
    );
    assert_eq!(state.blocked_factories(), [pos]);
}

#[test]
fn starvation_marks_only_powered_processors_with_a_missing_input() {
    let (mut state, pos) = processing_world(BuildingType::Smelter, 0.0);
    assert_eq!(state.starved_factories(), [pos]);

    state.input_buffers.insert((pos.x, pos.y), 10.0);
    assert!(state.starved_factories().is_empty());

    state
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .powered = false;
    state.input_buffers.insert((pos.x, pos.y), 0.0);
    assert!(
        state.starved_factories().is_empty(),
        "an offline machine needs power, not an input warning"
    );
}

#[test]
fn physical_delivery_keeps_its_resource_identity_at_the_core() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let id = state.drones.spawn_drone(core);
    let drone = state.drones.get_drone_mut(id).unwrap();
    drone.dispatch(core, vec![core], 3.0, crate::engine::ResourceType::Biomass);
    drone.progress = 1.0;
    let before_minerals = state.resources.minerals;
    let before_biomass = state.resources.biomass;

    state.step(TICK_SECONDS, false);

    assert_eq!(state.resources.minerals, before_minerals);
    assert_eq!(state.resources.biomass, before_biomass + 3.0);
}

#[test]
fn an_output_nothing_can_carry_goes_straight_into_the_pool() {
    // A Server Bank turns carried alloy into Data, and nothing carries Data.
    let (mut state, pos) = processing_world(BuildingType::ServerBank, 10.0);
    state.resources.data = 0.0;

    state.update_recipes(1.0);

    assert!(state.resources.data > 0.0, "no Data was thought up");
    assert_eq!(
        state
            .output_buffers
            .get(&(pos.x, pos.y))
            .copied()
            .unwrap_or(0.0),
        0.0,
        "Data was left on the pad for a drone"
    );
    assert!(state.input_buffers.get(&(pos.x, pos.y)).copied().unwrap() < 10.0);
}

#[test]
fn research_that_speeds_up_thinking_speeds_up_a_data_recipe_too() {
    fn made(with_research: bool) -> f32 {
        let (mut state, _) = processing_world(BuildingType::ServerBank, 10.0);
        state.resources.data = 0.0;
        if with_research {
            state
                .research
                .unlocked_techs
                .push("advanced_research".to_string());
        }
        state.refresh_stats();
        state.update_recipes(1.0);
        state.resources.data
    }

    assert!(made(true) > made(false));
}

#[test]
fn the_dust_rate_is_data_rather_than_a_constant() {
    let mut slow = GameConfig::default();
    slow.upkeep.dust_rate = 0.05;
    let mut fast = GameConfig::default();
    fast.upkeep.dust_rate = 0.5;

    let settled = |config: GameConfig| {
        let mut state = PlanetState::new(2, 42, config);
        let core = state.grid.find_core().unwrap();
        for _ in 0..300 {
            state.step(TICK_SECONDS, false);
        }
        state
            .grid
            .get(core)
            .and_then(|tile| tile.building.as_ref())
            .map(|building| building.dust)
            .unwrap_or(0.0)
    };

    assert!(
        settled(fast) > settled(slow),
        "a bigger declared dust rate should settle more dust"
    );
}

#[test]
fn a_sweepers_reach_is_the_one_the_config_declares() {
    let mut config = GameConfig::default();
    config.upkeep.sweeper_radius = 9;
    let state = PlanetState::new(2, 42, config);
    assert_eq!(state.coverage_radius(BuildingType::Sweeper), Some(9));
    assert_eq!(
        state.coverage_radius(BuildingType::ShieldGenerator),
        Some(state.config.upkeep.hazard_counter_radius)
    );
}

#[test]
fn dust_accumulates_on_powered_buildings_over_time() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    state.update_dust(10.0);
    let dust = state.grid.get(pos).unwrap().building.as_ref().unwrap().dust;
    assert!(dust > 0.0);
}

#[test]
fn advance_runs_whole_ticks_and_banks_the_remainder() {
    let mut state = state();
    assert_eq!(state.advance(TICK_SECONDS * 2.5, false), 2);
    assert!((state.sim_accumulator - TICK_SECONDS * 0.5).abs() < 1e-4);
    assert!((state.time_played - (TICK_SECONDS * 2.0) as f64).abs() < 1e-4);

    // The banked half tick means the next one needs only half a tick more.
    assert_eq!(state.advance(TICK_SECONDS * 0.6, false), 1);
}

#[test]
fn a_paused_world_does_not_move() {
    let mut state = state();
    state.toggle_pause();
    assert!(state.paused);

    assert_eq!(state.advance(10.0, false), 0);
    assert_eq!(state.time_played, 0.0);
    assert_eq!(state.sim_accumulator, 0.0);

    // And starts again exactly where it stopped.
    state.toggle_pause();
    assert!(state.advance(TICK_SECONDS, false) > 0);
    assert!(state.time_played > 0.0);
}

#[test]
fn speed_buys_more_world_time_for_the_same_real_time() {
    let mut normal = state();
    let mut fast = state();
    fast.change_speed(true);
    assert_eq!(fast.time_scale, 2.0);

    // One second of wall clock each, in frames.
    for _ in 0..60 {
        normal.advance(1.0 / 60.0, false);
        fast.advance(1.0 / 60.0, false);
    }

    let ratio = fast.time_played / normal.time_played;
    assert!(
        (ratio - 2.0).abs() < 0.1,
        "double speed simulated {ratio} times as much"
    );
}

#[test]
fn the_speed_ladder_stops_at_both_ends() {
    let mut state = state();
    for _ in 0..10 {
        state.change_speed(false);
    }
    assert_eq!(state.time_scale, TIME_SCALES[0]);
    for _ in 0..10 {
        state.change_speed(true);
    }
    assert_eq!(state.time_scale, TIME_SCALES[TIME_SCALES.len() - 1]);
}

#[test]
fn the_fastest_speed_is_not_capped_away_by_the_catch_up_limit() {
    let mut state = state();
    for _ in 0..3 {
        state.change_speed(true);
    }
    assert_eq!(state.time_scale, 8.0);

    // A slow frame at top speed still buys everything it should.
    let ticks = state.advance(1.0 / 30.0, false);
    let expected = (8.0 / 30.0 / TICK_SECONDS).floor() as u32;
    assert_eq!(ticks, expected, "fast-forward lost time to the tick cap");
}

#[test]
fn advance_ignores_a_zero_or_nonsense_delta() {
    let mut state = state();
    assert_eq!(state.advance(0.0, false), 0);
    assert_eq!(state.advance(-1.0, false), 0);
    assert_eq!(state.advance(f32::NAN, false), 0);
    assert_eq!(state.time_played, 0.0);
}

#[test]
fn advance_drops_a_long_backlog_instead_of_replaying_it() {
    let mut state = state();
    // Ten seconds of stall: run the cap, then start clean.
    assert_eq!(state.advance(10.0, false), MAX_CATCHUP_TICKS);
    assert_eq!(state.sim_accumulator, 0.0);
}

#[test]
fn frame_rate_does_not_change_how_much_world_time_passes() {
    let mut fast = state();
    let mut slow = state();

    // Two seconds of wall clock, at 120fps and at 40fps.
    let mut fast_ticks = 0;
    for _ in 0..240 {
        fast_ticks += fast.advance(1.0 / 120.0, false);
    }
    let mut slow_ticks = 0;
    for _ in 0..80 {
        slow_ticks += slow.advance(1.0 / 40.0, false);
    }

    assert!(
        (fast_ticks as i64 - slow_ticks as i64).abs() <= 1,
        "{fast_ticks} vs {slow_ticks} ticks for the same wall clock"
    );
    assert!((fast.time_played - slow.time_played).abs() <= TICK_SECONDS as f64 * 1.5);
    assert!((fast.resources.energy - slow.resources.energy).abs() <= 1.0);
}

#[test]
fn power_collapse_triggers_after_sustained_negative_power() {
    let mut state = state();
    // Force a persistent deficit and drive the simulation past the 60s threshold.
    state.resources.energy = 1_000_000.0;
    state.config.resources.max_energy = 1_000_000.0;
    state.resources.minerals = 1_000_000.0;
    // A Server Bank placed adjacent to the Core is powered directly (Core
    // transmits power to neighbors) and consumes more than the Core generates.
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.unlock_building(BuildingType::ServerBank);
    state.select_building(BuildingType::ServerBank);
    assert!(state.try_place_building(pos));

    assert!(state.net_power() < 0.0);

    // Just past the trigger: the shutdown is short for a base this small,
    // so running well past it would only prove it had already lifted.
    for _ in 0..61 {
        state.step(1.0, false);
    }

    assert!(state.power_collapse_cooldown > 0.0);
    assert!(state.power_collapse_shutdown > 0.0);
}

#[test]
fn trigger_power_collapse_drops_drone_cargo_and_corrupts_progress() {
    let mut state = state();
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);
    state.drones.drones_mut()[0].carrying = 5.0;
    state.resources.data = 100.0;
    state.research.research_progress = 100.0;

    state.trigger_power_collapse();

    assert_eq!(state.drones.drones()[0].carrying, 0.0);
    assert_eq!(state.drones.drones()[0].state, DroneState::Error);
    // How much is lost scales with the size of the base, so this pins the
    // ends rather than one number that moves with the balance.
    let collapse = &GameConfig::default().collapse;
    let kept = state.resources.data;
    assert!(
        kept < 100.0 * (1.0 - collapse.min_data_loss) + 0.001
            && kept > 100.0 * (1.0 - collapse.max_data_loss),
        "a two-building base kept {} of 100 Data",
        kept
    );
    assert_eq!(state.research.research_progress, kept);
    assert_eq!(state.power_collapse_cooldown, collapse.cooldown_seconds);
}

#[test]
fn research_reduces_collapse_shutdown_and_data_loss_deterministically() {
    let mut bare = PlanetState::new(3, 7, GameConfig::default());
    let mut improved = PlanetState::new(3, 7, GameConfig::default());
    improved
        .research
        .unlocked_techs
        .extend(["thermal_sinks".to_string(), "advanced_research".to_string()]);
    improved.refresh_stats();

    bare.resources.data = 100.0;
    improved.resources.data = 100.0;
    bare.trigger_power_collapse();
    improved.trigger_power_collapse();
    assert!(improved.power_collapse_shutdown < bare.power_collapse_shutdown);
    assert!(improved.resources.data > bare.resources.data);
}

#[test]
fn tutorial_advances_when_first_drill_is_placed() {
    let mut state = state();
    assert_eq!(state.tutorial_step, 0);
    let core = state.grid.find_core().unwrap();
    let pos = GridPos::new(core.x + 1, core.y);
    state.grid.reveal_around(pos, 1);
    state.select_building(BuildingType::Drill);
    state.try_place_building(pos);

    state.update_tutorial();
    assert_eq!(state.tutorial_step, 1);
}
