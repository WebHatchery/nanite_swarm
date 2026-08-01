//! Neural network research interface

use crate::engine::{describe_modifier, ResearchNode, ResearchState, ResearchTree};
use crate::state::{StatReading, StatUnit};
use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::math::pulse01;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

const MAX_NODE_RADIUS: f32 = 25.0;
const GRID_SCALE: f32 = 100.0;
const HEADER_HEIGHT: f32 = 72.0;
/// Below this the nodes are too small to read, whatever the tree wants.
const MIN_NODE_RADIUS: f32 = 13.0;

/// Where the declared node positions land on screen.
///
/// Fitted to whatever `research.json` declares rather than pinned to a fixed
/// scale, because a tree that grows another row deep should not simply fall off
/// the bottom of the screen - which is what the last two nodes were already
/// doing at 720p, unclickable and invisible.
struct TreeLayout {
    origin_x: f32,
    origin_y: f32,
    scale: f32,
}

impl TreeLayout {
    fn fit(nodes: &[ResearchNode], area: Rect) -> Self {
        let mut min = (f32::MAX, f32::MAX);
        let mut max = (f32::MIN, f32::MIN);
        for node in nodes {
            min = (min.0.min(node.position.0), min.1.min(node.position.1));
            max = (max.0.max(node.position.0), max.1.max(node.position.1));
        }
        if nodes.is_empty() {
            return Self {
                origin_x: area.x + area.w * 0.5,
                origin_y: area.y + area.h * 0.5,
                scale: GRID_SCALE,
            };
        }

        // Room for the circle itself, the name above it and the cost below.
        let margin = MAX_NODE_RADIUS + 24.0;
        let span = ((max.0 - min.0).max(0.001), (max.1 - min.1).max(0.001));
        let scale = ((area.w - margin * 2.0) / span.0)
            .min((area.h - margin * 2.0) / span.1)
            .clamp(1.0, GRID_SCALE);
        let used = (span.0 * scale, span.1 * scale);

        Self {
            origin_x: area.x + (area.w - used.0) * 0.5 - min.0 * scale,
            origin_y: area.y + (area.h - used.1) * 0.5 + max.1 * scale,
            scale,
        }
    }

    fn to_screen(&self, position: (f32, f32)) -> (f32, f32) {
        (
            self.origin_x + position.0 * self.scale,
            self.origin_y - position.1 * self.scale,
        )
    }

    /// Nodes shrink with the tree, so a denser one does not draw itself as a
    /// pile of overlapping circles.
    fn node_radius(&self) -> f32 {
        (MAX_NODE_RADIUS * self.scale / GRID_SCALE).clamp(MIN_NODE_RADIUS, MAX_NODE_RADIUS)
    }
}

/// Actions from the research view
#[derive(Debug, Clone, PartialEq)]
pub enum ResearchAction {
    None,
    Close,
    StartResearch(String),
}

