use super::*;
use crate::data::GameConfig;
use crate::engine::{Building, BuildingType, ResourceType, TerrainType};

fn automated_processor() -> (PlanetState, GridPos) {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    state
        .research
        .unlocked_techs
        .push("adaptive_clocking".into());
    state.auto_clocking = true;
    let core = state.grid.find_core().unwrap();
    let processor = GridPos::new(core.x + 1, core.y);
    let turbine = GridPos::new(core.x - 1, core.y);
    for pos in [processor, turbine] {
        state.grid.get_mut(pos).unwrap().terrain = TerrainType::Empty;
        state.grid.reveal_around(pos, 1);
    }
    state.grid.get_mut(processor).unwrap().building =
        Some(Building::new(BuildingType::Smelter, processor));
    state.grid.get_mut(turbine).unwrap().building =
        Some(Building::new(BuildingType::WindTurbine, turbine));
    state.grid.update_power_grid();
    state.input_hoppers.insert(
        (processor.x, processor.y),
        [(ResourceType::Minerals, 20.0)].into_iter().collect(),
    );
    (state, processor)
}

#[test]
fn policy_boosts_a_fed_processor_then_normalizes_it_when_starved() {
    let (mut state, processor) = automated_processor();
    let (boosted, normalized) = state.rebalance_auto_clocking();
    assert_eq!((boosted, normalized), (1, 0));
    assert!(
        state
            .grid
            .get(processor)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );

    state.input_hoppers.clear();
    let (boosted, normalized) = state.rebalance_auto_clocking();
    assert_eq!((boosted, normalized), (0, 1));
    assert!(
        !state
            .grid
            .get(processor)
            .unwrap()
            .building
            .as_ref()
            .unwrap()
            .overclocked
    );
}

#[test]
fn dusty_processors_stay_normal_until_they_are_clean() {
    let (mut state, processor) = automated_processor();
    state
        .grid
        .get_mut(processor)
        .unwrap()
        .building
        .as_mut()
        .unwrap()
        .dust = 60.0;
    assert_eq!(state.rebalance_auto_clocking(), (0, 0));
}
