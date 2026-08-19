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
fn regular_inspector_keeps_the_building_art_and_four_rows() {
    let top = 96.0;
    let height = 192.0;
    let layout = inspector_row_layout(top, height);
    assert!(layout.show_icon);
    assert!(layout.row_base + layout.row_gap * 3.0 <= top + height);
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