/// Render the research neural network view
pub fn render_research_view(
    research_state: &ResearchState,
    research_tree: &ResearchTree,
    data_available: f32,
    research_locked: bool,
    sheet: &[StatReading],
) -> ResearchAction {
    clear_background(Colors::BACKGROUND);

    let screen_w = screen_width();
    let screen_h = screen_height();
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

    let left_panel_x = 16.0;
    let left_panel_y = HEADER_HEIGHT + 12.0;
    let left_panel_w = 280.0;
    let left_panel_h = screen_h - left_panel_y - 80.0;
    draw_panel(left_panel_x, left_panel_y, left_panel_w, left_panel_h);
    draw_ui_text(
        "Research Intel",
        left_panel_x + 12.0,
        left_panel_y + 28.0,
        16.0,
        Colors::PRIMARY,
    );

    let right_panel_w = 260.0;
    let right_panel_x = screen_w - right_panel_w - 16.0;
    let right_panel_y = HEADER_HEIGHT + 12.0;
    let right_panel_h = screen_h - right_panel_y - 80.0;
    draw_panel(right_panel_x, right_panel_y, right_panel_w, right_panel_h);
    draw_ui_text(
        "Legend",
        right_panel_x + 12.0,
        right_panel_y + 28.0,
        16.0,
        Colors::PRIMARY,
    );

    let mut left_text_y = left_panel_y + 56.0;
    if research_locked {
        draw_ui_text(
            "Research Locked (power collapse)",
            left_panel_x + 12.0,
            left_text_y,
            12.0,
            Colors::ERROR,
        );
        left_text_y += 24.0;
    }
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

    // The tree gets whatever the two side panels leave it.
    let tree_area = Rect::new(
        left_panel_x + left_panel_w + 16.0,
        HEADER_HEIGHT + 12.0,
        right_panel_x - (left_panel_x + left_panel_w) - 32.0,
        screen_h - HEADER_HEIGHT - 72.0,
    );
    let layout = TreeLayout::fit(&research_tree.nodes, tree_area);
    let node_radius = layout.node_radius();

    // Get mouse position
    let (mouse_x, mouse_y) = mouse_position();
    let mut hovered_node: Option<&str> = None;

    // Draw connections first (behind nodes)
    for (from, to) in research_tree.get_connections() {
        let from_unlocked = research_state.is_unlocked(&from.id);
        let to_unlocked = research_state.is_unlocked(&to.id);

        let (from_x, from_y) = layout.to_screen(from.position);
        let (to_x, to_y) = layout.to_screen(to.position);

        let line_color = if from_unlocked && to_unlocked {
            Colors::PRIMARY
        } else if from_unlocked {
            Color::new(0.0, 0.6, 0.8, 0.7)
        } else {
            Color::new(0.25, 0.25, 0.3, 0.7)
        };

        draw_line(from_x, from_y, to_x, to_y, 2.0, line_color);
        draw_line(
            from_x,
            from_y,
            to_x,
            to_y,
            1.0,
            Color::new(0.6, 0.8, 1.0, 0.15),
        );
    }

    // Draw nodes
    for node in &research_tree.nodes {
        let (node_x, node_y) = layout.to_screen(node.position);

        let is_unlocked = research_state.is_unlocked(&node.id);
        let can_select = research_tree.can_select(&node.id, &research_state.unlocked);
        let can_research_now =
            research_tree.can_research(&node.id, &research_state.unlocked, data_available);
        let is_current = research_state.current_research.as_ref() == Some(&node.id);

        // Check if mouse is hovering
        let dist = ((mouse_x - node_x).powi(2) + (mouse_y - node_y).powi(2)).sqrt();
        let is_hovered = dist < node_radius;
        if is_hovered {
            hovered_node = Some(&node.id);
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

                if research_tree.can_select(node_id, &research_state.unlocked) {
                    if research_tree.can_research(node_id, &research_state.unlocked, data_available)
                    {
                        draw_ui_text(
                            "Click to research",
                            left_panel_x + 12.0,
                            y,
                            12.0,
                            Colors::SUCCESS,
                        );
                    } else {
                        draw_ui_text(
                            "Click to select (insufficient Data)",
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
    } else {
        draw_ui_text(
            "Hover a node to inspect.",
            left_panel_x + 12.0,
            left_text_y,
            12.0,
            Colors::TEXT_DIM,
        );
    }

    // Legend
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

    // Instructions
    draw_ui_text(
        "Press ESC to return",
        20.0,
        screen_h - 20.0,
        Dimensions::FONT_SIZE_SMALL,
        Colors::TEXT_DIM,
    );

    // Handle input
    if is_key_pressed(KeyCode::Escape) {
        return ResearchAction::Close;
    }

    // Click to research
    if is_mouse_button_pressed(MouseButton::Left) {
        if let Some(node_id) = hovered_node {
            if research_tree.can_select(node_id, &research_state.unlocked) {
                return ResearchAction::StartResearch(node_id.to_string());
            }
        }
    }

    ResearchAction::None
}

/// What a node does, as lines for the hover panel: every stat it moves, then
/// anything it lets the swarm build.
///
/// A node with neither says so rather than showing nothing, because "no
/// effect" and "the panel forgot to draw" look identical otherwise.
fn node_effects(node_id: &str) -> Vec<(String, Color)> {
    let data = crate::data::game_data();
    let mut lines = Vec::new();

    if let Some(def) = data.research.nodes.iter().find(|node| node.id == node_id) {
        for modifier in &def.modifiers {
            let Some(summary) = describe_modifier(modifier) else {
                continue;
            };
            let color = if summary.is_gain {
                Colors::SUCCESS
            } else {
                Colors::WARNING
            };
            lines.push((format!("{} {}", summary.label, summary.change), color));
        }
    }

    for building in &data.buildings {
        if building.unlocked_by.as_deref() == Some(node_id) {
            lines.push((format!("Unlocks {}", building.name), Colors::PRIMARY));
        }
    }

    if lines.is_empty() {
        lines.push(("No direct effect".to_string(), Colors::TEXT_DIM));
    }
    lines
}

/// The swarm's stat sheet: what every stat is actually worth on this world,
/// and what it started at where something has moved it.
///
/// This is the other half of the node panel. That says what a tech would add;
/// this says what the additions have come to.
fn draw_swarm_sheet(sheet: &[StatReading], panel_x: f32, panel_y: f32, panel_w: f32) {
    draw_ui_text("Swarm", panel_x + 12.0, panel_y, 14.0, Colors::PRIMARY);
    let mut y = panel_y + 24.0;
    for reading in sheet {
        let unit = StatUnit::of(reading.stat);
        let color = if !reading.is_changed() {
            Colors::TEXT_DIM
        } else if reading.is_gain() {
            Colors::SUCCESS
        } else {
            Colors::WARNING
        };
        draw_ui_text(
            reading.stat.label(),
            panel_x + 12.0,
            y,
            11.0,
            Colors::TEXT_DIM,
        );
        let value = unit.format(reading.value);
        let width = measure_ui_text(&value, None, 11, 1.0).width;
        draw_ui_text(&value, panel_x + panel_w - 12.0 - width, y, 11.0, color);
        y += 17.0;
    }
}
