//! Compact research-tree viewport controls and detail panel.

use super::*;

pub(super) fn draw_compact_legend(x: f32, y: f32) {
    let entries = [
        ("Unlocked", Colors::PRIMARY),
        ("In Progress", Colors::WARNING),
        ("Available", Colors::SUCCESS),
        ("Locked", Colors::SECONDARY),
    ];
    let mut entry_x = x;
    for (label, color) in entries {
        draw_circle(entry_x + 5.0, y - 4.0, 5.0, color);
        draw_ui_text(label, entry_x + 14.0, y, 11.0, Colors::TEXT_DIM);
        entry_x += measure_ui_text(label, None, 11, 1.0).width + 36.0;
    }
}

pub(super) fn active_research_status(
    research_state: &ResearchState,
    node_id: &str,
    node: &crate::engine::ResearchNode,
) -> Option<String> {
    if research_state.current_research.as_deref() != Some(node_id)
        || research_state.is_unlocked(node_id)
    {
        return None;
    }
    Some(format!(
        "IN PROGRESS {:.0}/{:.0} DATA",
        research_state.research_progress.min(node.data_cost),
        node.data_cost
    ))
}

pub(super) fn parked_research_status(
    research_state: &ResearchState,
    node_id: &str,
    node: &crate::engine::ResearchNode,
) -> Option<String> {
    if research_state.current_research.as_deref() == Some(node_id)
        || research_state.is_unlocked(node_id)
    {
        return None;
    }
    let progress = research_state.progress_for(node_id);
    (progress > 0.001).then(|| {
        format!(
            "PAUSED {:.0}/{:.0} DATA - TAP TO RESUME",
            progress.min(node.data_cost),
            node.data_cost
        )
    })
}

pub(super) fn update_compact_viewport(viewport: &mut ResearchViewport, tree: Rect) {
    let pointer = Pointer::read(|position| position);
    apply_pointer_drag(viewport, pointer, tree);
    let (_, wheel_y) = mouse_wheel();
    if wheel_y.abs() > f32::EPSILON && tree.contains(pointer.position) {
        viewport.zoom = (viewport.zoom + wheel_y.signum() * 0.1).clamp(1.0, 2.4);
    }
    viewport.pan.x = viewport.pan.x.clamp(-tree.w, tree.w);
    viewport.pan.y = viewport.pan.y.clamp(-tree.h * 2.5, tree.h * 2.5);
}

/// Put the most relevant node in the middle of a compact tree the first time
/// it is shown. On a short phone viewport the authored network is taller than
/// the available field, so a neutral world-space center can hide the active
/// job completely behind the detail drawer.
pub(super) fn focus_compact_viewport(
    viewport: &mut ResearchViewport,
    research_tree: &ResearchTree,
    research_state: &ResearchState,
    tree: Rect,
    active_condition: Option<&str>,
) {
    if !viewport.focus_pending {
        return;
    }

    let focus_position = research_state
        .current_research
        .as_deref()
        .and_then(|id| research_tree.get_node(id))
        .or_else(|| {
            research_tree.nodes.iter().find(|node| {
                !research_state.is_unlocked(&node.id)
                    && research_tree.can_select_on(
                        &node.id,
                        &research_state.unlocked,
                        active_condition,
                    )
            })
        })
        .or_else(|| research_tree.nodes.first())
        .map(|node| node.position);

    if let Some(position) = focus_position {
        let layout = TreeLayout::fit_with_min_scale(&research_tree.nodes, tree, 60.0)
            .with_view(viewport.zoom, (0.0, 0.0));
        let target = vec2(layout.to_screen(position).0, layout.to_screen(position).1);
        let center = vec2(tree.x + tree.w * 0.5, tree.y + tree.h * 0.5);
        viewport.pan = center - target;
    }

    viewport.center_pan = viewport.pan;
    viewport.center_zoom = viewport.zoom;
    viewport.focus_pending = false;
}

