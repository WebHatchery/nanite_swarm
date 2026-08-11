use super::*;
use crate::data::GameConfig;

fn reading(state: &PlanetState, stat: StatId) -> StatReading {
    state
        .stat_sheet()
        .into_iter()
        .find(|reading| reading.stat == stat)
        .expect("every stat is on the sheet")
}

fn state() -> PlanetState {
    PlanetState::new(0, 7, GameConfig::default())
}

#[test]
fn a_swarm_that_knows_nothing_sits_on_its_bases() {
    let state = state();
    for reading in state.stat_sheet() {
        assert!(
            !reading.is_changed(),
            "{} moved with nothing researched: {} from {}",
            reading.stat.id(),
            reading.value,
            reading.base
        );
    }
}

#[test]
fn the_sheet_covers_every_stat_once_and_keeps_its_order() {
    let sheet = state().stat_sheet();
    assert_eq!(sheet.len(), StatId::ALL.len());
    for (index, stat) in StatId::ALL.into_iter().enumerate() {
        assert_eq!(sheet[index].stat, stat);
    }
}

#[test]
fn research_moves_the_number_the_simulation_is_actually_using() {
    let mut state = state();
    state
        .research
        .unlocked_techs
        .push("efficient_drills".to_string());
    state.refresh_stats();

    let drills = reading(&state, StatId::DrillOutput);
    assert!(drills.is_changed());
    assert!(drills.is_gain());
    assert!(
        drills.value > drills.base,
        "{} is not more than {}",
        drills.value,
        drills.base
    );
}

#[test]
fn less_of_a_bad_thing_still_counts_as_a_gain() {
    let mut state = state();
    state
        .research
        .unlocked_techs
        .push("self_cleaning_servos".to_string());
    state.refresh_stats();

    let dust = reading(&state, StatId::DustAccumulation);
    assert!(dust.value < dust.base, "dust did not fall");
    assert!(dust.is_gain(), "less dust read as a loss");
}

#[test]
fn a_world_with_no_acid_reads_zero_rather_than_a_counter_working() {
    // Mars has no hazards, so both halves of the line are nothing and the
    // sheet must not claim the swarm is holding anything off.
    let state = state();
    let acid = reading(&state, StatId::AcidResistance);
    assert_eq!(acid.base, 0.0);
    assert_eq!(acid.value, 0.0);
    assert!(!acid.is_changed());
}

#[test]
fn faster_servos_reach_the_drones_and_not_only_the_sheet() {
    let mut state = state();
    let before = state.drones.drone_speed;

    state
        .research
        .unlocked_techs
        .push("servo_tuning".to_string());
    state.refresh_stats();

    let speed = reading(&state, StatId::DroneSpeed);
    assert!(speed.value > speed.base, "the sheet did not move");
    assert!(
        state.drones.drone_speed > before,
        "the drones are still walking at the old speed"
    );
    assert_eq!(state.drones.drone_speed, speed.value);
}

#[test]
fn amplifiers_widen_the_grid_the_power_flood_actually_uses() {
    let mut state = state();
    let before = state.grid.repeater_range;

    state
        .research
        .unlocked_techs
        .push("grid_amplifiers".to_string());
    state.refresh_stats();

    let reach = reading(&state, StatId::RepeaterRange);
    assert!(reach.value > reach.base);
    assert!(
        state.grid.repeater_range > before,
        "the grid is still carrying power the old distance"
    );
    assert_eq!(state.grid.repeater_range as f32, reach.value);
}

#[test]
fn harvest_yield_reads_as_a_share_of_ordinary_ground() {
    let mut state = state();
    let plain = reading(&state, StatId::HarvestYield);
    assert_eq!(plain.base, 1.0);
    assert!(!plain.is_changed());

    state
        .research
        .unlocked_techs
        .push("excavation_charges".to_string());
    state.refresh_stats();

    let charged = reading(&state, StatId::HarvestYield);
    assert!(charged.value > 1.0);
    assert!(charged.is_gain());
    assert_eq!(StatUnit::of(StatId::HarvestYield), StatUnit::Share);
}

#[test]
fn a_stats_units_come_from_the_same_file_that_names_it() {
    assert_eq!(StatUnit::of(StatId::DrillOutput), StatUnit::PerSecond);
    assert_eq!(StatUnit::of(StatId::DronesPerDrill), StatUnit::Count);
    assert_eq!(StatUnit::of(StatId::AcidResistance), StatUnit::Share);
    assert_eq!(StatUnit::PerSecond.format(7.5), "7.50/s");
    assert_eq!(StatUnit::Count.format(2.0), "2");
    assert_eq!(StatUnit::Share.format(0.4), "40%");
}
