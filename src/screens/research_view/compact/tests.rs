use super::*;

#[test]
fn line_crossing_tree_is_clipped_to_both_edges() {
    let tree = Rect::new(10.0, 20.0, 100.0, 80.0);
    let (from, to) = clip_line(tree, vec2(-20.0, 60.0), vec2(140.0, 60.0)).unwrap();
    assert_eq!(from, vec2(10.0, 60.0));
    assert_eq!(to, vec2(110.0, 60.0));
}

#[test]
fn line_wholly_outside_tree_is_rejected() {
    let tree = Rect::new(10.0, 20.0, 100.0, 80.0);
    assert!(clip_line(tree, vec2(-20.0, 4.0), vec2(140.0, 4.0)).is_none());
}

#[test]
fn line_already_inside_tree_is_unchanged() {
    let tree = Rect::new(10.0, 20.0, 100.0, 80.0);
    let from = vec2(30.0, 40.0);
    let to = vec2(90.0, 70.0);
    assert_eq!(clip_line(tree, from, to), Some((from, to)));
}

#[test]
fn active_research_status_reports_progress_instead_of_a_selection_prompt() {
    let node = crate::engine::ResearchNode::new(
        "adaptive_clocking",
        "Adaptive Clocking",
        "A test node",
        115.0,
        vec![],
        (0.0, 0.0),
    );
    let state = crate::engine::ResearchState {
        unlocked: vec![],
        current_research: Some("adaptive_clocking".to_string()),
        research_progress: 42.0,
        parked_progress: std::collections::HashMap::new(),
    };

    assert_eq!(
        active_research_status(&state, "adaptive_clocking", &node),
        Some("IN PROGRESS 42/115 DATA".to_string())
    );
    assert!(active_research_status(&state, "other", &node).is_none());
}

#[test]
fn parked_research_status_invites_the_player_to_resume_work() {
    let node = crate::engine::ResearchNode::new(
        "adaptive_clocking",
        "Adaptive Clocking",
        "A test node",
        115.0,
        vec![],
        (0.0, 0.0),
    );
    let state = crate::engine::ResearchState {
        unlocked: vec![],
        current_research: Some("other".to_string()),
        research_progress: 2.0,
        parked_progress: std::collections::HashMap::from([("adaptive_clocking".to_string(), 42.0)]),
    };

    assert_eq!(
        parked_research_status(&state, "adaptive_clocking", &node),
        Some("PAUSED 42/115 DATA - TAP TO RESUME".to_string())
    );
    assert!(parked_research_status(&state, "other", &node).is_none());
}

#[test]
fn compact_tree_drag_tracks_touch_delta_and_preserves_tap_distance() {
    let tree = Rect::new(10.0, 20.0, 100.0, 80.0);
    let mut viewport = ResearchViewport::default();
    let first = Pointer {
        position: vec2(30.0, 40.0),
        down: true,
        ..Pointer::default()
    };
    let second = Pointer {
        position: vec2(42.0, 47.0),
        down: true,
        ..Pointer::default()
    };

    apply_pointer_drag(&mut viewport, first, tree);
    apply_pointer_drag(&mut viewport, second, tree);

    assert_eq!(viewport.pan, vec2(12.0, 7.0));
    assert!((viewport.drag_distance - (12.0_f32 * 12.0 + 7.0 * 7.0).sqrt()).abs() < 0.001);

    apply_pointer_drag(&mut viewport, Pointer::default(), tree);
    assert!(viewport.last_pointer.is_none());
    assert!(!viewport.drag_active);
    assert!(viewport.drag_distance > 0.0);
}

#[test]
fn compact_tree_drag_ignores_a_gesture_that_started_outside_the_tree() {
    let tree = Rect::new(10.0, 20.0, 100.0, 80.0);
    let mut viewport = ResearchViewport::default();
    apply_pointer_drag(
        &mut viewport,
        Pointer {
            position: vec2(2.0, 4.0),
            down: true,
            ..Pointer::default()
        },
        tree,
    );
    apply_pointer_drag(
        &mut viewport,
        Pointer {
            position: vec2(42.0, 47.0),
            down: true,
            ..Pointer::default()
        },
        tree,
    );

    assert_eq!(viewport.pan, Vec2::ZERO);
    assert_eq!(viewport.drag_distance, 0.0);
}
