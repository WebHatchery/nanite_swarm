//! Neural network research interface

use crate::engine::{ResearchState, ResearchTree};
use crate::state::StatReading;
use crate::ui::{draw_button_sized, draw_panel, Colors, Dimensions};
use macroquad::prelude::*;
use macroquad_toolkit::math::pulse01;
use macroquad_toolkit::ui::{draw_ui_text, Pointer};

mod compact;
mod desktop;
mod effects;
mod layout;
mod nodes;
mod sheet;
mod sidebar;
use compact::*;
use layout::{compact_info_offset, ResearchLayout, TreeLayout};

const HEADER_HEIGHT: f32 = 72.0;
const DEFAULT_RESEARCH_ZOOM: f32 = 1.45;

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
    last_pointer: Option<Vec2>,
    drag_active: bool,
    center_pan: Vec2,
    center_zoom: f32,
    focus_pending: bool,
}

impl Default for ResearchViewport {
    fn default() -> Self {
        Self {
            pan: Vec2::ZERO,
            zoom: DEFAULT_RESEARCH_ZOOM,
            drag_distance: 0.0,
            last_pointer: None,
            drag_active: false,
            center_pan: Vec2::ZERO,
            center_zoom: DEFAULT_RESEARCH_ZOOM,
            focus_pending: true,
        }
    }
}

impl ResearchViewport {
    /// Reframe the network when the player opens it from another screen.
    pub fn reset_for_entry(&mut self) {
        *self = Self::default();
    }
}

pub struct ResearchViewContext<'a> {
    pub data: &'a crate::data::GameData,
    pub research_state: &'a ResearchState,
    pub research_tree: &'a ResearchTree,
    pub data_available: f32,
    pub research_locked: bool,
    pub tutorial_target: Option<&'a str>,
    pub sheet: &'a [StatReading],
    pub active_condition: Option<&'a str>,
    pub viewport: &'a mut ResearchViewport,
}

/// Render the research neural network view
pub fn render_research_view(context: ResearchViewContext<'_>) -> ResearchAction {
    let ResearchViewContext {
        data,
        research_state,
        research_tree,
        data_available,
        research_locked,
        tutorial_target,
        sheet,
        active_condition,
        viewport,
    } = context;
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
        focus_compact_viewport(
            viewport,
            research_tree,
            research_state,
            view.tree,
            active_condition,
            tutorial_target,
        );
        update_compact_viewport(viewport, view.tree);
        draw_compact_viewport_controls(viewport, screen_w, screen_w < 600.0);
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
    if view.compact {
        if screen_w < 600.0 {
            draw_compact_legend(18.0, HEADER_HEIGHT + 70.0, screen_w);
            draw_ui_text(
                "Drag tree with mouse or touch; tap - / + to zoom",
                18.0,
                HEADER_HEIGHT + 88.0,
                10.0,
                Colors::TEXT_DIM,
            );
        } else {
            draw_compact_legend(18.0, HEADER_HEIGHT + 22.0, screen_w);
            draw_ui_text(
                "Drag tree with mouse or touch; tap - / + to zoom",
                screen_w - 390.0,
                HEADER_HEIGHT + 22.0,
                11.0,
                Colors::TEXT_DIM,
            );
        }
    }

    let info_offset = if view.compact {
        compact_info_offset(screen_w < 600.0, view.intel.h)
    } else {
        56.0
    };
    let mut left_text_y = left_panel_y + info_offset;
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
        left_text_y = sidebar::draw_active_research(
            research_state,
            research_tree,
            left_panel_x,
            left_panel_w,
            left_text_y,
        );
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
    let interaction = nodes::draw_research_nodes(
        research_tree,
        research_state,
        data_available,
        active_condition,
        tutorial_target,
        view.tree,
        &layout,
        Pointer::read(|position| position),
        viewport.drag_distance,
        pulse,
    );
    let hovered_node = interaction.hovered_node.as_deref();
    let activated_node = interaction.activated_node.as_deref();

    // Info panel. With nothing under the cursor it falls back to whatever is
    // being researched, so the panel is worth having between hovers.
    let inspected = hovered_node.or(research_state.current_research.as_deref());
    if let Some(node_id) = inspected {
        if research_tree.get_node(node_id).is_some() {
            let heading = if hovered_node.is_some() {
                "Hovered Node"
            } else {
                "Researching"
            };
            if view.compact {
                draw_compact_node_info(CompactNodeInfoInput {
                    data,
                    research_state,
                    research_tree,
                    node_id,
                    heading,
                    panel_x: left_panel_x,
                    y: left_text_y,
                    panel_w: left_panel_w,
                    data_available,
                    active_condition,
                });
            } else {
                desktop::draw(desktop::DesktopNodeInfoInput {
                    data,
                    research_state,
                    research_tree,
                    node_id,
                    heading,
                    panel_x: left_panel_x,
                    y: left_text_y,
                    data_available,
                    active_condition,
                });
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
        sidebar::draw_desktop_legend(
            data,
            sheet,
            right_panel_x,
            right_panel_y,
            right_panel_w,
            view.legend.h,
        );
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
