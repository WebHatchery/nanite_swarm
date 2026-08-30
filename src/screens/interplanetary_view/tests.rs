use super::compact_interplanetary_layout;
use super::shipping_panel::{
    remote_section_shift, reserve_status_text, route_hit_test, route_segment, route_status_label,
    shipment_route_label, target_pad_available,
};
use super::status::{
    driver_operation_status, landing_pad_operation_status, launch_action, launch_prompt,
    mass_driver_status_label, seed_ship_status_label,
};
use super::world_dossier::{compact_shipping_status, shipping_status_line};

#[test]
fn shipping_status_distinguishes_absent_and_offline_drivers() {
    assert_eq!(
        driver_operation_status(0, 0, "Mars"),
        "No Mass Driver on Mars"
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
use super::world_dossier::{
    core_stage_summary, maintenance_cause_label, storage_resource_label, world_telemetry_row_y,
};
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

#[test]
fn short_or_tablet_worlds_use_the_full_width_compact_layout() {
    assert!(compact_interplanetary_layout(800.0, 400.0));
    assert!(compact_interplanetary_layout(1280.0, 400.0));
    assert!(compact_interplanetary_layout(800.0, 720.0));
    assert!(!compact_interplanetary_layout(1280.0, 720.0));
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
        core_stage: 1,
        core_stage_count: 5,
        alerts,
        alert_kind,
        storage_full_resource: None,
        maintenance_cause: None,
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
    let mut maintenance = telemetry(1, crate::state::WorldAlert::Maintenance);
    maintenance.maintenance_cause = Some(crate::state::UpkeepAlertCause::Dust);
    assert_eq!(world_alert_code(Some(maintenance)), "MAINT DUST");
    maintenance.maintenance_cause = Some(crate::state::UpkeepAlertCause::Mixed);
    assert_eq!(world_alert_code(Some(maintenance)), "MAINT MIXED");
    assert_eq!(
        world_alert_code(Some(telemetry(1, crate::state::WorldAlert::Storage))),
        "STORE"
    );
    let mut data_storage = telemetry(1, crate::state::WorldAlert::Storage);
    data_storage.storage_full_resource = Some(crate::engine::ResourceType::Data);
    assert_eq!(world_alert_code(Some(data_storage)), "STORE DATA");
    data_storage.alerts = 2;
    assert_eq!(world_alert_code(Some(data_storage)), "STORE DATA x2");
    assert_eq!(
        storage_resource_label(Some(crate::engine::ResourceType::Data)),
        Some("DATA")
    );
    assert_eq!(
        storage_resource_label(Some(crate::engine::ResourceType::Minerals)),
        Some("ORE")
    );
    assert_eq!(
        storage_resource_label(Some(crate::engine::ResourceType::Energy)),
        None
    );
    assert_eq!(
        maintenance_cause_label(Some(crate::state::UpkeepAlertCause::Acid)),
        Some("ACID")
    );
}

#[test]
fn remote_dossiers_name_the_worlds_core_stage() {
    let data = crate::data::test_data();
    assert_eq!(
        core_stage_summary(
            data.data(),
            &telemetry(0, crate::state::WorldAlert::Nominal)
        ),
        "Core 1/5 > Foundry"
    );

    let mut finished = telemetry(0, crate::state::WorldAlert::Nominal);
    finished.core_stage = finished.core_stage_count;
    assert_eq!(core_stage_summary(data.data(), &finished), "Core 5/5");
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
    let view = super::MapView {
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
