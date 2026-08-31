//! Standing-order cycling and source-reserve policy checks.

use super::*;

#[test]
fn cycling_the_target_walks_the_colonized_worlds_and_then_holds() {
    let mut campaign = two_world_campaign();
    campaign.colonize(SATURN);

    campaign.cycle_export_target();
    assert_eq!(
        campaign.export_order().map(|order| order.target),
        Some(VENUS)
    );
    campaign.cycle_export_target();
    assert_eq!(
        campaign.export_order().map(|order| order.target),
        Some(SATURN)
    );
    campaign.cycle_export_target();
    assert!(
        campaign.export_order().is_none(),
        "past the last world is holding, not the first again"
    );
}

#[test]
fn a_world_is_never_offered_itself_as_a_target() {
    let mut campaign = two_world_campaign();
    for _ in 0..8 {
        campaign.cycle_export_target();
        if let Some(order) = campaign.export_order() {
            assert_ne!(order.target, campaign.current_index());
        }
    }
}

#[test]
fn a_lone_world_has_nowhere_to_ship_to() {
    let mut config = GameConfig::default();
    config.collapse.negative_power_seconds = f32::MAX;
    let mut campaign = Campaign::new(config, 42);

    campaign.cycle_export_target();

    assert!(campaign.export_order().is_none());
}

#[test]
fn cargo_cycles_through_what_the_drivers_accept() {
    let mut campaign = two_world_campaign();
    campaign.cycle_export_target();
    assert_eq!(
        campaign.export_order().map(|order| order.cargo),
        Some(ResourceType::Minerals)
    );

    campaign.cycle_export_cargo();
    assert_eq!(
        campaign.export_order().map(|order| order.cargo),
        Some(ResourceType::Alloy)
    );
    campaign.cycle_export_cargo();
    assert_eq!(
        campaign.export_order().map(|order| order.cargo),
        Some(ResourceType::Components)
    );
    campaign.cycle_export_cargo();
    assert_eq!(
        campaign.export_order().map(|order| order.cargo),
        Some(ResourceType::Minerals)
    );
}

#[test]
fn standing_orders_cycle_schedule_priority_and_reserve_policy() {
    let mut campaign = two_world_campaign();
    campaign.cycle_export_target();
    assert_eq!(campaign.export_order().unwrap().schedule_seconds, 0.0);

    campaign.cycle_export_schedule_for(MARS);
    campaign.cycle_export_priority_for(MARS);
    campaign.cycle_export_reserve_for(MARS);
    let order = campaign.export_order().unwrap();
    assert_eq!(order.schedule_seconds, 2.0);
    assert_eq!(order.priority, 1);
    assert!(order.surplus_only);
    assert_eq!(order.reserve_source, 100.0);

    campaign.cycle_export_reserve_for(MARS);
    assert_eq!(campaign.export_order().unwrap().reserve_source, 250.0);
    campaign.cycle_export_reserve_for(MARS);
    assert_eq!(campaign.export_order().unwrap().reserve_source, 500.0);
    campaign.cycle_export_reserve_for(MARS);
    assert_eq!(campaign.export_order().unwrap().reserve_source, 1_000.0);
    campaign.cycle_export_reserve_for(MARS);
    let order = campaign.export_order().unwrap();
    assert!(!order.surplus_only);
    assert_eq!(order.reserve_source, 0.0);
}

#[test]
fn a_source_reserve_holds_dispatch_until_stock_is_above_the_threshold() {
    let mut campaign = two_world_campaign();
    let driver = driver_on_current(&mut campaign);
    set_route(&mut campaign, VENUS, ResourceType::Minerals);
    campaign.current_mut().export = Some(ExportOrder {
        reserve_source: 100.0,
        surplus_only: true,
        ..campaign.export_order().unwrap()
    });
    campaign.current_mut().resources.minerals = 100.0;
    campaign
        .current_mut()
        .input_buffers
        .insert((driver.x, driver.y), 60.0);

    run(&mut campaign, 1.0);
    assert_eq!(campaign.current().pod_fraction(), 0.0);

    campaign.current_mut().resources.minerals = 101.0;
    run(&mut campaign, 1.0);
    assert!(campaign.current().pod_fraction() > 0.0);
}

#[test]
fn a_cargo_with_no_destination_is_not_an_order() {
    let mut campaign = two_world_campaign();
    campaign.cycle_export_cargo();
    assert!(campaign.export_order().is_none());
}
