use super::*;
use crate::data::GameConfig;

#[test]
fn compact_inspector_rows_end_inside_the_panel_without_needing_the_icon_column() {
    let top = 86.0;
    let height = 148.0;
    let layout = inspector_row_layout(top, height);
    assert!(!layout.show_icon);
    assert!(layout.row_base + layout.row_gap * 3.0 <= top + height);
}

#[test]
fn tight_inspector_keeps_its_three_core_rows_inside_the_panel() {
    let top = 86.0;
    let height = 116.0;
    let layout = inspector_row_layout(top, height);

    assert!(!layout.show_icon);
    assert!(!layout.show_detail_rows);
    assert!(layout.row_base + layout.row_gap * 2.0 <= top + height);
}

#[test]
fn regular_inspector_keeps_the_building_art_and_four_rows() {
    let top = 96.0;
    let height = 192.0;
    let layout = inspector_row_layout(top, height);
    assert!(layout.show_icon);
    assert!(layout.row_base + layout.row_gap * 3.0 <= top + height);
}

#[test]
fn compact_terrain_bonus_clears_the_two_button_action_row() {
    let (_, _, bonus_y) = terrain_detail_offsets(148.0);
    let button_top = 148.0 - 38.0;

    assert!(bonus_y + 10.0 <= button_top);
}

#[test]
fn feature_detail_uses_only_the_space_the_inspector_can_hold() {
    assert_eq!(terrain_feature_lines(160.0, true), 0);
    assert_eq!(terrain_feature_lines(180.0, true), 1);
    assert_eq!(terrain_feature_lines(210.0, true), 2);
    assert_eq!(terrain_feature_lines(240.0, false), 0);
}

#[test]
fn feature_label_surfaces_local_feature_effects() {
    assert_eq!(
        feature_label("Iron Belt", Some(1.5), None),
        "Feature: Iron Belt  +50% ORE"
    );
    assert_eq!(
        feature_label("Acid Sea", None, Some(1.8)),
        "Feature: Acid Sea  +80% ACID"
    );
    assert_eq!(
        feature_label("Landing Crater", None, None),
        "Feature: Landing Crater"
    );
}

#[test]
fn multi_input_flow_is_structured_and_names_the_missing_feed() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let pos = state.grid.find_core().unwrap();
    state.input_hoppers.insert(
        (pos.x, pos.y),
        [(ResourceType::Minerals, 0.0), (ResourceType::Alloy, 8.0)]
            .into_iter()
            .collect(),
    );
    state.output_buffers.insert((pos.x, pos.y), 1.5);

    let flow = recipe_flow_data(&state, pos, BuildingType::Assembler).unwrap();
    assert_eq!(
        flow.inputs,
        vec![(ResourceType::Minerals, 0.0), (ResourceType::Alloy, 8.0)]
    );
    assert_eq!(flow.output, (ResourceType::Components, 1.5));
    assert_eq!(recipe_status(&flow, true), "Needs Ore");
    assert_eq!(recipe_status(&flow, false), "No power");
}

#[test]
fn paused_processor_explains_why_boost_is_unavailable() {
    assert_eq!(
        processor_action_label(false, false, true, true, false),
        "BOOST PAUSED"
    );
}

#[test]
fn full_processor_pad_keeps_purge_ahead_of_boost_policy() {
    assert_eq!(
        processor_action_label(true, false, true, true, false),
        "PURGE PAD"
    );
    assert_eq!(
        processor_action_label(true, true, true, true, false),
        "PURGE AGAIN"
    );
}

#[test]
fn a_full_dispatch_pad_is_named_before_it_can_hide_as_idle() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());
    let pos = state.grid.find_core().unwrap();
    state
        .output_buffers
        .insert((pos.x, pos.y), state.processor_pad_capacity());
    let flow = recipe_flow_data(&state, pos, BuildingType::Smelter).unwrap();
    assert_eq!(recipe_status(&flow, true), "Output pad full");
}

#[test]
fn core_inspector_names_its_current_and_next_evolution() {
    let mut state = PlanetState::new(2, 42, GameConfig::default());

    assert_eq!(core_stage_status_text(&state), "CORE 1/5 / STR1/10+NET1/6");
    assert_eq!(core_next_stage_label(&state), "Foundry");
    assert_eq!(state.core_requirement_progress().len(), 2);

    state.core_stage = (state.core_stages().len() - 1) as u8;
    assert_eq!(core_stage_status_text(&state), "CORE 5/5 // FULLY EVOLVED");
    assert_eq!(core_next_stage_label(&state), "FULLY EVOLVED");
}
