//! World-space route and processor-node visuals for the flow overlay.

use super::{tile_center, FlowLink, FlowNode};
use crate::data::UiTheme;
use crate::screens::planetary_view::metrics::HudMetrics;
use crate::ui::{color_from_rgba, draw_resource_icon, resource_color, Colors};
use macroquad::prelude::*;
use macroquad_toolkit::colors::with_alpha;
use macroquad_toolkit::ui::draw_ui_text;

pub(super) fn draw_link(link: &FlowLink, metrics: HudMetrics, theme: &UiTheme, time: f32) {
    if link.path.len() < 2 {
        return;
    }
    let resource = resource_color(theme, link.resource);
    let saturation = link.peak_load as f32 / link.capacity.max(1.0);
    let congested = saturation > 1.0;
    let color = if congested {
        color_from_rgba(&theme.colors.warning)
    } else {
        resource
    };
    let pressure = saturation.clamp(0.0, 1.0);
    let route_width = 1.2 + pressure * 1.8;
    let pulse = 0.55 + (time * 4.0).sin().abs() * 0.3;
    for pair in link.path.windows(2) {
        let from = tile_center(pair[0], metrics);
        let to = tile_center(pair[1], metrics);
        draw_line(
            from.x,
            from.y,
            to.x,
            to.y,
            4.0,
            with_alpha(Colors::BACKGROUND, 0.8),
        );
        if congested {
            draw_line(
                from.x,
                from.y,
                to.x,
                to.y,
                5.0,
                with_alpha(color, 0.12 * pulse),
            );
        }
        draw_line(
            from.x,
            from.y,
            to.x,
            to.y,
            route_width,
            with_alpha(color, 0.44 + pressure * 0.38),
        );
    }

    let packet_speed = 1.8 / saturation.max(1.0);
    let travel =
        (time * packet_speed + link.path[0].x as f32 * 0.17 + link.path[0].y as f32 * 0.11)
            .rem_euclid((link.path.len() - 1) as f32);
    let index = travel.floor() as usize;
    let progress = travel.fract();
    let from = tile_center(link.path[index], metrics);
    let to = tile_center(link.path[index + 1], metrics);
    let packet = from.lerp(to, progress);
    draw_circle(packet.x, packet.y, 5.0, with_alpha(Colors::BACKGROUND, 0.9));
    draw_resource_icon(
        link.resource,
        Rect::new(packet.x - 4.0, packet.y - 4.0, 8.0, 8.0),
        resource,
    );
}

pub(super) fn draw_node(node: &FlowNode, starved: bool, metrics: HudMetrics, theme: &UiTheme) {
    let center = tile_center(node.pos, metrics);
    let icon = (metrics.tile_size * 0.29).clamp(8.0, 11.0);
    let item_count = node.inputs.len() + node.outputs.len();
    let width = (20.0 + item_count as f32 * (icon + 3.0)).max(40.0);
    let height = icon + 9.0;
    let x = center.x - width * 0.5;
    let y = center.y - metrics.tile_size * 0.72 - height;
    let border = if node.standby || !node.powered {
        color_from_rgba(&theme.colors.text_dim)
    } else if node.blocked {
        color_from_rgba(&theme.colors.error)
    } else if starved {
        color_from_rgba(&theme.colors.warning)
    } else {
        color_from_rgba(&theme.colors.border_bright)
    };
    draw_rectangle(x, y, width, height, Color::new(0.006, 0.025, 0.035, 0.94));
    draw_rectangle_lines(x, y, width, height, 1.0, with_alpha(border, 0.86));

    let mut cursor = x + 5.0;
    for resource in &node.inputs {
        draw_resource_icon(
            *resource,
            Rect::new(cursor, y + 3.0, icon, icon),
            resource_color(theme, *resource),
        );
        cursor += icon + 3.0;
    }
    draw_ui_text(
        ">",
        cursor,
        y + icon + 2.0,
        8.0,
        color_from_rgba(&theme.colors.text_dim),
    );
    cursor += 8.0;
    for resource in &node.outputs {
        draw_resource_icon(
            *resource,
            Rect::new(cursor, y + 3.0, icon, icon),
            resource_color(theme, *resource),
        );
        cursor += icon + 3.0;
    }

    let bar_y = y + height - 3.0;
    draw_rectangle(
        x + 2.0,
        bar_y,
        width - 4.0,
        1.5,
        with_alpha(Colors::TEXT_DIM, 0.2),
    );
    draw_rectangle(
        x + 2.0,
        bar_y,
        (width - 4.0) * node.readiness,
        1.5,
        if node.blocked {
            color_from_rgba(&theme.colors.error)
        } else if starved {
            color_from_rgba(&theme.colors.warning)
        } else {
            color_from_rgba(&theme.colors.success)
        },
    );
    let output_x = x + width - 3.0;
    let output_h = (height - 4.0) * node.output_pressure;
    draw_rectangle(
        output_x,
        y + 2.0,
        1.5,
        height - 4.0,
        with_alpha(Colors::TEXT_DIM, 0.2),
    );
    draw_rectangle(
        output_x,
        y + height - 2.0 - output_h,
        1.5,
        output_h,
        if node.blocked {
            color_from_rgba(&theme.colors.error)
        } else {
            node.outputs
                .first()
                .map(|resource| resource_color(theme, *resource))
                .unwrap_or(Colors::TEXT_DIM)
        },
    );
    if node.priority {
        draw_circle(
            x + 3.0,
            y + 3.0,
            4.0,
            color_from_rgba(&theme.colors.primary),
        );
        draw_ui_text("P", x + 1.0, y + 6.0, 7.0, Colors::BACKGROUND);
    }
    if node.standby {
        let standby = color_from_rgba(&theme.colors.text_dim);
        draw_line(x + 5.0, y + 4.0, x + 5.0, y + 11.0, 1.5, standby);
        draw_line(x + 8.0, y + 4.0, x + 8.0, y + 11.0, 1.5, standby);
    }
}
