//! Game-specific UI widgets

use super::core::{Colors, Dimensions};
use crate::data::UiTheme;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{
    draw_text_centered_in_box, draw_ui_text, note_neighbour, touch_area, Pointer,
};

/// Draw a styled button and return true if clicked
#[allow(dead_code)]
pub fn draw_button(x: f32, y: f32, width: f32, text: &str) -> bool {
    draw_button_sized(x, y, width, Dimensions::BUTTON_HEIGHT, text)
}

/// Draw a styled button with a custom size
pub fn draw_button_sized(x: f32, y: f32, width: f32, height: f32, text: &str) -> bool {
    let rect = Rect::new(x, y, width, height);
    note_neighbour(rect);
    let hit_rect = touch_area(rect);
    let pointer = Pointer::read(|position| position);
    let hovered = pointer.hovering_over(hit_rect);
    let pressed = pointer.pressing(hit_rect);

    let base_color = if pressed {
        Colors::PRIMARY_SOFT
    } else if hovered {
        Colors::PRIMARY
    } else {
        Colors::SURFACE_DARK
    };
    let text_color = if hovered {
        Colors::BACKGROUND
    } else {
        Colors::TEXT
    };

    let surface = macroquad_toolkit::ui::SurfaceStyle::new(base_color)
        .with_shadow(vec2(2.0, 3.0), Color::new(0.0, 0.0, 0.0, 0.35))
        .with_border(2.0, Colors::PANEL_BORDER)
        .with_top_highlight(3.0, Color::new(1.0, 1.0, 1.0, 0.08));
    macroquad_toolkit::ui::draw_surface(rect, &surface);

    let font_size = if height >= 38.0 {
        Dimensions::FONT_SIZE_NORMAL
    } else {
        Dimensions::FONT_SIZE_SMALL
    };
    draw_text_centered_in_box(
        text,
        x + 8.0,
        y,
        width - 16.0,
        height,
        font_size,
        text_color,
    );

    pointer.released_on(hit_rect)
}

/// Draw a panel background
pub fn draw_panel(x: f32, y: f32, width: f32, height: f32) {
    let surface = macroquad_toolkit::ui::SurfaceStyle::new(Colors::SURFACE)
        .with_shadow(vec2(3.0, 4.0), Color::new(0.0, 0.0, 0.0, 0.32))
        .with_header(6.0, Colors::SURFACE_DARK)
        .with_top_highlight(2.0, Color::new(1.0, 1.0, 1.0, 0.06))
        .with_border(1.0, Colors::PANEL_BORDER);
    macroquad_toolkit::ui::draw_surface(Rect::new(x, y, width, height), &surface);
}

pub fn color_from_rgba(rgba: &[f32; 4]) -> Color {
    Color::new(rgba[0], rgba[1], rgba[2], rgba[3])
}

pub fn draw_hud_panel(theme: &UiTheme, rect: Rect, title: Option<&str>) {
    let border = color_from_rgba(&theme.colors.border);
    let bright = color_from_rgba(&theme.colors.border_bright);
    let panel = color_from_rgba(&theme.colors.panel);
    let shadow = color_from_rgba(&theme.colors.shadow);
    let title_color = color_from_rgba(&theme.colors.primary);

    draw_rectangle(rect.x + 3.0, rect.y + 4.0, rect.w, rect.h, shadow);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, panel);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, border);

    let corner = 14.0;
    draw_line(
        rect.x,
        rect.y + corner,
        rect.x + corner,
        rect.y,
        1.5,
        bright,
    );
    draw_line(
        rect.x + rect.w - corner,
        rect.y,
        rect.x + rect.w,
        rect.y + corner,
        1.5,
        bright,
    );
    draw_line(
        rect.x,
        rect.y + rect.h - corner,
        rect.x + corner,
        rect.y + rect.h,
        1.5,
        bright,
    );
    draw_line(
        rect.x + rect.w - corner,
        rect.y + rect.h,
        rect.x + rect.w,
        rect.y + rect.h - corner,
        1.5,
        bright,
    );

    if let Some(text) = title {
        draw_ui_text(
            text,
            rect.x + theme.layout.panel_padding,
            rect.y + 24.0,
            theme.typography.section,
            title_color,
        );
        draw_line(
            rect.x + theme.layout.panel_padding,
            rect.y + 32.0,
            rect.x + rect.w - theme.layout.panel_padding,
            rect.y + 32.0,
            1.0,
            border,
        );
    }
}

pub fn draw_hud_button(theme: &UiTheme, rect: Rect, label: &str) -> bool {
    note_neighbour(rect);
    let hit_rect = touch_area(rect);
    let pointer = Pointer::read(|position| position);
    let hovered = pointer.hovering_over(hit_rect);
    let pressed = pointer.pressing(hit_rect);
    let mut fill = color_from_rgba(&theme.colors.panel_deep);
    if hovered {
        fill = color_from_rgba(&theme.colors.panel_inner);
    }
    if pressed {
        fill = color_from_rgba(&theme.colors.primary_soft);
    }

    let border = if hovered {
        color_from_rgba(&theme.colors.border_bright)
    } else {
        color_from_rgba(&theme.colors.border)
    };
    let text_color = color_from_rgba(&theme.colors.text);

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, border);
    draw_text_centered_in_box(
        label,
        rect.x + 4.0,
        rect.y,
        rect.w - 8.0,
        rect.h,
        theme.typography.body,
        text_color,
    );

    pointer.released_on(hit_rect)
}

pub fn draw_hud_progress_bar(theme: &UiTheme, rect: Rect, progress: f32, fill_color: Color) {
    let clamped = progress.clamp(0.0, 1.0);
    let background = color_from_rgba(&theme.colors.panel_deep);
    let border = color_from_rgba(&theme.colors.border);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, background);
    draw_rectangle(rect.x, rect.y, rect.w * clamped, rect.h, fill_color);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, border);
}