/// Track a drag from the shared pointer so a finger can pan the same compact
/// tree as a mouse. A gesture that starts outside the tree never steals a
/// later release from the surrounding controls.
fn apply_pointer_drag(viewport: &mut ResearchViewport, pointer: Pointer, tree: Rect) {
    if !pointer.down {
        viewport.last_pointer = None;
        viewport.drag_active = false;
        return;
    }

    if viewport.last_pointer.is_none() {
        viewport.drag_distance = 0.0;
        viewport.drag_active = tree.contains(pointer.position);
    }
    if viewport.drag_active {
        if let Some(previous) = viewport.last_pointer {
            let delta = pointer.position - previous;
            viewport.pan += delta;
            viewport.drag_distance += delta.length();
        }
    }
    viewport.last_pointer = Some(pointer.position);
}

pub(super) fn draw_compact_viewport_controls(
    viewport: &mut ResearchViewport,
    screen_w: f32,
    mobile: bool,
) {
    let y = if mobile {
        HEADER_HEIGHT + 8.0
    } else {
        HEADER_HEIGHT + 4.0
    };
    let mut x = if mobile { 16.0 } else { screen_w - 178.0 };
    if draw_button_sized(x, y, 32.0, 28.0, "-") {
        viewport.zoom = (viewport.zoom - 0.15).max(1.0);
    }
    x += 38.0;
    if draw_button_sized(x, y, 32.0, 28.0, "+") {
        viewport.zoom = (viewport.zoom + 0.15).min(2.4);
    }
    x += 38.0;
    if draw_button_sized(x, y, 68.0, 28.0, "Center") {
        viewport.pan = viewport.center_pan;
        viewport.zoom = viewport.center_zoom;
    }
}

