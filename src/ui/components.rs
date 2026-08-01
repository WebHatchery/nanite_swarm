//! Game-specific UI widgets

use super::core::{Colors, Dimensions};
use crate::data::UiTheme;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_ui_text, measure_ui_text};
use macroquad_toolkit::{input::is_hovered, ui::draw_text_centered_in_box};

/// Draw a styled button and return true if clicked
#[allow(dead_code)]
pub fn draw_button(x: f32, y: f32, width: f32, text: &str) -> bool {
    draw_button_sized(x, y, width, Dimensions::BUTTON_HEIGHT, text)
}

/// Draw a styled button with a custom size
pub fn draw_button_sized(x: f32, y: f32, width: f32, height: f32, text: &str) -> bool {
    let hovered = is_hovered(x, y, width, height);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);

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
    macroquad_toolkit::ui::draw_surface(Rect::new(x, y, width, height), &surface);

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

    hovered && is_mouse_button_pressed(MouseButton::Left)
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
    let hovered = is_hovered(rect.x, rect.y, rect.w, rect.h);
    let pressed = hovered && is_mouse_button_down(MouseButton::Left);
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

    hovered && is_mouse_button_pressed(MouseButton::Left)
}

pub fn draw_hud_progress_bar(theme: &UiTheme, rect: Rect, progress: f32, fill_color: Color) {
    let clamped = progress.clamp(0.0, 1.0);
    let background = color_from_rgba(&theme.colors.panel_deep);
    let border = color_from_rgba(&theme.colors.border);
    draw_rectangle(rect.x, rect.y, rect.w, rect.h, background);
    draw_rectangle(rect.x, rect.y, rect.w * clamped, rect.h, fill_color);
    draw_rectangle_lines(rect.x, rect.y, rect.w, rect.h, 1.0, border);
}

fn draw_metric_icon(kind: &str, rect: Rect, color: Color) {
    let cx = rect.x + rect.w * 0.5;
    let cy = rect.y + rect.h * 0.5;
    match kind {
        "energy" => {
            let top = vec2(cx + rect.w * 0.12, rect.y + rect.h * 0.14);
            let mid_left = vec2(cx - rect.w * 0.18, cy + rect.h * 0.02);
            let mid = vec2(cx + rect.w * 0.04, cy + rect.h * 0.02);
            let bottom = vec2(cx - rect.w * 0.1, rect.y + rect.h * 0.86);
            let mid_right = vec2(cx + rect.w * 0.24, cy - rect.h * 0.1);
            draw_triangle(top, mid_left, mid, color);
            draw_triangle(mid, mid_right, bottom, color);
        }
        "minerals" => {
            draw_poly(cx, cy - 3.0, 6, rect.w * 0.24, 0.0, color);
            draw_poly_lines(cx, cy - 3.0, 6, rect.w * 0.28, 0.0, 1.2, WHITE);
            draw_line(cx, rect.y + 8.0, cx, rect.y + rect.h - 8.0, 1.0, WHITE);
        }
        "data" => {
            let node = rect.w * 0.14;
            draw_rectangle(cx - node * 2.2, cy - node * 1.9, node, node, color);
            draw_rectangle(cx + node * 1.2, cy - node * 1.9, node, node, color);
            draw_rectangle(cx - node * 0.5, cy + node * 0.9, node, node, color);
            draw_line(cx - node * 1.2, cy - node * 1.4, cx, cy + node, 1.2, color);
            draw_line(cx + node * 1.7, cy - node * 1.4, cx, cy + node, 1.2, color);
        }
        "alloy" => {
            // A poured ingot: trapezoid over a shadow line.
            let w = rect.w * 0.30;
            let h = rect.h * 0.16;
            draw_triangle(
                vec2(cx - w, cy + h),
                vec2(cx + w, cy + h),
                vec2(cx + w * 0.66, cy - h),
                color,
            );
            draw_triangle(
                vec2(cx - w, cy + h),
                vec2(cx + w * 0.66, cy - h),
                vec2(cx - w * 0.66, cy - h),
                color,
            );
            draw_line(cx - w, cy + h + 2.0, cx + w, cy + h + 2.0, 1.2, WHITE);
        }
        "biomass" => {
            draw_ellipse(
                cx - 5.0,
                cy + 1.0,
                rect.w * 0.13,
                rect.h * 0.26,
                25.0,
                color,
            );
            draw_ellipse(
                cx + 5.0,
                cy + 1.0,
                rect.w * 0.13,
                rect.h * 0.26,
                -25.0,
                color,
            );
            draw_line(cx, cy + 9.0, cx, cy - 10.0, 1.2, color);
        }
        _ => {
            draw_text_centered_in_box(kind, rect.x, rect.y, rect.w, rect.h, 17.0, color);
        }
    }
}

