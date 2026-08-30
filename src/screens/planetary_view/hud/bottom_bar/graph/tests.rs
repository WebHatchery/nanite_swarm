use super::*;
use crate::data::GameConfig;
use crate::state::GraphSample;

#[test]
fn graph_peak_uses_the_largest_signal_in_each_mode() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    state.graph_samples.push(GraphSample {
        power_produced: 4.0,
        power_consumed: 7.0,
        data_produced: 3.0,
        data_consumed: 9.0,
        minerals_consumed: 2.0,
        alloy_produced: 5.0,
        ..GraphSample::default()
    });

    assert_eq!(graph_peak(&state, GraphMode::Power), 7.0);
    assert_eq!(graph_peak(&state, GraphMode::Data), 9.0);
    assert_eq!(graph_peak(&state, GraphMode::Flow), 5.0);
}

#[test]
fn graph_readouts_name_the_latest_signal_or_empty_state() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());

    assert_eq!(latest_graph_readout(&state, GraphMode::Power), "NO PWR");
    assert_eq!(latest_graph_readout(&state, GraphMode::Data), "NO DATA");

    state.graph_samples.push(GraphSample {
        power_produced: 4.0,
        power_consumed: 1.0,
        data_produced: 6.0,
        data_consumed: 2.0,
        ..GraphSample::default()
    });
    assert_eq!(latest_graph_readout(&state, GraphMode::Power), "+4/-1");
    assert_eq!(latest_graph_readout(&state, GraphMode::Data), "+6/-2");
}
