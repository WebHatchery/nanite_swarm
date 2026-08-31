use super::{
    compact_stockpile_summary, mobile_footer_hint_text, mobile_info_height, mobile_route_action,
    mobile_status_text, mobile_world_focus, mobile_worlds_layout,
};
use crate::state::{InterplanetaryMobilePanel, WorldAlert, WorldTelemetry};

fn telemetry() -> WorldTelemetry {
    WorldTelemetry {
        minerals: 123.0,
        mineral_capacity: 200.0,
        data: 4.0,
        biomass: 5.0,
        alloy: 6.0,
        components: 7.0,
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
        core_next_requirement: None,
        alerts: 0,
        alert_kind: WorldAlert::Nominal,
        storage_full_resource: None,
        maintenance_cause: None,
    }
}

#[test]
fn mobile_dossier_status_keeps_the_remote_alert_reason_visible() {
    assert_eq!(
        mobile_status_text(true, None),
        "Status: COLONIZED".to_string()
    );
    assert_eq!(
        mobile_status_text(true, Some("MAINT DUST")),
        "Status: COLONIZED / MAINT DUST".to_string()
    );
    assert_eq!(
        mobile_status_text(false, None),
        "Status: UNEXPLORED".to_string()
    );
}

#[test]
fn short_dossier_stock_summary_keeps_every_remote_pool_scannable() {
    assert_eq!(
        compact_stockpile_summary(&telemetry()),
        "Stock M123/200 D4 B5 A6 C7"
    );
}

#[test]
fn short_phone_dossier_stays_above_the_footer() {
    let list_bottom = 156.0 + 44.0 + crate::state::PLANET_COUNT as f32 * 24.0;
    let dossier_bottom = list_bottom + 10.0 + mobile_info_height(480.0, true, false);

    assert_eq!(mobile_info_height(480.0, false, false), 96.0);
    assert_eq!(mobile_info_height(480.0, true, false), 124.0);
    assert!(dossier_bottom <= 454.0);
}

#[test]
fn very_short_worlds_layout_keeps_the_dossier_action_visible() {
    let layout = mobile_worlds_layout(400.0, crate::state::PLANET_COUNT);
    let info_y = 156.0 + layout.list_h + layout.info_gap;
    let dossier_bottom = info_y + mobile_info_height(400.0, true, false);

    assert!(layout.compact);
    assert!((layout.row_height - 19.0).abs() < 0.001);
    assert!((layout.list_h - 129.0).abs() < 0.001);
    assert!((info_y - 291.0).abs() < 0.001);
    assert!((dossier_bottom - 371.0).abs() < 0.001);
    assert!(dossier_bottom < 400.0 - 18.0);
}

#[test]
fn regular_phone_worlds_layout_keeps_the_existing_rows() {
    let layout = mobile_worlds_layout(480.0, crate::state::PLANET_COUNT);

    assert!(!layout.compact);
    assert!((layout.row_height - 24.0).abs() < 0.001);
    assert!((layout.list_h - 164.0).abs() < 0.001);
    assert!((layout.info_gap - 10.0).abs() < 0.001);
}

#[test]
fn regular_phone_dossier_keeps_the_full_world_summary() {
    assert_eq!(mobile_info_height(640.0, true, false), 250.0);
    assert_eq!(mobile_info_height(640.0, true, true), 282.0);
}

#[test]
fn narrow_footer_hints_keep_the_next_visible_control_named() {
    let worlds = mobile_footer_hint_text(InterplanetaryMobilePanel::Worlds, false, 240.0);
    let confirm = mobile_footer_hint_text(InterplanetaryMobilePanel::Worlds, true, 240.0);
    let shipping = mobile_footer_hint_text(InterplanetaryMobilePanel::Shipping, false, 240.0);

    assert!(worlds.contains("SHIPPING"));
    assert!(worlds.contains("world"));
    assert!(confirm.contains("CONFIRM"));
    assert!(shipping.contains("WORLDS"));
}

#[test]
fn mobile_worlds_keeps_the_selected_remote_source_after_switching_tabs() {
    assert_eq!(mobile_world_focus(None, None, 4, 0, 5), 4);
    assert_eq!(mobile_world_focus(Some(2), None, 4, 0, 5), 2);
    assert_eq!(mobile_world_focus(None, Some(3), 4, 0, 5), 3);
    assert_eq!(mobile_world_focus(None, None, 8, 0, 5), 0);
}

#[test]
fn mobile_route_recovery_names_the_visible_world_row_to_tap() {
    assert_eq!(
        mobile_route_action(
            "Mars",
            Some("Venus"),
            Some("PAD OFFLINE"),
            false,
            false,
            true,
        ),
        Some("Tap Venus row to travel".to_string())
    );
    assert_eq!(
        mobile_route_action(
            "Mars",
            Some("Venus"),
            Some("DRIVER OFFLINE"),
            false,
            false,
            true,
        ),
        Some("Tap Mars row to travel".to_string())
    );
    assert_eq!(
        mobile_route_action(
            "Mars",
            Some("Venus"),
            Some("PAD OFFLINE"),
            true,
            false,
            true,
        ),
        Some("Tap Venus row to travel".to_string())
    );
    assert_eq!(
        mobile_route_action("Mars", Some("Venus"), Some("NO PAD"), false, true, true),
        Some("Current location - tap Back, then BUILD".to_string())
    );
    assert_eq!(
        mobile_route_action("Mars", Some("Venus"), None, true, false, true),
        Some("Current location".to_string())
    );
}
