use super::world_rule_lines;

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
