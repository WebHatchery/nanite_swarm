use super::*;
use crate::data::DustResponseConfig;

fn drill_at_origin() -> Building {
    Building::new(BuildingType::Drill, GridPos::new(0, 0))
}

#[test]
fn new_building_starts_unpowered_unless_core() {
    let drill = drill_at_origin();
    assert!(!drill.powered);
    assert!(!drill.connected_to_core);

    let core = Building::new(BuildingType::Core, GridPos::new(1, 1));
    assert!(core.powered);
    assert!(core.connected_to_core);
}

#[test]
fn dust_efficiency_degrades_in_steps() {
    let mut building = drill_at_origin();
    building.dust = 0.0;
    assert_eq!(building.dust_efficiency(), 1.0);
    building.dust = 25.0;
    assert_eq!(building.dust_efficiency(), 0.9);
    building.dust = 100.0;
    assert_eq!(building.dust_efficiency(), 0.0);
}

#[test]
fn dust_drone_speed_multiplier_slows_at_50() {
    let mut building = drill_at_origin();
    building.dust = 49.0;
    assert_eq!(building.dust_drone_speed_multiplier(), 1.0);
    building.dust = 50.0;
    assert_eq!(building.dust_drone_speed_multiplier(), 0.7);
}

#[test]
fn dust_stalled_only_at_100() {
    let mut building = drill_at_origin();
    building.dust = 99.9;
    assert!(!building.is_dust_stalled());
    building.dust = 100.0;
    assert!(building.is_dust_stalled());
}

#[test]
fn dust_power_leak_only_for_transmitters_over_75() {
    let mut conduit = Building::new(BuildingType::Conduit, GridPos::new(0, 0));
    conduit.dust = 80.0;
    assert!(conduit.transmits_power());
    assert_eq!(conduit.dust_power_leak(), 0.5);

    let mut drill = drill_at_origin();
    drill.dust = 80.0;
    assert!(!drill.transmits_power());
    assert_eq!(drill.dust_power_leak(), 0.0);
}

#[test]
fn dust_response_thresholds_are_applied_by_runtime_helpers() {
    let response = DustResponseConfig {
        efficiency_threshold: 40.0,
        speed_threshold: 60.0,
        leak_threshold: 80.0,
        stall_threshold: 90.0,
        ..DustResponseConfig::default()
    };
    let mut conduit = Building::new(BuildingType::Conduit, GridPos::new(0, 0));

    conduit.dust = 39.9;
    assert_eq!(conduit.dust_efficiency_with(&response), 1.0);
    assert_eq!(conduit.dust_drone_speed_multiplier_with(&response), 1.0);
    assert_eq!(conduit.dust_power_leak_with(&response), 0.0);
    assert_eq!(
        conduit.dust_power_generation_multiplier_with(&response),
        1.0
    );

    conduit.dust = 80.0;
    assert_eq!(conduit.dust_efficiency_with(&response), response.efficiency);
    assert_eq!(
        conduit.dust_drone_speed_multiplier_with(&response),
        response.speed_multiplier
    );
    assert_eq!(conduit.dust_power_leak_with(&response), response.leak);
    assert_eq!(
        conduit.dust_power_generation_multiplier_with(&response),
        0.7
    );

    conduit.dust = 90.0;
    assert!(conduit.is_dust_stalled_with(&response));
    assert_eq!(conduit.dust_efficiency_with(&response), 0.0);
    assert_eq!(
        conduit.dust_power_generation_multiplier_with(&response),
        0.0
    );
    assert_eq!(
        conduit.dust_power_consumption_multiplier_with(&response),
        0.0
    );
}

#[test]
fn only_recipe_buildings_accept_the_overclock_tradeoff() {
    let mut assembler = Building::new(BuildingType::Assembler, GridPos::new(0, 0));
    assert!(assembler.supports_overclock());
    assert_eq!(assembler.work_multiplier(), 1.0);
    assembler.overclocked = true;
    assert_eq!(assembler.work_multiplier(), 1.5);
    assert_eq!(assembler.power_demand_multiplier(), 1.75);
    assert_eq!(assembler.dust_accumulation_multiplier(), 1.8);
    assembler.standby = true;
    assert_eq!(assembler.work_multiplier(), 0.0);
    assert_eq!(assembler.power_demand_multiplier(), 0.0);

    let mut conduit = Building::new(BuildingType::Conduit, GridPos::new(1, 0));
    conduit.overclocked = true;
    assert!(!conduit.supports_overclock());
    assert_eq!(conduit.work_multiplier(), 1.0);
    assert_eq!(conduit.power_demand_multiplier(), 1.0);
}
