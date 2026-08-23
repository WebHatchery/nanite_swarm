//! Recipe, processor, and routed-output simulation checks.

use super::*;

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
fn standby_processor_holds_both_sides_of_its_recipe_unchanged() {
    let (mut state, pos) = processing_world(BuildingType::Smelter, 60.0);
    state
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .standby = true;

    state.update_recipes(5.0);

    assert_eq!(state.input_buffers.get(&(pos.x, pos.y)), Some(&60.0));
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

    state
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .powered = true;
    state
        .grid
        .get_mut(pos)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .standby = true;
    assert!(
        state.starved_factories().is_empty(),
        "a deliberately paused machine is not starved"
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
