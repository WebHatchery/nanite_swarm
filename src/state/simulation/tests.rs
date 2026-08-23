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

mod recipes;
mod runtime;
