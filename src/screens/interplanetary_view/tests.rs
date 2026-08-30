use super::status::{
    driver_operation_status, landing_pad_operation_status, launch_action, launch_prompt,
    mass_driver_status_label, seed_ship_status_label,
};

#[test]
fn shipping_status_distinguishes_absent_and_offline_drivers() {
    assert_eq!(
        driver_operation_status(0, 0, "Mars"),
        "No Mass Driver on this world"
    );
    assert_eq!(
        driver_operation_status(1, 0, "Mars"),
        "Mass Driver offline on Mars"
    );
    assert_eq!(
        driver_operation_status(2, 2, "Mars"),
        "2 drivers online on Mars"
    );
}

#[test]
fn shipping_status_distinguishes_absent_and_offline_landing_pads() {
    assert_eq!(
        landing_pad_operation_status(0, 0, "Saturn"),
        "No Landing Pad on Saturn"
    );
    assert_eq!(
        landing_pad_operation_status(1, 0, "Saturn"),
        "Landing Pad offline on Saturn"
    );
    assert_eq!(
        landing_pad_operation_status(2, 2, "Saturn"),
        "2 landing pads online on Saturn"
    );
}
use super::world_dossier::world_telemetry_row_y;
use super::{
    shipping_summary, solar_map_layout, world_alert_code, world_rule_lines, world_telemetry_height,
    MIN_SOLAR_MAP_SCALE,
};

#[test]
fn launch_header_distinguishes_a_built_driver_that_is_offline() {
    assert_eq!(mass_driver_status_label(false, 0, 0), "Mass Driver: LOCKED");
    assert_eq!(
        mass_driver_status_label(true, 0, 0),
        "Mass Driver: NOT BUILT"
    );
    assert_eq!(mass_driver_status_label(true, 1, 0), "Mass Driver: OFFLINE");
    assert_eq!(mass_driver_status_label(true, 1, 1), "Mass Driver: ONLINE");
}

#[test]
fn launch_header_explains_why_a_finished_ship_cannot_leave() {
    assert_eq!(
        seed_ship_status_label(false, 0, 0),
        "Seed Ship: UNDER CONSTRUCTION"
    );
    assert_eq!(
        seed_ship_status_label(true, 0, 0),
        "Seed Ship: READY / DRIVER NEEDED"
    );
    assert_eq!(
        seed_ship_status_label(true, 1, 0),
        "Seed Ship: READY / DRIVER OFFLINE"
    );
    assert_eq!(seed_ship_status_label(true, 1, 1), "Seed Ship: READY");
}

#[test]
fn launching_an_untouched_world_requires_a_second_tap() {
    assert_eq!(
        launch_action(0, false),
        super::InterplanetaryAction::RequestLaunch(0)
    );
    assert_eq!(
        launch_action(0, true),
        super::InterplanetaryAction::LaunchSeedShip(0)
    );
    assert_eq!(launch_prompt(false), "Tap to prepare launch");
    assert_eq!(launch_prompt(true), "Tap again to launch the Seed Ship");
}

#[test]
fn dossier_telemetry_rows_leave_room_for_refined_stock_and_health() {
    let base_y = 100.0;
    assert_eq!(world_telemetry_row_y(base_y, 0), 100.0);
    assert_eq!(world_telemetry_row_y(base_y, 1), 116.0);
    assert_eq!(world_telemetry_row_y(base_y, 2), 132.0);
    assert_eq!(world_telemetry_row_y(base_y, 3), 148.0);
    assert_eq!(world_telemetry_height(), 70.0);
    assert!(world_telemetry_row_y(base_y, 3) < base_y + world_telemetry_height());
}

#[test]
fn solar_map_scales_and_recenters_between_panels_on_narrow_windows() {
    let wide = solar_map_layout(1280.0, 720.0, 72.0, 16.0, 220.0, 310.0);
    let narrow = solar_map_layout(800.0, 600.0, 72.0, 16.0, 220.0, 310.0);
    let compressed = solar_map_layout(640.0, 480.0, 72.0, 16.0, 220.0, 310.0);

    assert!(narrow.2 < wide.2);
    assert!(narrow.2 * 310.0 <= 112.0);
    assert!(narrow.0 > 248.0 && narrow.0 < 472.0);
    assert!(narrow.1 > 72.0 && narrow.1 < 570.0);
    assert!(compressed.2 < MIN_SOLAR_MAP_SCALE);
}

fn telemetry(alerts: usize, alert_kind: crate::state::WorldAlert) -> crate::state::WorldTelemetry {
    crate::state::WorldTelemetry {
        minerals: 0.0,
        data: 0.0,
        biomass: 0.0,
        alloy: 0.0,
        components: 0.0,
        ore_rate: 0.0,
        data_rate: 0.0,
        biomass_rate: 0.0,
        alloy_rate: 0.0,
        components_rate: 0.0,
        power_balance: 0.0,
        structures: 0,
        drones: 0,
        alerts,
        alert_kind,
    }
}

#[test]
fn planet_list_alert_codes_keep_remote_failures_scannable() {
    assert_eq!(world_alert_code(None), "--");
    assert_eq!(
        world_alert_code(Some(telemetry(0, crate::state::WorldAlert::Nominal))),
        "OK"
    );
    assert_eq!(
        world_alert_code(Some(telemetry(1, crate::state::WorldAlert::Power))),
        "PWR"
    );
    assert_eq!(
        world_alert_code(Some(telemetry(3, crate::state::WorldAlert::Collapse))),
        "COLL x3"
    );
    assert_eq!(
        world_alert_code(Some(telemetry(1, crate::state::WorldAlert::Maintenance))),
        "MAINT"
    );
}

#[test]
fn dossier_rules_explain_buildings_a_world_will_not_accept() {
    let data = crate::data::test_data();
    let rules = world_rule_lines(data.data(), 1);

    assert!(
        rules.iter().any(|line| line == "Unavailable: Wind Turbine"),
        "Venus rules omitted its wind ban: {rules:?}"
    );
    assert!(
        rules
            .iter()
            .any(|line| line == "Unavailable: Biomass Harvester"),
        "Venus rules omitted its biomass ban: {rules:?}"
    );
}

#[test]
fn dossier_rules_keep_infrastructure_requirements_with_world_bans() {
    let data = crate::data::test_data();
    let rules = world_rule_lines(data.data(), 4);

    assert!(rules
        .iter()
        .any(|line| line == "Unavailable: Wind Turbine + Biomass Harvester"));
    assert!(rules.iter().any(|line| line == "Needs Heater Node"));
    assert!(rules
        .iter()
        .any(|line| line == "Needs 10 power generation via Fusion Reactor"));
}

#[test]
fn dossier_rules_name_the_power_route_for_hostile_worlds() {
    let data = crate::data::test_data();

    assert!(world_rule_lines(data.data(), 1)
        .iter()
        .any(|line| line == "Power source: Geothermal Tap"));
    assert!(world_rule_lines(data.data(), 3)
        .iter()
        .any(|line| line == "Power source: Fusion Reactor"));
}

#[test]
fn dossier_can_name_a_worlds_standing_shipping_route() {
    let data = crate::data::test_data();
    let order = crate::state::ExportOrder {
        target: 4,
        cargo: crate::engine::ResourceType::Minerals,
        ..crate::state::ExportOrder::default()
    };

    assert_eq!(
        shipping_summary(data.data(), Some(order), 180.0),
        Some("Shipping: Minerals -> Saturn @ 180/min".to_string())
    );
    assert!(shipping_summary(data.data(), None, 0.0).is_none());
}
