use super::{
    solar_map_layout, world_alert_code, world_rule_lines, world_telemetry_height,
    world_telemetry_row_y, MIN_SOLAR_MAP_SCALE,
};

#[test]
fn dossier_telemetry_rows_leave_room_for_the_stock_line() {
    let base_y = 100.0;
    assert_eq!(world_telemetry_row_y(base_y, 0), 100.0);
    assert_eq!(world_telemetry_row_y(base_y, 1), 116.0);
    assert_eq!(world_telemetry_row_y(base_y, 2), 132.0);
    assert_eq!(world_telemetry_height(), 54.0);
    assert!(world_telemetry_row_y(base_y, 2) < base_y + world_telemetry_height());
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

    assert!(rules.iter().any(|line| line == "Unavailable: Wind Turbine"));
    assert!(rules.iter().any(|line| line == "Needs Heater Node"));
    assert!(rules.iter().any(|line| line == "Needs 10 power generation"));
}
