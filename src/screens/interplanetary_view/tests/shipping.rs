use super::*;

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

#[test]
fn dossier_shipping_copy_keeps_route_health_visible() {
    assert_eq!(
        shipping_status_line("PAD OFFLINE"),
        "Route status: PAD OFFLINE".to_string()
    );
    assert_eq!(
        compact_shipping_status("Saturn", "RESERVE HELD"),
        "Route: Saturn RESERVE HELD".to_string()
    );
}

#[test]
fn standing_route_map_skips_self_routes_and_stale_endpoints() {
    let positions = [(10.0, 20.0), (30.0, 50.0)];

    assert_eq!(
        route_segment(0, 1, &positions),
        Some(((10.0, 20.0), (30.0, 50.0)))
    );
    assert!(route_segment(0, 0, &positions).is_none());
    assert!(route_segment(0, 2, &positions).is_none());
}

#[test]
fn remote_route_status_prioritizes_the_next_fixable_blocker() {
    assert_eq!(route_status_label(0, 0, 1, 1, 0, 1, 0, true), "NO DRIVER");
    assert_eq!(
        route_status_label(1, 0, 1, 1, 0, 1, 0, true),
        "DRIVER OFFLINE"
    );
    assert_eq!(route_status_label(1, 1, 0, 0, 0, 1, 0, true), "NO PAD");
    assert_eq!(route_status_label(1, 1, 1, 0, 0, 1, 0, true), "PAD OFFLINE");
    assert_eq!(route_status_label(1, 1, 1, 1, 1, 1, 0, true), "HOLDING");
    assert_eq!(route_status_label(1, 1, 1, 1, 0, 1, 1, true), "OVERFLOW");
    assert_eq!(route_status_label(1, 1, 1, 1, 0, 1, 0, true), "NOMINAL");
}

#[test]
fn remote_route_status_names_a_missing_pinned_landing_pad() {
    assert_eq!(
        route_status_label(1, 1, 1, 1, 0, 1, 0, false),
        "PAD TARGET UNAVAILABLE"
    );
}

#[test]
fn pinned_landing_pad_status_uses_the_live_online_pad_positions() {
    let data = crate::data::test_data();
    let live_pad = crate::engine::GridPos::new(4, 7);
    let pad_positions = vec![vec![live_pad]];
    let view = super::super::MapView {
        data: data.data(),
        current_planet: 0,
        mass_driver_researched: true,
        seed_ship_ready: false,
        colonized: &[],
        stockpiles: &[],
        telemetry: &[],
        current_drivers_online: 0,
        current_drivers_built: 0,
        launch_confirmation_target: None,
        drivers_online: &[],
        drivers_built: &[],
        pads: &[],
        pads_built: &[],
        pad_positions: &pad_positions,
        pod_fractions: &[],
        export_rates: &[],
        shipments: &[],
        orders: &[],
        pending_pods: &[],
        pod_caps: &[],
        overflow_pods: &[],
        editing_world: 0,
    };
    let order = crate::state::ExportOrder {
        target: 0,
        target_pad: Some(live_pad),
        ..crate::state::ExportOrder::default()
    };

    assert!(target_pad_available(&view, order));
    assert_eq!(
        route_status_label(1, 1, 1, 1, 0, 1, 0, target_pad_available(&view, order)),
        "NOMINAL"
    );

    let missing_order = crate::state::ExportOrder {
        target_pad: Some(crate::engine::GridPos::new(5, 7)),
        ..order
    };
    assert!(!target_pad_available(&view, missing_order));
    assert_eq!(
        route_status_label(
            1,
            1,
            1,
            1,
            0,
            1,
            0,
            target_pad_available(&view, missing_order)
        ),
        "PAD TARGET UNAVAILABLE"
    );
}

#[test]
fn shipping_reserve_status_explains_when_surplus_is_available() {
    assert_eq!(
        reserve_status_text(100.0, 100.0),
        ("Source reserve held: 100/100".to_string(), true)
    );
    assert_eq!(
        reserve_status_text(145.0, 100.0),
        ("Source surplus available: +45".to_string(), false)
    );
}

#[test]
fn biomass_reserve_uses_the_live_remote_stockpile() {
    let data = crate::data::test_data();
    let telemetry = [
        Some(crate::state::WorldTelemetry {
            minerals: 0.0,
            mineral_capacity: 100.0,
            data: 0.0,
            biomass: 100.0,
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
            alerts: 0,
            alert_kind: crate::state::WorldAlert::Nominal,
            storage_full_resource: None,
            maintenance_cause: None,
            core_stage: 0,
            core_stage_count: 1,
            core_next_requirement: None,
        }),
        None,
    ];
    let pads = [1, 1];
    let pads_built = [1, 1];
    let drivers = [1, 1];
    let colonized = [true, true];
    let pad_positions = [vec![], vec![]];
    let view = super::super::MapView {
        data: data.data(),
        current_planet: 0,
        mass_driver_researched: true,
        seed_ship_ready: false,
        colonized: &colonized,
        stockpiles: &[],
        telemetry: &telemetry,
        current_drivers_online: 1,
        current_drivers_built: 1,
        drivers_online: &drivers,
        drivers_built: &drivers,
        pads: &pads,
        pads_built: &pads_built,
        pad_positions: &pad_positions,
        pod_fractions: &[],
        export_rates: &[],
        shipments: &[],
        orders: &[],
        pending_pods: &[],
        pod_caps: &[],
        overflow_pods: &[],
        editing_world: 0,
        launch_confirmation_target: None,
    };
    let order = crate::state::ExportOrder {
        target: 1,
        cargo: crate::engine::ResourceType::Biomass,
        reserve_source: 100.0,
        surplus_only: true,
        ..crate::state::ExportOrder::default()
    };

    assert_eq!(route_status_for(&view, 0, order), "RESERVE HELD");
}

#[test]
fn shipping_route_notes_make_room_for_remote_orders() {
    assert_eq!(remote_section_shift(0), 0.0);
    assert_eq!(remote_section_shift(2), 0.0);
    assert_eq!(remote_section_shift(3), 16.0);
    assert_eq!(remote_section_shift(4), 32.0);
}

#[test]
fn in_flight_ledger_names_both_ends_of_a_shipment() {
    let data = crate::data::test_data();
    let shipment = crate::state::Shipment {
        from: 2,
        to: 4,
        cargo: crate::engine::ResourceType::Components,
        amount: 60.0,
        remaining: 10.0,
        transit: 20.0,
        target_pad: None,
        overflow: false,
        priority: 0,
    };

    assert_eq!(
        shipment_route_label(data.data(), &shipment),
        "60 Components Mars -> Saturn"
    );
}

#[test]
fn freight_lane_hit_test_selects_the_nearest_route_only_within_tolerance() {
    let positions = [(10.0, 20.0), (30.0, 50.0), (60.0, 20.0)];
    let orders = [
        Some(crate::state::ExportOrder {
            target: 1,
            ..crate::state::ExportOrder::default()
        }),
        None,
        Some(crate::state::ExportOrder {
            target: 1,
            ..crate::state::ExportOrder::default()
        }),
    ];

    assert_eq!(
        route_hit_test((20.0, 35.0), &orders, &positions, 2.0),
        Some(0)
    );
    assert_eq!(route_hit_test((20.0, 39.0), &orders, &positions, 2.0), None);
    assert_eq!(
        route_hit_test((45.0, 35.0), &orders, &positions, 2.0),
        Some(2)
    );
}
