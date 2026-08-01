//! Left sidebar: the build palette list and quick actions

use crate::assets::GameTextures;
use crate::data::{self, UiTheme};
use crate::engine::BuildingType;
use crate::state::PlanetState;
use crate::ui::{color_from_rgba, draw_hud_button, draw_hud_panel};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};

use super::super::format::{fit_text_to_width, format_power_delta};
use super::super::metrics::HudMetrics;
use super::PanelColors;

#[allow(clippy::too_many_arguments)]
fn draw_build_row(
    state: &mut PlanetState,
    theme: &UiTheme,
    textures: &GameTextures,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    building_type: BuildingType,
) -> f32 {
    let (mouse_x, mouse_y) = mouse_position();
    let hovered = mouse_x >= x && mouse_x <= x + width && mouse_y >= y && mouse_y <= y + height;
    let selected = state.selected_building == Some(building_type);
    let (mineral_cost, energy_cost) = building_type.cost();
    let can_afford = state.resources.can_afford(mineral_cost, energy_cost);
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
    let name_text = fit_text_to_width(building_type.name(), width - 138.0, theme.typography.body);
    let name_color = if unlocked && can_afford { text } else { dim };
    draw_ui_text(
        &name_text,
        name_x,
        y + 20.0,
        theme.typography.body,
        name_color,
    );

    if let Some(hotkey) = building_type.hotkey() {
        let hotkey_text = format!("[{}]", hotkey);
        let hotkey_width =
            measure_ui_text(&hotkey_text, None, theme.typography.small as u16, 1.0).width;
        draw_ui_text(
            &hotkey_text,
            x + width - hotkey_width - 10.0,
            y + 20.0,
            theme.typography.small,
            accent,
        );
    }

    let description = fit_text_to_width(
        building_type.description(),
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

        let power_text = format!("P {}", format_power_delta(building_type.power_delta()));
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
        } else {
            ("LOCKED", color_from_rgba(&theme.colors.warning))
        };
        draw_ui_text(label, name_x, cost_y, theme.typography.small, label_color);
    }

    if unlocked && hovered && is_mouse_button_pressed(MouseButton::Left) {
        state.select_building(building_type);
    }

    height
}

pub(super) fn draw(
    state: &mut PlanetState,
    theme: &UiTheme,
    textures: &GameTextures,
    metrics: HudMetrics,
    colors: &PanelColors,
    screen_h: f32,
) {
    let dim = colors.dim;

    let sidebar_x = 10.0;
    let sidebar_y = metrics.top_bar_height + metrics.panel_gap;
    let sidebar_w = metrics.left_panel_width - metrics.panel_gap * 1.5;
    let sidebar_h = screen_h - sidebar_y - metrics.bottom_bar_height - metrics.panel_gap;
    draw_hud_panel(
        theme,
        Rect::new(sidebar_x, sidebar_y, sidebar_w, sidebar_h),
        Some("BUILD PALETTE"),
    );
    draw_ui_text(
        "Drag or click to build",
        sidebar_x + metrics.panel_padding,
        sidebar_y + 50.0,
        theme.typography.small,
        dim,
    );

    let mut building_defs: Vec<_> = data::game_data()
        .buildings
        .iter()
        .filter(|def| def.show_in_build_menu)
        .collect();
    building_defs.sort_by_key(|def| def.build_menu_order);

    let list_top = sidebar_y + 62.0;
    let quick_actions_h = 64.0;
    let list_bottom = sidebar_y + sidebar_h - quick_actions_h - metrics.panel_gap;
    let list_height = (list_bottom - list_top).max(0.0);
    let content_x = sidebar_x + metrics.panel_padding;
    let content_w = sidebar_w - metrics.panel_padding * 2.0;
    let row_gap = 8.0;
    let card_w = content_w;

    let mut visible_buildings = Vec::new();
    for def in building_defs {
        let Some(building) = BuildingType::from_id(&def.id) else {
            continue;
        };
        // Banned buildings stay on the list once researched, greyed out: the
        // constraint is the puzzle, so hiding it hides the puzzle.
        if !state.is_building_researched(building) {
            continue;
        }
        visible_buildings.push(building);
    }

    let total_rows = visible_buildings.len();
    let total_height = if total_rows == 0 {
        0.0
    } else {
        total_rows as f32 * metrics.build_row_height
            + (total_rows.saturating_sub(1)) as f32 * row_gap
    };
    let list_view = Rect::new(content_x, list_top, content_w, list_height);
    state.build_palette_scroll.update(list_view, total_height);
    let scroll_offset = state.build_palette_scroll.offset();

    let start_y = list_top - scroll_offset;
    for (index, building) in visible_buildings.into_iter().enumerate() {
        let card_y = start_y + index as f32 * (metrics.build_row_height + row_gap);
        if card_y + metrics.build_row_height < list_top || card_y > list_bottom {
            continue;
        }
        draw_build_row(
            state,
            theme,
            textures,
            content_x,
            card_y,
            card_w,
            metrics.build_row_height,
            building,
        );
    }

    state
        .build_palette_scroll
        .draw_scrollbar(list_view, total_height);

    let quick_actions_y = list_bottom + 12.0;
    if draw_hud_button(
        theme,
        Rect::new(
            sidebar_x + metrics.panel_padding,
            quick_actions_y,
            sidebar_w - metrics.panel_padding * 2.0,
            30.0,
        ),
        if state.demolish_mode {
            "DEMOLISH MODE  [ON]"
        } else {
            "DEMOLISH MODE"
        },
    ) {
        state.toggle_demolish_mode();
    }
    draw_ui_text(
        "[H] Harvest terrain  [F] Forest filter",
        sidebar_x + metrics.panel_padding,
        quick_actions_y + 48.0,
        theme.typography.small,
        dim,
    );
}
