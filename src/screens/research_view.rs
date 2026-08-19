//! Neural network research interface

use crate::engine::{ResearchState, ResearchTree};
use crate::state::StatReading;
use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::math::pulse01;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, truncate_text_to_width, Pointer};

mod effects;
mod layout;
mod sheet;
use effects::node_effects;
use layout::{ResearchLayout, TreeLayout};
use sheet::draw_swarm_sheet;

const HEADER_HEIGHT: f32 = 72.0;

/// Actions from the research view
#[derive(Debug, Clone, PartialEq)]
pub enum ResearchAction {
    None,
    Close,
    StartResearch(String),
}

#[derive(Debug, Clone)]
pub struct ResearchViewport {
    pan: Vec2,
    zoom: f32,
    drag_distance: f32,
}

impl Default for ResearchViewport {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: 1.45,
            drag_distance: 0.0,
        }
    }
}

/// Render the research neural network view
pub fn render_research_view(
    research_state: &ResearchState,
    research_tree: &ResearchTree,
    data_available: f32,
    research_locked: bool,
    sheet: &[StatReading],
    active_condition: Option<&str>,
    viewport: &mut ResearchViewport,
) -> ResearchAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
    let view = ResearchLayout::for_screen(screen_w, screen_h, HEADER_HEIGHT);
    let pulse = pulse01(2.0);

    // Background neural haze
    for i in 0..120u32 {
        let x = (i as f32 * 37.7).sin().abs() * screen_w;
        let y = (i as f32 * 19.3).cos().abs() * screen_h;
        draw_circle(
            x,
            y,
            1.0 + (i % 3) as f32 * 0.4,
            Color::new(0.0, 0.7, 0.9, 0.06),
        );
    }

    // Header
    draw_panel(0.0, 0.0, screen_w, HEADER_HEIGHT);
    draw_ui_text("Neural Network", 18.0, 30.0, 18.0, Colors::PRIMARY);
    draw_ui_text(
        &format!("Data {:.0}", data_available),
        18.0,
        52.0,
        12.0,
        Colors::TEXT_DIM,
    );
    if draw_button_sized(screen_w - 110.0, 18.0, 80.0, 34.0, "Back") {
        return ResearchAction::Close;
    }

    if view.compact {
        update_compact_viewport(viewport, view.tree);
        draw_compact_viewport_controls(viewport, screen_w);
    }

    let left_panel_x = view.intel.x;
    let left_panel_y = view.intel.y;
    let left_panel_w = view.intel.w;
    let left_panel_h = view.intel.h;
    draw_panel(left_panel_x, left_panel_y, left_panel_w, left_panel_h);
    draw_ui_text(
        "Research Intel",
        left_panel_x + 12.0,
        left_panel_y + 28.0,
        16.0,
        Colors::PRIMARY,
    );

    let right_panel_w = view.legend.w;
    let right_panel_x = view.legend.x;
    let right_panel_y = view.legend.y;
    if !view.compact {
        draw_panel(right_panel_x, right_panel_y, right_panel_w, view.legend.h);
        draw_ui_text(
            "Legend",
            right_panel_x + 12.0,
            right_panel_y + 28.0,
            16.0,
            Colors::PRIMARY,
        );
    } else {
        draw_compact_legend(18.0, HEADER_HEIGHT + 22.0);
        draw_ui_text(
            "Drag tree; tap - / + to zoom",
            screen_w - 390.0,
            HEADER_HEIGHT + 22.0,
            11.0,
            Colors::TEXT_DIM,
        );
    }

    let mut left_text_y = left_panel_y + 56.0;
    if research_locked {
        let (locked_x, locked_y) = if view.compact {
            (left_panel_x + left_panel_w - 238.0, left_panel_y + 28.0)
        } else {
            (left_panel_x + 12.0, left_text_y)
        };
        draw_ui_text(
            "Research Locked (power collapse)",
            locked_x,
            locked_y,
            12.0,
            Colors::ERROR,
        );
        if !view.compact {
            left_text_y += 24.0;
        }
    }
    if !view.compact {
        if let Some(current) = &research_state.current_research {
            if let Some(node) = research_tree.get_node(current) {
                let progress = research_state.research_progress.min(node.data_cost);
                let pct = if node.data_cost > 0.0 {
                    progress / node.data_cost
                } else {
                    1.0
                };
                draw_ui_text(
                    "Active Research",
                    left_panel_x + 12.0,
                    left_text_y,
                    12.0,
                    Colors::TEXT_DIM,
                );
                draw_ui_text(
                    &node.name,
                    left_panel_x + 12.0,
                    left_text_y + 18.0,
                    14.0,
                    Colors::TEXT,
                );
                draw_rectangle(
                    left_panel_x + 12.0,
                    left_text_y + 32.0,
                    left_panel_w - 24.0,
                    10.0,
                    Colors::SURFACE_DARK,
                );
                draw_rectangle(
                    left_panel_x + 12.0,
                    left_text_y + 32.0,
                    (left_panel_w - 24.0) * pct,
                    10.0,
                    Colors::PRIMARY,
                );
                draw_rectangle_lines(
                    left_panel_x + 12.0,
                    left_text_y + 32.0,
                    left_panel_w - 24.0,
                    10.0,
                    1.0,
                    Colors::PANEL_BORDER,
                );
                left_text_y += 60.0;
            }
        } else {
            draw_ui_text(
                "No research selected.",
                left_panel_x + 12.0,
                left_text_y,
                12.0,
                Colors::TEXT_DIM,
            );
            left_text_y += 24.0;
        }
    }

    let (zoom, pan) = if view.compact {
        (viewport.zoom, (viewport.pan.x, viewport.pan.y))
    } else {
        (1.0, (0.0, 0.0))
    };
    let layout = if view.compact {
        TreeLayout::fit_with_min_scale(&research_tree.nodes, view.tree, 60.0)
    } else {
        TreeLayout::fit(&research_tree.nodes, view.tree)
    }
    .with_view(zoom, pan);
    let node_radius = layout.node_radius();

    let pointer = Pointer::read(|position| position);
    let mut hovered_node: Option<&str> = None;
    let mut hovered_distance = f32::MAX;
    let mut activated_node: Option<&str> = None;
    let mut activated_distance = f32::MAX;

    // Draw connections first (behind nodes)
    for (from, to) in research_tree.get_connections() {
        let from_unlocked = research_state.is_unlocked(&from.id);
        let to_unlocked = research_state.is_unlocked(&to.id);

        let (from_x, from_y) = layout.to_screen(from.position);
        let (to_x, to_y) = layout.to_screen(to.position);
        let Some((from_point, to_point)) =
            clip_line(view.tree, vec2(from_x, from_y), vec2(to_x, to_y))
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

    // Draw nodes
    for node in &research_tree.nodes {
        let (node_x, node_y) = layout.to_screen(node.position);
        let safe_tree = Rect::new(
            view.tree.x + 28.0,
            view.tree.y + 28.0,
            view.tree.w - 56.0,
            view.tree.h - 56.0,
        );
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
            hovered_node = Some(&node.id);
            hovered_distance = pointer_distance;
        }
        if pointer.released_on(target)
            && viewport.drag_distance < 6.0
            && pointer_distance < activated_distance
        {
            activated_node = Some(&node.id);
            activated_distance = pointer_distance;
        }

        // Node colors
        let (fill_color, border_color) = if is_unlocked {
            (Colors::PRIMARY, Colors::PRIMARY)
        } else if is_current {
            (Color::new(0.0, 0.5, 0.7, 1.0), Colors::WARNING)
        } else if can_research_now {
            (Colors::SURFACE, Colors::SUCCESS)
        } else if can_select {
            (Colors::SURFACE, Colors::PRIMARY_SOFT)
        } else {
            (Colors::SURFACE, Colors::SECONDARY)
        };

        // Draw glow for unlocked nodes
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

        // Draw node
        draw_circle(node_x, node_y, node_radius, fill_color);
        draw_circle_lines(node_x, node_y, node_radius, 2.0, border_color);

        // Progress ring for current research
        if is_current && node.data_cost > 0.0 {
            let pct = (research_state.research_progress / node.data_cost).clamp(0.0, 1.0);
            let segments = 24;
            for i in 0..segments {
                let t0 = (i as f32 / segments as f32) * std::f32::consts::TAU;
                let t1 = ((i + 1) as f32 / segments as f32) * std::f32::consts::TAU;
                if (i as f32 / segments as f32) <= pct {
                    let r = node_radius + 6.0;
                    let x0 = node_x + t0.cos() * r;
                    let y0 = node_y + t0.sin() * r;
                    let x1 = node_x + t1.cos() * r;
                    let y1 = node_y + t1.sin() * r;
                    draw_line(x0, y0, x1, y1, 2.0, Colors::WARNING);
                }
            }
        }

        // Hover effect
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

        // Draw abbreviated name
        let abbrev = &node.name[..node.name.len().min(7)];
        let text_size = measure_ui_text(abbrev, None, 12, 1.0);
        let text_color = if is_unlocked {
            Colors::BACKGROUND
        } else {
            Colors::TEXT
        };
        draw_ui_text(
            abbrev,
            node_x - text_size.width / 2.0,
            node_y + 4.0,
            12.0,
            text_color,
        );

        // Draw cost below if not unlocked
        if !is_unlocked && node.data_cost > 0.0 {
            let cost_str = format!("{:.0}", node.data_cost);
            let cost_color = if can_research_now {
                Colors::SUCCESS
            } else {
                Colors::TEXT_DIM
            };
            draw_ui_text(
                &cost_str,
                node_x - 10.0,
                node_y + node_radius + 15.0,
                12.0,
                cost_color,
            );
        }
    }

    // Info panel. With nothing under the cursor it falls back to whatever is
    // being researched, so the panel is worth having between hovers.
    let inspected = hovered_node.or(research_state.current_research.as_deref());
    if let Some(node_id) = inspected {
        if let Some(node) = research_tree.get_node(node_id) {
            let heading = if hovered_node.is_some() {
                "Hovered Node"
            } else {
                "Researching"
            };
            if view.compact {
                draw_compact_node_info(
                    research_state,
                    research_tree,
                    node_id,
                    heading,
                    left_panel_x,
                    left_text_y,
                    left_panel_w,
                    data_available,
                    active_condition,
                );
            } else {
                draw_ui_text(
                    heading,
                    left_panel_x + 12.0,
                    left_text_y,
                    12.0,
                    Colors::TEXT_DIM,
                );
                draw_ui_text(
                    &node.name,
                    left_panel_x + 12.0,
                    left_text_y + 18.0,
                    14.0,
                    Colors::TEXT,
                );
                draw_ui_text(
                    &node.description,
                    left_panel_x + 12.0,
                    left_text_y + 36.0,
                    12.0,
                    Colors::TEXT_DIM,
                );

                // What it actually does, under what it says it does.
                let mut y = left_text_y + 58.0;
                for (text, color) in node_effects(node_id) {
                    draw_ui_text(&text, left_panel_x + 12.0, y, 12.0, color);
                    y += 16.0;
                }
                y += 4.0;

                if !research_state.is_unlocked(node_id) {
                    let cost_text = format!("Cost {:.0} Data", node.data_cost);
                    draw_ui_text(&cost_text, left_panel_x + 12.0, y, 12.0, Colors::ACCENT);
                    y += 18.0;

                    if research_tree.can_select_on(
                        node_id,
                        &research_state.unlocked,
                        active_condition,
                    ) {
                        if research_tree.can_research_on(
                            node_id,
                            &research_state.unlocked,
                            data_available,
                            active_condition,
                        ) {
                            draw_ui_text(
                                "Tap node to research",
                                left_panel_x + 12.0,
                                y,
                                12.0,
                                Colors::SUCCESS,
                            );
                        } else {
                            draw_ui_text(
                                "Tap node to select (insufficient Data)",
                                left_panel_x + 12.0,
                                y,
                                11.0,
                                Colors::WARNING,
                            );
                        }
                    } else if !node
                        .prerequisites
                        .iter()
                        .all(|p| research_state.is_unlocked(p))
                    {
                        draw_ui_text(
                            "Prerequisites not met",
                            left_panel_x + 12.0,
                            y,
                            12.0,
                            Colors::ERROR,
                        );
                    } else {
                        draw_ui_text(
                            "Not enough Data",
                            left_panel_x + 12.0,
                            y,
                            12.0,
                            Colors::WARNING,
                        );
                    }
                } else {
                    draw_ui_text("UNLOCKED", left_panel_x + 12.0, y, 12.0, Colors::SUCCESS);
                }
            }
        }
    } else {
        draw_ui_text(
            "Point at or tap a node to inspect.",
            left_panel_x + 12.0,
            left_text_y,
            12.0,
            Colors::TEXT_DIM,
        );
    }

    // Legend and stat sheet stay in the sidebar on roomy screens. The compact
    // legend is above the full-width tree, while the lower drawer is reserved
    // for the active/inspected node and remains a large touch target.
    if !view.compact {
        draw_ui_text(
            "Unlocked",
            right_panel_x + 12.0,
            right_panel_y + 56.0,
            12.0,
            Colors::TEXT_DIM,
        );
        draw_circle(
            right_panel_x + 18.0,
            right_panel_y + 74.0,
            6.0,
            Colors::PRIMARY,
        );
        draw_ui_text(
            "In Progress",
            right_panel_x + 12.0,
            right_panel_y + 98.0,
            12.0,
            Colors::TEXT_DIM,
        );
        draw_circle(
            right_panel_x + 18.0,
            right_panel_y + 116.0,
            6.0,
            Colors::WARNING,
        );
        draw_ui_text(
            "Available",
            right_panel_x + 12.0,
            right_panel_y + 140.0,
            12.0,
            Colors::TEXT_DIM,
        );
        draw_circle(
            right_panel_x + 18.0,
            right_panel_y + 158.0,
            6.0,
            Colors::SUCCESS,
        );
        draw_ui_text(
            "Locked",
            right_panel_x + 12.0,
            right_panel_y + 182.0,
            12.0,
            Colors::TEXT_DIM,
        );
        draw_circle(
            right_panel_x + 18.0,
            right_panel_y + 200.0,
            6.0,
            Colors::SECONDARY,
        );

        draw_swarm_sheet(sheet, right_panel_x, right_panel_y + 232.0, right_panel_w);
    }

    // Instructions
    draw_ui_text(
        "Tap Back to return",
        20.0,
        screen_h - 20.0,
        Dimensions::FONT_SIZE_SMALL,
        Colors::TEXT_DIM,
    );

    // Handle input
    if is_key_pressed(KeyCode::Escape) {
        return ResearchAction::Close;
    }

    if let Some(node_id) = activated_node {
        if research_tree.can_select_on(node_id, &research_state.unlocked, active_condition) {
            return ResearchAction::StartResearch(node_id.to_string());
        }
    }

    ResearchAction::None
}

fn draw_compact_legend(x: f32, y: f32) {
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

fn update_compact_viewport(viewport: &mut ResearchViewport, tree: Rect) {
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

fn draw_compact_viewport_controls(viewport: &mut ResearchViewport, screen_w: f32) {
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

fn clip_line(rect: Rect, from: Vec2, to: Vec2) -> Option<(Vec2, Vec2)> {
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
fn draw_compact_node_info(
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
        .all(|prerequisite| research_state.is_unlocked(prerequisite))
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
