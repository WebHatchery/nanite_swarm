use super::*;

#[test]
fn the_solar_map_reads_live_telemetry_for_colonized_worlds() {
    let mut campaign = campaign();
    assert!(campaign.world_telemetry(0).is_none());

    assert!(campaign.colonize(0));
    let fresh = campaign.world_telemetry(0).unwrap();
    assert_eq!(fresh.minerals, 50.0);
    assert_eq!(fresh.mineral_capacity, 100.0);
    assert_eq!(fresh.data, 0.0);
    assert_eq!(fresh.biomass, 0.0);
    assert_eq!(fresh.alloy, 0.0);
    assert_eq!(fresh.components, 0.0);
    assert_eq!(fresh.data_rate, 0.25);
    assert_eq!(fresh.biomass_rate, 0.0);
    assert_eq!(fresh.structures, 1);
    assert_eq!(fresh.drones, 0);
    assert_eq!(fresh.core_stage, 1);
    assert_eq!(fresh.core_stage_count, 5);
    let next_core = fresh
        .core_next_requirement
        .expect("fresh Core should expose its first blocker");
    assert_eq!(next_core.milestone, crate::state::Milestone::Structures);
    assert_eq!(next_core.current, 1.0);
    assert_eq!(next_core.target, 10.0);
    assert_eq!(fresh.alerts, 0);
    assert_eq!(fresh.alert_kind, WorldAlert::Nominal);
    assert_eq!(fresh.storage_full_resource, None);

    let mercury = campaign.planet_mut(0).unwrap();
    mercury.config.resources.base_mineral_cap = 200.0;
    mercury.resources.minerals = 123.0;
    mercury.resources.data = 23.0;
    mercury.resources.biomass = 7.0;
    mercury.resources.alloy = 41.0;
    mercury.resources.components = 13.0;
    mercury.power_balance = -1.0;
    let live = campaign.world_telemetry(0).unwrap();
    assert_eq!(live.minerals, 123.0);
    assert_eq!(live.data, 23.0);
    assert_eq!(live.biomass, 7.0);
    assert_eq!(live.alloy, 41.0);
    assert_eq!(live.components, 13.0);
    assert_eq!(live.power_balance, -1.0);
    assert_eq!(live.alerts, 1);
    assert_eq!(live.alert_kind, WorldAlert::Power);
}

#[test]
fn the_solar_map_reports_a_remote_world_with_full_storage() {
    let mut campaign = campaign();
    assert!(campaign.colonize(0));
    let world = campaign.planet_mut(0).expect("colonized Mercury");
    world.resources.data = world.resource_capacity(crate::engine::ResourceType::Data);

    let telemetry = campaign.world_telemetry(0).unwrap();
    assert_eq!(
        telemetry.storage_full_resource,
        Some(crate::engine::ResourceType::Data)
    );
    assert_eq!(telemetry.alerts, 1);
    assert_eq!(telemetry.alert_kind, WorldAlert::Storage);
}

#[test]
fn the_solar_map_does_not_alert_for_a_full_idle_mineral_reserve() {
    let mut campaign = campaign();
    assert!(campaign.colonize(0));
    let world = campaign.planet_mut(0).expect("colonized Mercury");
    world.config.resources.base_mineral_cap = world.resources.minerals;

    let telemetry = campaign.world_telemetry(0).unwrap();
    assert_eq!(telemetry.storage_full_resource, None);
    assert_eq!(telemetry.alerts, 0);
    assert_eq!(telemetry.alert_kind, WorldAlert::Nominal);
}

#[test]
fn the_solar_map_prioritizes_collapse_over_secondary_world_alerts() {
    let mut campaign = campaign();
    let world = campaign.current_mut();
    world.power_balance = -1.0;
    world.battery_seconds = 0.0;
    world.power_collapse_shutdown = 5.0;

    let telemetry = campaign.world_telemetry(STARTING_PLANET).unwrap();
    assert_eq!(telemetry.alerts, 3);
    assert_eq!(telemetry.alert_kind, WorldAlert::Collapse);
}

#[test]
fn the_solar_map_reports_upkeep_degradation_on_a_remote_world() {
    let mut campaign = campaign();
    assert!(campaign.colonize(0));
    let world = campaign.planet_mut(0).expect("colonized Mercury");
    let core = world.grid.find_core().expect("generated Core");
    world
        .grid
        .get_mut(core)
        .expect("core tile")
        .building
        .as_mut()
        .expect("core building")
        .dust = world.resolved_dust_response().efficiency_threshold;

    let telemetry = campaign.world_telemetry(0).unwrap();
    assert_eq!(telemetry.alerts, 1);
    assert_eq!(telemetry.alert_kind, WorldAlert::Maintenance);
    assert_eq!(
        telemetry.maintenance_cause,
        Some(crate::state::UpkeepAlertCause::Dust)
    );
}
