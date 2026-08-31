use super::*;

fn metrics(screen_w: f32, screen_h: f32) -> HudMetrics {
    HudMetrics::for_screen(
        &UiTheme::default(),
        screen_w,
        screen_h,
        crate::state::Camera::default(),
        false,
    )
}

#[test]
fn factory_deck_fits_inside_portrait_viewports() {
    for (screen_w, screen_h) in [(360.0, 640.0), (320.0, 480.0)] {
        let layout = factory_deck_layout(screen_w, screen_h, metrics(screen_w, screen_h));
        assert!(layout.narrow);
        assert!(layout.panel.x >= 0.0);
        assert!(layout.panel.right() <= screen_w);
        assert!(layout.panel.bottom() <= screen_h - 64.0 - 12.0);
    }
}

#[test]
fn factory_deck_keeps_the_wide_control_room_geometry() {
    let layout = factory_deck_layout(1280.0, 720.0, metrics(1280.0, 720.0));

    assert!(!layout.narrow);
    assert!(!layout.compact);
    assert_eq!(layout.panel.w, 500.0);
    assert_eq!(layout.panel.h, 302.0);
    assert!(layout.panel.x > 300.0);
}

#[test]
fn narrow_focus_copy_stays_short_enough_for_touch_cards() {
    for focus in FactoryFocus::ALL {
        assert!(narrow_focus_description(focus).len() <= 25);
    }
}

#[test]
fn factory_deck_names_live_requirements_for_the_next_core_stage() {
    let state = PlanetState::new(2, 42, crate::data::GameConfig::default());
    let line = core_progress_text(&state, true);

    assert!(line.starts_with("CORE 1/5 -> FOUNDRY"));
    assert!(line.contains("STR "));
    assert!(line.contains("NET "));
}

#[test]
fn factory_deck_marks_a_fully_evolved_core() {
    let mut state = PlanetState::new(2, 42, crate::data::GameConfig::default());
    state.core_stage = (state.core_stages().len() - 1) as u8;

    assert_eq!(
        core_progress_text(&state, true),
        "CORE 5/5 // FULLY EVOLVED"
    );
}

#[test]
fn focus_status_explains_when_the_selected_deck_is_not_contributing() {
    let (status, warning) = focus_status_line(
        FactoryFocus::Assembly,
        FactoryFocusReadout {
            targets: 2,
            powered: 2,
            online: 2,
            producing: 1,
        },
    );

    assert_eq!(status, "FOCUS LIMITED // PARTS 1/2 producing");
    assert!(warning);
}

#[test]
fn focus_status_marks_an_unbuilt_deck_as_idle() {
    let (status, warning) =
        focus_status_line(FactoryFocus::Refining, FactoryFocusReadout::default());

    assert_eq!(status, "FOCUS IDLE // no target machines");
    assert!(warning);
}

#[test]
fn balanced_focus_does_not_claim_any_map_targets() {
    let state = PlanetState::new(2, 42, crate::data::GameConfig::default());
    assert!(focus_target_positions(&state).is_empty());
}

#[test]
fn assembly_focus_marks_both_machine_types_in_its_deck() {
    let mut state = PlanetState::new(2, 42, crate::data::GameConfig::default());
    let core = state.grid.find_core().unwrap();
    let server = crate::engine::GridPos::new(core.x + 1, core.y);
    let assembler = crate::engine::GridPos::new(core.x + 2, core.y);
    state.grid.get_mut(server).unwrap().building = Some(crate::engine::Building::new(
        crate::engine::BuildingType::ServerBank,
        server,
    ));
    state.grid.get_mut(assembler).unwrap().building = Some(crate::engine::Building::new(
        crate::engine::BuildingType::Assembler,
        assembler,
    ));
    state.factory_focus = FactoryFocus::Assembly;

    let targets = focus_target_positions(&state);
    assert_eq!(targets, vec![server, assembler]);
}
