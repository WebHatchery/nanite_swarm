//! Neural research connections, nodes, and their pointer interaction.

use crate::engine::{ResearchState, ResearchTree};
use crate::ui::Colors;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, Pointer};

use super::compact::clip_line;
use super::layout::TreeLayout;

/// The node identities the coordinator needs after the tree has been drawn.
/// Strings keep the result independent from the tree borrow while the info
/// panel decides whether a hover or tap should win over the active job.
pub(super) struct NodeInteraction {
    pub(super) hovered_node: Option<String>,
    pub(super) activated_node: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_research_nodes(
    research_tree: &ResearchTree,
    research_state: &ResearchState,
    data_available: f32,
    active_condition: Option<&str>,
    tutorial_target: Option<&str>,
    tree_area: Rect,
    layout: &TreeLayout,
    pointer: Pointer,
    drag_distance: f32,
    pulse: f32,
) -> NodeInteraction {
    draw_connections(research_tree, research_state, tree_area, layout);

    let node_radius = layout.node_radius();
    let safe_tree = Rect::new(
        tree_area.x + 28.0,
        tree_area.y + 28.0,
        tree_area.w - 56.0,
        tree_area.h - 56.0,
    );
    let mut hovered_node: Option<String> = None;
    let mut hovered_distance = f32::MAX;
    let mut activated_node: Option<String> = None;
    let mut activated_distance = f32::MAX;

    for node in &research_tree.nodes {
        let (node_x, node_y) = layout.to_screen(node.position);
        if !safe_tree.contains(vec2(node_x, node_y)) {
            continue;
        }

        let is_unlocked = research_state.is_unlocked(&node.id);
        let can_select =
            research_tree.can_select_on(&node.id, &research_state.unlocked, active_condition);
        let can_research_now = research_tree.can_research_on(
            &node.id,
            &research_state.unlocked,
            data_available,
            active_condition,
        );
        let is_current = research_state.current_research.as_ref() == Some(&node.id);
        let has_parked_progress = !is_current && research_state.progress_for(&node.id) > 0.001;
        let is_tutorial_target = tutorial_target == Some(node.id.as_str()) && !is_unlocked;

        let target_radius = node_radius.max(22.0);
        let target = Rect::new(
            node_x - target_radius,
            node_y - target_radius,
            target_radius * 2.0,
            target_radius * 2.0,
        );
        let is_hovered = pointer.hovering_over(target);
        let pointer_distance = pointer.position.distance(vec2(node_x, node_y));
        if (is_hovered || pointer.pressing(target) || pointer.released_on(target))
            && pointer_distance < hovered_distance
        {
            hovered_node = Some(node.id.clone());
            hovered_distance = pointer_distance;
        }
        if pointer.released_on(target)
            && drag_distance < 6.0
            && pointer_distance < activated_distance
        {
            activated_node = Some(node.id.clone());
            activated_distance = pointer_distance;
        }

        let (fill_color, border_color) = if is_unlocked {
            (Colors::PRIMARY, Colors::PRIMARY)
        } else if is_current {
            (Color::new(0.0, 0.5, 0.7, 1.0), Colors::WARNING)
        } else if has_parked_progress {
            (Colors::SURFACE, Colors::WARNING)
        } else if can_research_now {
            (Colors::SURFACE, Colors::SUCCESS)
        } else if can_select {
            (Colors::SURFACE, Colors::PRIMARY_SOFT)
        } else {
            (Colors::SURFACE, Colors::SECONDARY)
        };

        if is_unlocked {
            let glow_outer = node_radius + 6.0 + pulse * 3.0;
            let glow_inner = node_radius + 3.0 + pulse * 1.5;
            draw_circle(
                node_x,
                node_y,
                glow_outer,
                Color::new(0.0, 0.85, 1.0, 0.18 + pulse * 0.08),
            );
            draw_circle(
                node_x,
                node_y,
                glow_inner,
                Color::new(0.0, 0.85, 1.0, 0.25 + pulse * 0.1),
            );
        }

        if is_tutorial_target {
            let target_radius = node_radius + 11.0 + pulse * 3.0;
            draw_circle_lines(node_x, node_y, target_radius, 3.0, Colors::ACCENT);
            draw_circle_lines(
                node_x,
                node_y,
                target_radius + 5.0,
                1.0,
                Color::new(1.0, 0.7, 0.2, 0.55),
            );
        }

        draw_circle(node_x, node_y, node_radius, fill_color);
        draw_circle_lines(node_x, node_y, node_radius, 2.0, border_color);
        draw_progress_ring(
            node,
            research_state,
            node_x,
            node_y,
            node_radius,
            is_current,
            has_parked_progress,
        );

        if is_hovered {
            draw_circle_lines(
                node_x,
                node_y,
                node_radius + 5.0 + pulse * 2.0,
                2.0,
                Colors::PRIMARY,
            );
            draw_ui_text(
                &node.name,
                node_x - 22.0,
                node_y - node_radius - 12.0,
                12.0,
                Colors::TEXT,
            );
        }

        let abbrev = &node.name[..node.name.len().min(7)];
        let text_size = measure_ui_text(abbrev, None, 12, 1.0);
        draw_ui_text(
            abbrev,
            node_x - text_size.width / 2.0,
            node_y + 4.0,
            12.0,
            if is_unlocked {
                Colors::BACKGROUND
            } else {
                Colors::TEXT
            },
        );

        if !is_unlocked && node.data_cost > 0.0 {
            let cost_str = format!("{:.0}", node.data_cost);
            draw_ui_text(
                &cost_str,
                node_x - 10.0,
                node_y + node_radius + 15.0,
                12.0,
                if can_research_now {
                    Colors::SUCCESS
                } else {
                    Colors::TEXT_DIM
                },
            );
        }
    }

    NodeInteraction {
        hovered_node,
        activated_node,
    }
}

fn draw_connections(
    research_tree: &ResearchTree,
    research_state: &ResearchState,
    tree_area: Rect,
    layout: &TreeLayout,
) {
    for (from, to) in research_tree.get_connections() {
        let from_unlocked = research_state.is_unlocked(&from.id);
        let to_unlocked = research_state.is_unlocked(&to.id);
        let (from_x, from_y) = layout.to_screen(from.position);
        let (to_x, to_y) = layout.to_screen(to.position);
        let Some((from_point, to_point)) =
            clip_line(tree_area, vec2(from_x, from_y), vec2(to_x, to_y))
        else {
            continue;
        };
        let line_color = if from_unlocked && to_unlocked {
            Colors::PRIMARY
        } else if from_unlocked {
            Color::new(0.0, 0.6, 0.8, 0.7)
        } else {
            Color::new(0.25, 0.25, 0.3, 0.7)
        };
        draw_line(
            from_point.x,
            from_point.y,
            to_point.x,
            to_point.y,
            2.0,
            line_color,
        );
        draw_line(
            from_point.x,
            from_point.y,
            to_point.x,
            to_point.y,
            1.0,
            Color::new(0.6, 0.8, 1.0, 0.15),
        );
    }
}

fn draw_progress_ring(
    node: &crate::engine::ResearchNode,
    research_state: &ResearchState,
    node_x: f32,
    node_y: f32,
    node_radius: f32,
    is_current: bool,
    has_parked_progress: bool,
) {
    if !(is_current || has_parked_progress) || node.data_cost <= 0.0 {
        return;
    }
    let progress = research_state.progress_for(&node.id);
    let pct = (progress / node.data_cost).clamp(0.0, 1.0);
    let ring_color = if is_current {
        Colors::WARNING
    } else {
        Colors::PRIMARY_SOFT
    };
    let segments = 24;
    for i in 0..segments {
        let fraction = i as f32 / segments as f32;
        if fraction > pct {
            continue;
        }
        let t0 = fraction * std::f32::consts::TAU;
        let t1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
        let radius = node_radius + 6.0;
        let x0 = node_x + t0.cos() * radius;
        let y0 = node_y + t0.sin() * radius;
        let x1 = node_x + t1.cos() * radius;
        let y1 = node_y + t1.sin() * radius;
        draw_line(x0, y0, x1, y1, 2.0, ring_color);
    }
}
