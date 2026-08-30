use super::*;

#[test]
fn resource_spending_records_the_actual_build_costs() {
    let mut resources = crate::state::Resources {
        minerals: 40.0,
        energy: 12.0,
        ..Default::default()
    };

    assert!(resources.spend(15.0, 4.0));
    assert_eq!(resources.minerals, 25.0);
    assert_eq!(resources.energy, 8.0);
    assert_eq!(resources.consumed.minerals, 15.0);
    assert_eq!(resources.consumed.energy, 4.0);
}

#[test]
fn records_only_finite_positive_sinks() {
    let mut totals = ConsumptionTotals::default();
    totals.record(ResourceType::Minerals, 12.5);
    totals.record(ResourceType::Data, -3.0);
    totals.record(ResourceType::Energy, f32::NAN);

    assert_eq!(totals.minerals, 12.5);
    assert_eq!(totals.data, 0.0);
    assert_eq!(totals.energy, 0.0);
}

#[test]
fn combines_world_totals_without_aliasing_them() {
    let mut total = ConsumptionTotals {
        minerals: 2.0,
        ..Default::default()
    };
    total.add(ConsumptionTotals {
        minerals: 3.0,
        components: 4.0,
        ..Default::default()
    });

    assert_eq!(total.minerals, 5.0);
    assert_eq!(total.components, 4.0);
}

#[test]
fn old_save_shape_defaults_to_an_empty_ledger() {
    let resources: crate::state::Resources = serde_json::from_str(
        r#"{"energy":10.0,"minerals":20.0,"data":0.0,"biomass":0.0,"alloy":0.0,"components":0.0}"#,
    )
    .expect("legacy resources should still load");

    assert_eq!(resources.consumed, ConsumptionTotals::default());
}
