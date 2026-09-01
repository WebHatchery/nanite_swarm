//! Individual building cards in the build palette.

use crate::assets::GameTextures;
use crate::data::UiTheme;
use crate::engine::BuildingType;
use crate::state::{MobilePanel, PlanetState};
use crate::ui::color_from_rgba;
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text, Pointer};

use super::super::super::format::{fit_text_to_width, format_power_delta};

#[allow(clippy::too_many_arguments)]
pub(super) fn draw(
    state: &mut PlanetState,
    theme: &UiTheme,
    textures: &GameTextures,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    building_type: BuildingType,
    interactive: bool,
    close_on_select: bool,
) -> f32 {
    let data_context = state.data.clone();
    let data = data_context.data();
    let rect = Rect::new(x, y, width, height);
    let pointer = Pointer::read(|position| position);
    let hovered = pointer.hovering_over(rect);
    let selected = state.selected_building == Some(building_type);
    let (mineral_cost, energy_cost) = building_type.cost(data);
    let can_afford = state.resources.can_afford(mineral_cost, energy_cost);
    let researched = state.is_building_researched(building_type);
    let unlocked = state.is_building_unlocked(building_type);
    let text = color_from_rgba(&theme.colors.text);
    let dim = color_from_rgba(&theme.colors.text_dim);
    let accent = color_from_rgba(&theme.colors.primary);
    let minerals_color = color_from_rgba(&theme.colors.minerals);
    let energy_color = color_from_rgba(&theme.colors.energy);

    let base_color = if !unlocked {
        color_from_rgba(&theme.colors.panel_deep)
    } else if selected {
        color_from_rgba(&theme.colors.panel_inner)
    } else if hovered {
        Color::new(0.035, 0.15, 0.19, 0.96)
    } else {
        color_from_rgba(&theme.colors.panel_deep)
    };

    let border_color = if selected {
        color_from_rgba(&theme.colors.border_bright)
    } else if unlocked && can_afford {
        color_from_rgba(&theme.colors.border)
    } else {
        color_from_rgba(&theme.colors.text_dim)
    };
    let border_width = if selected { 2.0 } else { 1.0 };

    draw_rectangle(
        x + 2.0,
        y + 3.0,
        width,
        height,
        color_from_rgba(&theme.colors.shadow),
    );
    draw_rectangle(x, y, width, height, base_color);
    draw_rectangle_lines(x, y, width, height, border_width, border_color);

    if selected {
        draw_rectangle_lines(
            x - 1.0,
            y - 1.0,
            width + 2.0,
            height + 2.0,
            1.0,
            with_alpha(accent, 0.65),
        );
    }

    // The tutorial points at what it is asking for, so the instruction can be
    // followed without being read twice.
    if unlocked && state.tutorial_highlight() == Some(building_type) {
        let glow = 0.4 + macroquad_toolkit::math::pulse01(1.6) * 0.5;
        draw_rectangle_lines(
            x - 3.0,
            y - 3.0,
            width + 6.0,
            height + 6.0,
            2.0,
            with_alpha(color_from_rgba(&theme.colors.success), glow),
        );
    }

    let icon_size = 46.0;
    let icon_x = x + 10.0;
    let icon_y = y + (height - icon_size) * 0.5;
    draw_rectangle(
        icon_x,
        icon_y,
        icon_size,
        icon_size,
        color_from_rgba(&theme.colors.panel_inner),
    );
    draw_rectangle_lines(icon_x, icon_y, icon_size, icon_size, 1.0, border_color);

    if let Some(icon) = textures
        .building_icons
        .by_id
        .get(building_type.id())
        .or_else(|| textures.buildings.by_id.get(building_type.id()))
    {
        let icon_tint = if unlocked && can_afford {
            WHITE
        } else {
            Color::new(0.45, 0.48, 0.5, 1.0)
        };
        draw_texture_ex(
            icon,
            icon_x + 4.0,
            icon_y + 4.0,
            icon_tint,
            DrawTextureParams {
                dest_size: Some(vec2(icon_size - 8.0, icon_size - 8.0)),
                ..Default::default()
            },
        );
    }

    let name_x = icon_x + icon_size + 10.0;
    let name_text = fit_text_to_width(
        building_type.name(data),
        width - 138.0,
        theme.typography.body,
    );
    let name_color = if unlocked && can_afford { text } else { dim };
    draw_ui_text(
        &name_text,
        name_x,
        y + 20.0,
        theme.typography.body,
        name_color,
    );

    let description = fit_text_to_width(
        building_type.description(data),
        width - 92.0,
        theme.typography.small,
    );
    draw_ui_text(&description, name_x, y + 38.0, theme.typography.small, dim);

    let cost_y = y + 60.0;
    let minerals_text = format!("M {}", mineral_cost as i32);
    let energy_text = format!("E {}", energy_cost as i32);
    let mineral_value_color = if state.resources.minerals >= mineral_cost {
        minerals_color
    } else {
        color_from_rgba(&theme.colors.error)
    };
    let energy_value_color = if state.resources.energy >= energy_cost {
        energy_color
    } else {
        color_from_rgba(&theme.colors.error)
    };

    if unlocked {
        draw_ui_text(
            &minerals_text,
            name_x,
            cost_y,
            theme.typography.small,
            mineral_value_color,
        );
        let minerals_width =
            measure_ui_text(&minerals_text, None, theme.typography.small as u16, 1.0).width;
        draw_ui_text(
            &energy_text,
            name_x + minerals_width + 12.0,
            cost_y,
            theme.typography.small,
            energy_value_color,
        );

        let power_text = format!("P {}", format_power_delta(building_type.power_delta(data)));
        let power_width =
            measure_ui_text(&power_text, None, theme.typography.small as u16, 1.0).width;
        draw_ui_text(
            &power_text,
            x + width - power_width - 8.0,
            cost_y,
            theme.typography.small,
            accent,
        );
    } else {
        // The cost is beside the point when the building cannot be placed at
        // all; say why instead. A world that refuses a building is a different
        // problem from one the swarm has not researched yet, and the player can
        // only fix one of them.
        let (label, label_color) = if state.is_building_banned(building_type) {
            (
                "UNAVAILABLE ON THIS WORLD",
                color_from_rgba(&theme.colors.error),
            )
        } else if researched && !state.building_condition_met(building_type) {
            (
                world_condition_label(state, building_type),
                color_from_rgba(&theme.colors.warning),
            )
        } else {
            ("LOCKED", color_from_rgba(&theme.colors.warning))
        };
        draw_ui_text(label, name_x, cost_y, theme.typography.small, label_color);
    }

    if unlocked && interactive && pointer.released_on(rect) {
        state.select_building(building_type);
        if close_on_select {
            state.mobile_panel = MobilePanel::Map;
            state.desktop_panel = crate::state::DesktopPanel::None;
            state.focus_mode = true;
        }
    }

    height
}

fn world_condition_label(state: &PlanetState, building_type: BuildingType) -> &'static str {
    match state.building_planet_condition(building_type) {
        Some("acid") => "NEEDS ACID WORLD",
        Some("cold") => "NEEDS COLD WORLD",
        _ => "REQUIRES A DIFFERENT WORLD",
    }
}
