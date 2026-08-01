//! Main grid gameplay screen

mod entity_render;
mod format;
mod hud;
mod input;
mod metrics;
mod terrain_render;

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::directives::Directive;
use crate::state::PlanetState;
use crate::ui::color_from_rgba;
use macroquad::prelude::*;
use macroquad_toolkit::math::{lerp, pulse01_at};

use metrics::{is_cursor_over_ui, screen_to_grid, HudMetrics};

/// Actions from the planetary view
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlanetaryAction {
    None,
    OpenResearch,
    OpenSeedShip,
    OpenInterplanetary,
    OpenMenu,
}

/// Render the planetary grid view
pub fn render_planetary_view(
    state: &mut PlanetState,
    textures: &GameTextures,
    directive: &Directive,
    theme: &UiTheme,
) -> PlanetaryAction {
    clear_background(color_from_rgba(&theme.colors.background));

    let screen_w = screen_width();
    let screen_h = screen_height();
    let pulse = pulse01_at(state.time_played, 2.5);
    let global_pulse = lerp(0.8, 1.0, pulse01_at(state.time_played, 2.0));
    let time = state.time_played as f32;

    // Move the camera first, then lay out the frame around where it ended up,
    // so drawing and hit-testing agree about which tile is under the cursor.
    let (mouse_x, mouse_y) = mouse_position();
    let layout = HudMetrics::for_screen(theme, screen_w, screen_h, state.camera);
    let cursor_over_ui = is_cursor_over_ui(mouse_x, mouse_y, screen_w, screen_h, layout);
    input::handle_camera(state, layout, cursor_over_ui, screen_w, screen_h);

    let metrics = HudMetrics::for_screen(theme, screen_w, screen_h, state.camera);
    let hovered_pos = if cursor_over_ui {
        None
    } else {
        screen_to_grid(mouse_x, mouse_y, metrics)
            .filter(|pos| pos.in_bounds(state.grid.width, state.grid.height))
    };

    terrain_render::draw_planetary_background(screen_w, screen_h, time);
    terrain_render::draw_grid_tiles(state, textures, metrics, hovered_pos, pulse, global_pulse);
    entity_render::draw_drones(state, metrics, time);
    entity_render::draw_particles(state, metrics);

    let ui_action = hud::draw_ui_panels(
        state,
        screen_w,
        screen_h,
        hovered_pos,
        directive,
        textures,
        theme,
        metrics,
    );

    if ui_action != PlanetaryAction::None {
        ui_action
    } else {
        input::handle_input(state, hovered_pos)
    }
}