pub(super) fn clip_line(rect: Rect, from: Vec2, to: Vec2) -> Option<(Vec2, Vec2)> {
    let delta = to - from;
    let checks = [
        (-delta.x, from.x - rect.x),
        (delta.x, rect.right() - from.x),
        (-delta.y, from.y - rect.y),
        (delta.y, rect.bottom() - from.y),
    ];
    let (mut start, mut end) = (0.0_f32, 1.0_f32);
    for (p, q) in checks {
        if p.abs() < f32::EPSILON {
            if q < 0.0 {
                return None;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                start = start.max(t);
            } else {
                end = end.min(t);
            }
        }
    }
    (start <= end).then_some((from + delta * start, from + delta * end))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_compact_node_info(
    data: &crate::data::GameData,
    research_state: &ResearchState,
    research_tree: &ResearchTree,
    node_id: &str,
    heading: &str,
    panel_x: f32,
    y: f32,
    panel_w: f32,
    data_available: f32,
    active_condition: Option<&str>,
) {
    let Some(node) = research_tree.get_node(node_id) else {
        return;
    };
    if panel_w < 360.0 {
        draw_mobile_node_info(
            data,
            research_state,
            research_tree,
            node_id,
            heading,
            panel_x,
            y,
            panel_w,
            data_available,
            active_condition,
        );
        return;
    }
    let split_x = panel_x + panel_w * 0.52;
    draw_ui_text(heading, panel_x + 12.0, y, 12.0, Colors::TEXT_DIM);
    draw_ui_text(&node.name, panel_x + 12.0, y + 18.0, 14.0, Colors::TEXT);
    let description = truncate_text_to_width(&node.description, panel_w * 0.46, 11.0);
    draw_ui_text(
        &description,
        panel_x + 12.0,
        y + 38.0,
        11.0,
        Colors::TEXT_DIM,
    );
    let mut detail_y = y;
    for (text, color) in node_effects(data, node_id) {
        draw_ui_text(&text, split_x, detail_y, 11.0, color);
        detail_y += 16.0;
    }
    if research_state.is_unlocked(node_id) {
        draw_ui_text("UNLOCKED", split_x, detail_y + 2.0, 11.0, Colors::SUCCESS);
        return;
    }
    if let Some(status) = active_research_status(research_state, node_id, node) {
        draw_ui_text(&status, split_x, detail_y + 2.0, 11.0, Colors::WARNING);
        return;
    }
    if let Some(status) = parked_research_status(research_state, node_id, node) {
        draw_ui_text(
            &truncate_text_to_width(&status, panel_w * 0.46, 11.0),
            split_x,
            detail_y + 2.0,
            11.0,
            Colors::WARNING,
        );
        return;
    }
    draw_ui_text(
        &format!("Cost {:.0} Data", node.data_cost),
        split_x,
        detail_y + 2.0,
        11.0,
        Colors::ACCENT,
    );
    let status = if node
        .planet_condition
        .as_deref()
        .is_some_and(|required| active_condition != Some(required))
    {
        (
            format!(
                "Requires {} world",
                node.planet_condition.as_deref().unwrap_or("matching")
            ),
            Colors::ERROR,
        )
    } else if !node
        .prerequisites
        .iter()
        .all(|id| research_state.is_unlocked(id))
    {
        ("Prerequisites not met".to_string(), Colors::ERROR)
    } else if research_tree.can_research_on(
        node_id,
        &research_state.unlocked,
        data_available,
        active_condition,
    ) {
        ("Tap node to research".to_string(), Colors::SUCCESS)
    } else {
        (
            "Tap node to select (insufficient Data)".to_string(),
            Colors::WARNING,
        )
    };
    draw_ui_text(
        &truncate_text_to_width(&status.0, panel_w * 0.46, 11.0),
        split_x,
        detail_y + 18.0,
        11.0,
        status.1,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_mobile_node_info(
    data: &crate::data::GameData,
    research_state: &ResearchState,
    research_tree: &ResearchTree,
    node_id: &str,
    heading: &str,
    panel_x: f32,
    y: f32,
    panel_w: f32,
    data_available: f32,
    active_condition: Option<&str>,
) {
    let Some(node) = research_tree.get_node(node_id) else {
        return;
    };
    let text_w = (panel_w - 24.0).max(40.0);
    draw_ui_text(heading, panel_x + 12.0, y, 12.0, Colors::TEXT_DIM);
    draw_ui_text(
        &truncate_text_to_width(&node.name, text_w, 14.0),
        panel_x + 12.0,
        y + 18.0,
        14.0,
        Colors::TEXT,
    );
    draw_ui_text(
        &truncate_text_to_width(&node.description, text_w, 11.0),
        panel_x + 12.0,
        y + 38.0,
        11.0,
        Colors::TEXT_DIM,
    );

    let mut detail_y = y + 58.0;
    for (text, color) in node_effects(data, node_id) {
        draw_ui_text(
            &truncate_text_to_width(&text, text_w, 11.0),
            panel_x + 12.0,
            detail_y,
            11.0,
            color,
        );
        detail_y += 15.0;
    }
    let (status, color) = if research_state.is_unlocked(node_id) {
        ("UNLOCKED".to_string(), Colors::SUCCESS)
    } else if let Some(status) = active_research_status(research_state, node_id, node) {
        (status, Colors::WARNING)
    } else if let Some(status) = parked_research_status(research_state, node_id, node) {
        (
            format!("Cost {:.0} Data - {}", node.data_cost, status),
            Colors::WARNING,
        )
    } else {
        let status = if node
            .planet_condition
            .as_deref()
            .is_some_and(|required| active_condition != Some(required))
        {
            format!(
                "Requires {} world",
                node.planet_condition.as_deref().unwrap_or("matching")
            )
        } else if !node
            .prerequisites
            .iter()
            .all(|id| research_state.is_unlocked(id))
        {
            "Prerequisites not met".to_string()
        } else if research_tree.can_research_on(
            node_id,
            &research_state.unlocked,
            data_available,
            active_condition,
        ) {
            "Tap node to research".to_string()
        } else {
            "Tap node to select (insufficient Data)".to_string()
        };
        let color = if status.starts_with("Tap node") {
            if research_tree.can_research_on(
                node_id,
                &research_state.unlocked,
                data_available,
                active_condition,
            ) {
                Colors::SUCCESS
            } else {
                Colors::WARNING
            }
        } else if status.starts_with("Requires") || status.starts_with("Prerequisites") {
            Colors::ERROR
        } else {
            Colors::WARNING
        };
        (
            format!("Cost {:.0} Data - {}", node.data_cost, status),
            color,
        )
    };
    draw_ui_text(
        &truncate_text_to_width(&status, text_w, 11.0),
        panel_x + 12.0,
        detail_y + 2.0,
        11.0,
        color,
    );
}

#[cfg(test)]
mod tests;
