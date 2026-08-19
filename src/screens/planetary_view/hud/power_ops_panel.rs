//! Right sidebar: "POWER GRID" and "OPERATIONS" panels

use crate::data::UiTheme;
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_panel, draw_hud_progress_bar, draw_status_row};
use macroquad::prelude::*;
use macroquad_toolkit::input::is_hovered;
use macroquad_toolkit::ui::draw_ui_text;

use super::super::PlanetaryAction;
use super::{PanelColors, RightStackLayout};

pub(super) fn draw(
    state: &PlanetState,
    theme: &UiTheme,
    colors: &PanelColors,
    right: &RightStackLayout,
) -> PlanetaryAction {
    let text = colors.text;
    let dim = colors.dim;
    let primary_soft = colors.primary_soft;
    let success = colors.success;
    let error = colors.error;
    let power_color = colors.power;
    let right_x = right.right_x;
    let right_w = right.right_w;
    let power_y = right.power.y;
    let power_h = right.power.h;
    let ops_y = right.ops.y;
    let ops_h = right.ops.h;

    draw_hud_panel(theme, right.power, Some("POWER GRID"));
    let generation = state.power_generation();
    let consumption = state.power_consumption();
    let power_content_y = power_y + if power_h < 130.0 { 48.0 } else { 54.0 };
    let power_row_gap = if power_h < 130.0 { 17.0 } else { 18.0 };
    draw_status_row(
        theme,
        right_x + 12.0,
        power_content_y,
        right_w - 24.0,
        "Generation",
        &format!("+{:.1}/s", generation),
        success,
    );
    draw_status_row(
        theme,
        right_x + 12.0,
        power_content_y + power_row_gap,
        right_w - 24.0,
        "Consumption",
        &format!("-{:.1}/s", consumption),
        error,
    );
    draw_status_row(
        theme,
        right_x + 12.0,
        power_content_y + power_row_gap * 2.0,
        right_w - 24.0,
        "Net",
        &format!("{:+.1}/s", state.power_balance),
        power_color,
    );
    let battery_ratio = (state.battery_seconds / (4.0 * 60.0 * 60.0)).clamp(0.0, 1.0);
    let battery_y = power_y + power_h - 22.0;
    draw_ui_text("Battery", right_x + 12.0, battery_y + 10.0, 10.0, dim);
    draw_hud_progress_bar(
        theme,
        Rect::new(right_x + 86.0, battery_y, right_w - 108.0, 10.0),
        battery_ratio,
        primary_soft,
    );

    draw_hud_panel(theme, right.ops, Some("OPERATIONS"));
    let ops_content_y = ops_y + if ops_h < 110.0 { 48.0 } else { 54.0 };
    let ops_row_gap = if ops_h < 110.0 { 17.0 } else { 20.0 };
    let stalled = state.stalled_drone_count();
    let (drone_label, drone_color) = if stalled > 0 {
        (
            format!("{} ({} STALLED)", state.drones.total_count(), stalled),
            error,
        )
    } else {
        (format!("{}", state.drones.total_count()), text)
    };
    draw_status_row(
        theme,
        right_x + 12.0,
        ops_content_y,
        right_w - 24.0,
        "Drones",
        &drone_label,
        drone_color,
    );
    let starved = state.starved_factories().len();
    let structure_value = if starved > 0 {
        format!("{} ({} STARVED)", state.grid.total_buildings(), starved)
    } else {
        format!("{}", state.grid.total_buildings())
    };
    draw_status_row(
        theme,
        right_x + 12.0,
        ops_content_y + ops_row_gap,
        right_w - 24.0,
        "Structures",
        &structure_value,
        if starved > 0 { colors.warning } else { text },
    );
    // The one row on the stack that opens something: a count of achievements
    // with no way to see which ones is a count of nothing.
    let (achieved, total) = state.achievements_progress();
    let achievements_row = Rect::new(
        right_x + 8.0,
        ops_content_y + ops_row_gap * 2.0 - 12.0,
        right_w - 16.0,
        18.0,
    );
    let hovered = is_hovered(
        achievements_row.x,
        achievements_row.y,
        achievements_row.w,
        achievements_row.h,
    );
    if hovered {
        draw_rectangle(
            achievements_row.x,
            achievements_row.y,
            achievements_row.w,
            achievements_row.h,
            color_from_rgba(&theme.colors.panel_inner),
        );
    }
    draw_status_row(
        theme,
        right_x + 12.0,
        ops_content_y + ops_row_gap * 2.0,
        right_w - 24.0,
        "Achievements",
        &format!("{}/{}", achieved, total),
        if hovered {
            colors.primary
        } else {
            primary_soft
        },
    );

    if hovered && is_mouse_button_pressed(MouseButton::Left) {
        return PlanetaryAction::OpenRecords;
    }
    PlanetaryAction::None
}
