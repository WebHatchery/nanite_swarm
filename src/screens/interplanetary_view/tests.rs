use super::{world_alert_code, world_rule_lines};

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