pub fn draw_metric_card(
    theme: &UiTheme,
    rect: Rect,
    icon_kind: &str,
    label: &str,
    value: &str,
    rate: &str,
    capacity: Option<&str>,
    accent: Color,
) {
    draw_hud_panel(theme, rect, None);

    let compact = rect.w < 112.0;
    let icon_size = if rect.w < 130.0 { 28.0 } else { 34.0 };
    let icon_box = Rect::new(rect.x + 10.0, rect.y + 13.0, icon_size, icon_size);
    let icon_fill = color_from_rgba(&theme.colors.panel_inner);
    let text = color_from_rgba(&theme.colors.text);
    let dim = color_from_rgba(&theme.colors.text_dim);

    if !compact {
        draw_rectangle(icon_box.x, icon_box.y, icon_box.w, icon_box.h, icon_fill);
        draw_rectangle_lines(icon_box.x, icon_box.y, icon_box.w, icon_box.h, 1.0, accent);
        draw_metric_icon(icon_kind, icon_box, accent);
    } else {
        draw_metric_icon(
            icon_kind,
            Rect::new(rect.x + rect.w - 24.0, rect.y + 9.0, 16.0, 16.0),
            accent,
        );
    }

    let text_x = if compact {
        rect.x + 10.0
    } else {
        icon_box.x + icon_box.w + 12.0
    };
    let value_size = if compact {
        16.0
    } else if rect.w < 130.0 {
        18.0
    } else {
        theme.typography.value
    };
    let small_size = if compact { 9.0 } else { theme.typography.small };
    draw_ui_text(label, text_x, rect.y + 19.0, small_size, text);
    let value_y = if rect.h < 65.0 {
        rect.y + 39.0
    } else {
        rect.y + 42.0
    };
    let footer_y = rect.y + rect.h - 7.0;
    draw_ui_text(value, text_x, value_y, value_size, text);
    draw_ui_text(rate, text_x, footer_y, small_size, accent);
    if let Some(capacity_text) = capacity.filter(|_| rect.w >= 118.0) {
        let cap_width = measure_ui_text(capacity_text, None, small_size as u16, 1.0).width;
        draw_ui_text(
            capacity_text,
            rect.x + rect.w - cap_width - 10.0,
            footer_y,
            small_size,
            dim,
        );
    }
}

pub fn draw_status_row(
    theme: &UiTheme,
    x: f32,
    y: f32,
    width: f32,
    label: &str,
    value: &str,
    value_color: Color,
) {
    let label_color = color_from_rgba(&theme.colors.text_dim);
    draw_ui_text(label, x, y, theme.typography.body, label_color);
    let value_width = measure_ui_text(value, None, theme.typography.body as u16, 1.0).width;
    let value_x = (x + width - value_width).max(x + width * 0.48);
    draw_ui_text(value, value_x, y, theme.typography.body, value_color);
}

/// Draw resource display
#[allow(dead_code)]
pub fn draw_resource(x: f32, y: f32, label: &str, value: f32, color: Color) {
    draw_ui_text(
        &format!("{}: {:.0}", label, value),
        x,
        y,
        Dimensions::FONT_SIZE_NORMAL,
        color,
    );
}
