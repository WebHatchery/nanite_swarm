//! Compact research-tree viewport controls and detail-panel dispatch.

use super::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, Pointer};

mod node_info;

/// Data and layout for the compact node detail drawer.
pub(super) struct CompactNodeInfoInput<'a> {
    pub(super) data: &'a crate::data::GameData,
    pub(super) research_state: &'a ResearchState,
    pub(super) research_tree: &'a ResearchTree,
    pub(super) node_id: &'a str,
    pub(super) heading: &'a str,
    pub(super) panel_x: f32,
    pub(super) y: f32,
    pub(super) panel_w: f32,
    pub(super) data_available: f32,
    pub(super) active_condition: Option<&'a str>,
}

pub(super) fn draw_compact_node_info(input: CompactNodeInfoInput<'_>) {
    node_info::draw(input);
}

const SHORT_PHONE_TREE_HEIGHT: f32 = 120.0;

fn compact_legend_labels(screen_w: f32) -> [&'static str; 4] {
    if screen_w < 300.0 {
        ["OPEN", "WORK", "READY", "LOCK"]
    } else {
        ["Unlocked", "In Progress", "Available", "Locked"]
    }
}

pub(super) fn draw_compact_legend(x: f32, y: f32, screen_w: f32) {
    let labels = compact_legend_labels(screen_w);
    let colors = [
        Colors::PRIMARY,
        Colors::WARNING,
        Colors::SUCCESS,
        Colors::SECONDARY,
    ];
    let font_size = if screen_w < 300.0 { 9 } else { 11 };
    let entry_gap = if screen_w < 300.0 { 28.0 } else { 36.0 };
    let mut entry_x = x;
    for (label, color) in labels.into_iter().zip(colors) {
        draw_circle(entry_x + 5.0, y - 4.0, 5.0, color);
        draw_ui_text(label, entry_x + 14.0, y, font_size as f32, Colors::TEXT_DIM);
        entry_x += measure_ui_text(label, None, font_size, 1.0).width + entry_gap;
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
    tutorial_target: Option<&str>,
) {
    if !viewport.focus_pending {
        return;
    }

    if tree.h < SHORT_PHONE_TREE_HEIGHT {
        // The authored tree is taller than a short phone's field at the
        // minimum layout scale. A 1x view keeps the active node inside the
        // pan limits instead of leaving it underneath the Intel drawer.
        viewport.zoom = 1.0;
    }

    let focus_position = tutorial_target_position(research_tree, research_state, tutorial_target)
        .or_else(|| {
            research_state
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
                .map(|node| node.position)
        });

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

fn tutorial_target_position(
    research_tree: &ResearchTree,
    research_state: &ResearchState,
    tutorial_target: Option<&str>,
) -> Option<(f32, f32)> {
    let target = tutorial_target.and_then(|id| research_tree.get_node(id))?;
    (!research_state.is_unlocked(&target.id)).then_some(target.position)
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

#[cfg(test)]
mod tests;
