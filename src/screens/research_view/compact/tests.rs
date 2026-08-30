use super::*;

#[test]
fn narrow_research_legend_uses_labels_that_fit_four_states() {
    assert_eq!(
        compact_legend_labels(240.0),
        ["OPEN", "WORK", "READY", "LOCK"]
    );
    assert_eq!(
        compact_legend_labels(360.0),
        ["Unlocked", "In Progress", "Available", "Locked"]
    );
}

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

#[test]
fn compact_viewport_focuses_the_active_research_node() {
    let research_tree = ResearchTree::from_nodes(vec![
        crate::engine::ResearchNode::new("core", "Core", "", 0.0, vec![], (0.0, 0.0)),
        crate::engine::ResearchNode::new("active", "Active", "", 100.0, vec!["core"], (-3.5, -5.0)),
    ]);
    let research_state = ResearchState {
        unlocked: vec!["core".to_string()],
        current_research: Some("active".to_string()),
        research_progress: 20.0,
        parked_progress: std::collections::HashMap::new(),
    };
    let tree = Rect::new(12.0, 172.0, 296.0, 98.0);
    let mut viewport = ResearchViewport::default();

    focus_compact_viewport(&mut viewport, &research_tree, &research_state, tree, None);

    let layout = TreeLayout::fit_with_min_scale(&research_tree.nodes, tree, 60.0)
        .with_view(viewport.zoom, (viewport.pan.x, viewport.pan.y));
    let focused = layout.to_screen((-3.5, -5.0));
    let center = vec2(tree.x + tree.w * 0.5, tree.y + tree.h * 0.5);
    assert!(vec2(focused.0, focused.1).distance(center) < 0.001);
    assert!(!viewport.focus_pending);
}

#[test]
fn short_phone_focus_uses_one_to_one_zoom_for_the_small_tree() {
    let research_tree = ResearchTree::from_nodes(vec![
        crate::engine::ResearchNode::new("core", "Core", "", 0.0, vec![], (0.0, 0.0)),
        crate::engine::ResearchNode::new("active", "Active", "", 100.0, vec!["core"], (-3.5, -5.0)),
    ]);
    let research_state = ResearchState {
        unlocked: vec!["core".to_string()],
        current_research: Some("active".to_string()),
        research_progress: 20.0,
        parked_progress: std::collections::HashMap::new(),
    };
    let tree = Rect::new(12.0, 172.0, 216.0, 70.0);
    let mut viewport = ResearchViewport::default();

    focus_compact_viewport(&mut viewport, &research_tree, &research_state, tree, None);

    assert_eq!(viewport.zoom, 1.0);
    let layout = TreeLayout::fit_with_min_scale(&research_tree.nodes, tree, 60.0)
        .with_view(viewport.zoom, (viewport.pan.x, viewport.pan.y));
    let focused = layout.to_screen((-3.5, -5.0));
    let center = vec2(tree.x + tree.w * 0.5, tree.y + tree.h * 0.5);
    assert!(vec2(focused.0, focused.1).distance(center) < 0.001);
}
