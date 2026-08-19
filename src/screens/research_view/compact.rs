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

pub(super) fn update_compact_viewport(viewport: &mut ResearchViewport, tree: Rect) {
    let mouse = Vec2::from(mouse_position());
    if is_mouse_button_pressed(MouseButton::Left) && tree.contains(mouse) {
        viewport.drag_distance = 0.0;
    }
    if is_mouse_button_down(MouseButton::Left) && tree.contains(mouse) {
        let delta = mouse_delta_position();
        viewport.pan += delta;
        viewport.drag_distance += delta.length();
    }
    let (_, wheel_y) = mouse_wheel();
    if wheel_y.abs() > f32::EPSILON && tree.contains(mouse) {
        viewport.zoom = (viewport.zoom + wheel_y.signum() * 0.1).clamp(1.0, 2.4);
    }
    viewport.pan.x = viewport.pan.x.clamp(-tree.w, tree.w);
    viewport.pan.y = viewport.pan.y.clamp(-tree.h * 2.5, tree.h * 2.5);
}

pub(super) fn draw_compact_viewport_controls(viewport: &mut ResearchViewport, screen_w: f32) {
    let y = HEADER_HEIGHT + 4.0;
    let mut x = screen_w - 178.0;
    if draw_button_sized(x, y, 32.0, 28.0, "-") {
        viewport.zoom = (viewport.zoom - 0.15).max(1.0);
    }
    x += 38.0;
    if draw_button_sized(x, y, 32.0, 28.0, "+") {
        viewport.zoom = (viewport.zoom + 0.15).min(2.4);
    }
    x += 38.0;
    if draw_button_sized(x, y, 68.0, 28.0, "Center") {
        viewport.pan = Vec2::ZERO;
        viewport.zoom = 1.45;
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
    for (text, color) in node_effects(node_id) {
        draw_ui_text(&text, split_x, detail_y, 11.0, color);
        detail_y += 16.0;
    }
    if research_state.is_unlocked(node_id) {
        draw_ui_text("UNLOCKED", split_x, detail_y + 2.0, 11.0, Colors::SUCCESS);
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
    draw_ui_text(&status.0, split_x, detail_y + 18.0, 11.0, status.1);
}

#[cfg(test)]
mod tests;
