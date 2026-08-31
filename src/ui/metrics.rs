//! Resource icons and metric widgets shared by world-facing screens.

use super::components::{color_from_rgba, draw_hud_panel};
use super::core::Dimensions;
use crate::data::UiTheme;
use crate::engine::ResourceType;
use macroquad::prelude::*;
use macroquad_toolkit::ui::{draw_text_centered_in_box, draw_ui_text, measure_ui_text};

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
            draw_line(
                cx,
                rect.y + rect.h * 0.24,
                cx,
                rect.y + rect.h * 0.76,
                1.0,
                WHITE,
            );
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
        "components" => {
            // Interlocked precision rings: readable independently of colour.
            let radius = rect.w * 0.18;
            let spread = rect.w * 0.14;
            draw_poly(cx - spread, cy, 6, radius, 0.0, color);
            draw_poly_lines(cx - spread, cy, 6, radius, 0.0, 1.2, WHITE);
            draw_poly(cx + spread, cy, 6, radius, 30.0, color);
            draw_poly_lines(cx + spread, cy, 6, radius, 30.0, 1.2, WHITE);
            let cutout = Color::new(0.006, 0.025, 0.035, 1.0);
            draw_circle(cx - spread, cy, radius * 0.42, cutout);
            draw_circle(cx + spread, cy, radius * 0.42, cutout);
        }
        "biomass" => {
            draw_ellipse(
                cx - rect.w * 0.14,
                cy + 1.0,
                rect.w * 0.13,
                rect.h * 0.26,
                25.0,
                color,
            );
            draw_ellipse(
                cx + rect.w * 0.14,
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

/// Draw the same shape language used by the resource ribbon at any scale.
/// Freight packets use this so their material remains readable without colour.
pub fn draw_resource_icon(resource: ResourceType, rect: Rect, color: Color) {
    draw_metric_icon(resource.id(), rect, color);
}

/// Keep world-space freight and HUD cards on the same data-driven palette.
pub fn resource_color(theme: &UiTheme, resource: ResourceType) -> Color {
    color_from_rgba(match resource {
        ResourceType::Energy => &theme.colors.energy,
        ResourceType::Minerals => &theme.colors.minerals,
        ResourceType::Data => &theme.colors.data,
        ResourceType::Biomass => &theme.colors.biomass,
        ResourceType::Alloy => &theme.colors.alloy,
        ResourceType::Components => &theme.colors.components,
    })
}

#[allow(clippy::too_many_arguments)]
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
