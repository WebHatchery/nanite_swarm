//! Core, drill, and conduit tile art.

use super::super::super::utils::*;
use super::super::save_building_with_icon;
use image::{Rgba, RgbaImage};

pub(super) fn generate() {
    save_building_with_icon("building_core_stage_1a", create_core_stage_1a());
    save_building_with_icon("building_core_stage_1b", create_core_stage_1b());
    save_building_with_icon("building_core_stage_1c", create_core_stage_1c());
    save_building_with_icon("building_core_stage_2a", create_core_stage_2a());
    save_building_with_icon("building_core_stage_2b", create_core_stage_2b());
    save_building_with_icon("building_core_stage_3a", create_core_stage_3a());
    save_building_with_icon("building_core_stage_3b", create_core_stage_3b());
    save_building_with_icon("building_core_stage_4a", create_core_stage_4a());
    save_building_with_icon("building_core_stage_4b", create_core_stage_4b());
    save_building_with_icon("building_drill", create_drill());
    save_building_with_icon(
        "building_conduit_straight_h",
        create_conduit_variant(ConduitVariant::StraightH),
    );
    save_building_with_icon(
        "building_conduit_straight_v",
        create_conduit_variant(ConduitVariant::StraightV),
    );
    save_building_with_icon(
        "building_conduit_corner_ne",
        create_conduit_variant(ConduitVariant::CornerNE),
    );
    save_building_with_icon(
        "building_conduit_corner_nw",
        create_conduit_variant(ConduitVariant::CornerNW),
    );
    save_building_with_icon(
        "building_conduit_corner_se",
        create_conduit_variant(ConduitVariant::CornerSE),
    );
    save_building_with_icon(
        "building_conduit_corner_sw",
        create_conduit_variant(ConduitVariant::CornerSW),
    );
    save_building_with_icon(
        "building_conduit_tee_n",
        create_conduit_variant(ConduitVariant::TeeN),
    );
    save_building_with_icon(
        "building_conduit_tee_e",
        create_conduit_variant(ConduitVariant::TeeE),
    );
    save_building_with_icon(
        "building_conduit_tee_s",
        create_conduit_variant(ConduitVariant::TeeS),
    );
    save_building_with_icon(
        "building_conduit_tee_w",
        create_conduit_variant(ConduitVariant::TeeW),
    );
    save_building_with_icon(
        "building_conduit_cross",
        create_conduit_variant(ConduitVariant::Cross),
    );
}

fn create_core_stage_1a() -> RgbaImage {
    let mut img = create_tile_base(Rgba([40, 45, 55, 255]));
    add_noise(&mut img, 6, 1111);
    draw_circle(&mut img, 16, 16, 12, Rgba([20, 120, 160, 255]));
    draw_circle(&mut img, 16, 16, 9, Rgba([15, 70, 95, 255]));
    draw_circle(&mut img, 16, 16, 5, Rgba([120, 230, 255, 255]));
    draw_circle(&mut img, 16, 16, 2, Rgba([255, 255, 255, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_core_stage_1b() -> RgbaImage {
    let mut img = create_core_stage_1a();
    draw_rect(&mut img, 6, 14, 4, 4, Rgba([80, 100, 120, 255]));
    draw_rect(&mut img, 22, 14, 4, 4, Rgba([80, 100, 120, 255]));
    draw_rect(&mut img, 14, 6, 4, 4, Rgba([80, 100, 120, 255]));
    draw_rect(&mut img, 14, 22, 4, 4, Rgba([80, 100, 120, 255]));
    img
}

fn create_core_stage_1c() -> RgbaImage {
    let mut img = create_core_stage_1b();
    draw_rect(&mut img, 10, 24, 4, 3, Rgba([30, 180, 210, 255]));
    draw_rect(&mut img, 18, 24, 4, 3, Rgba([30, 180, 210, 255]));
    img
}

fn create_core_stage_2a() -> RgbaImage {
    let mut img = create_core_stage_1c();
    draw_rect(&mut img, 4, 4, 4, 24, Rgba([120, 120, 130, 255]));
    draw_rect(&mut img, 24, 4, 4, 24, Rgba([120, 120, 130, 255]));
    img
}

fn create_core_stage_2b() -> RgbaImage {
    let mut img = create_core_stage_2a();
    draw_rect(&mut img, 8, 2, 4, 8, Rgba([140, 150, 160, 255]));
    draw_rect(&mut img, 20, 2, 4, 10, Rgba([140, 150, 160, 255]));
    draw_rect(&mut img, 12, 20, 8, 6, Rgba([90, 110, 130, 255]));
    img
}

fn create_core_stage_3a() -> RgbaImage {
    let mut img = create_core_stage_2b();
    draw_circle(&mut img, 16, 16, 14, Rgba([40, 180, 205, 255]));
    draw_circle(&mut img, 16, 16, 10, Rgba([20, 55, 80, 255]));
    for (x, y) in [(3, 3), (25, 3), (3, 25), (25, 25)] {
        draw_rect(&mut img, x, y, 4, 4, Rgba([195, 160, 60, 255]));
    }
    img
}

fn create_core_stage_3b() -> RgbaImage {
    let mut img = create_core_stage_3a();
    draw_rect(&mut img, 13, 1, 6, 8, Rgba([220, 230, 235, 255]));
    draw_rect(&mut img, 13, 23, 6, 8, Rgba([220, 230, 235, 255]));
    draw_circle(&mut img, 16, 16, 4, Rgba([240, 250, 255, 255]));
    img
}

fn create_core_stage_4a() -> RgbaImage {
    let mut img = create_core_stage_3b();
    draw_circle(&mut img, 16, 16, 15, Rgba([125, 75, 210, 255]));
    draw_circle(&mut img, 16, 16, 12, Rgba([25, 35, 70, 255]));
    for i in 0..8 {
        let x = 5 + (i * 7) % 22;
        let y = 4 + (i * 11) % 22;
        draw_rect(&mut img, x, y, 2, 2, Rgba([100, 220, 255, 255]));
    }
    img
}

fn create_core_stage_4b() -> RgbaImage {
    let mut img = create_core_stage_4a();
    draw_circle(&mut img, 16, 16, 6, Rgba([225, 250, 255, 255]));
    draw_circle(&mut img, 16, 16, 2, Rgba([255, 255, 255, 255]));
    img
}

fn create_drill() -> RgbaImage {
    let mut img = create_tile_base(Rgba([55, 55, 60, 255]));
    add_noise(&mut img, 8, 1212);
    draw_rect(&mut img, 6, 20, 20, 8, Rgba([80, 80, 90, 255]));
    draw_rect(&mut img, 13, 8, 6, 14, Rgba([110, 110, 120, 255]));
    draw_circle(&mut img, 16, 6, 5, Rgba([180, 160, 60, 255]));
    draw_circle(&mut img, 16, 6, 2, Rgba([230, 210, 90, 255]));
    for i in 0..4 {
        let x = 7 + i * 5;
        draw_rect(&mut img, x, 22, 2, 4, Rgba([180, 140, 30, 255]));
    }
    add_edge_darkening(&mut img, 2, 8);
    img
}

#[derive(Clone, Copy)]
enum ConduitVariant {
    StraightH,
    StraightV,
    CornerNE,
    CornerNW,
    CornerSE,
    CornerSW,
    TeeN,
    TeeE,
    TeeS,
    TeeW,
    Cross,
}

fn create_conduit_variant(variant: ConduitVariant) -> RgbaImage {
    let mut img = create_tile_base(Rgba([35, 40, 45, 255]));
    add_noise(&mut img, 5, 1313);

    let center_u = (16u32, 16u32);
    let center_i = (16i32, 16i32);
    let thickness_u: u32 = 4;

    let mut arms = Vec::new();
    match variant {
        ConduitVariant::StraightH => arms.extend(["W", "E"]),
        ConduitVariant::StraightV => arms.extend(["N", "S"]),
        ConduitVariant::CornerNE => arms.extend(["N", "E"]),
        ConduitVariant::CornerNW => arms.extend(["N", "W"]),
        ConduitVariant::CornerSE => arms.extend(["S", "E"]),
        ConduitVariant::CornerSW => arms.extend(["S", "W"]),
        ConduitVariant::TeeN => arms.extend(["N", "E", "W"]),
        ConduitVariant::TeeE => arms.extend(["E", "N", "S"]),
        ConduitVariant::TeeS => arms.extend(["S", "E", "W"]),
        ConduitVariant::TeeW => arms.extend(["W", "N", "S"]),
        ConduitVariant::Cross => arms.extend(["N", "E", "S", "W"]),
    }

    for arm in arms {
        match arm {
            "N" => {
                draw_rect(
                    &mut img,
                    center_u.0 - thickness_u / 2,
                    2,
                    thickness_u,
                    center_u.1 - 2,
                    Rgba([20, 120, 160, 255]),
                );
                draw_rect(
                    &mut img,
                    center_u.0 - thickness_u / 2,
                    3,
                    thickness_u,
                    center_u.1 - 4,
                    Rgba([60, 200, 230, 255]),
                );
                draw_circle(&mut img, center_i.0, 4, 2, Rgba([150, 230, 240, 255]));
            }
            "S" => {
                draw_rect(
                    &mut img,
                    center_u.0 - thickness_u / 2,
                    center_u.1,
                    thickness_u,
                    TILE_HEIGHT - center_u.1 - 2,
                    Rgba([20, 120, 160, 255]),
                );
                draw_rect(
                    &mut img,
                    center_u.0 - thickness_u / 2,
                    center_u.1 + 1,
                    thickness_u,
                    TILE_HEIGHT - center_u.1 - 4,
                    Rgba([60, 200, 230, 255]),
                );
                draw_circle(
                    &mut img,
                    center_i.0,
                    (TILE_HEIGHT - 4) as i32,
                    2,
                    Rgba([150, 230, 240, 255]),
                );
            }
            "W" => {
                draw_rect(
                    &mut img,
                    2,
                    center_u.1 - thickness_u / 2,
                    center_u.0 - 2,
                    thickness_u,
                    Rgba([20, 120, 160, 255]),
                );
                draw_rect(
                    &mut img,
                    3,
                    center_u.1 - thickness_u / 2,
                    center_u.0 - 4,
                    thickness_u,
                    Rgba([60, 200, 230, 255]),
                );
                draw_circle(&mut img, 4, center_i.1, 2, Rgba([150, 230, 240, 255]));
            }
            "E" => {
                draw_rect(
                    &mut img,
                    center_u.0,
                    center_u.1 - thickness_u / 2,
                    TILE_WIDTH - center_u.0 - 2,
                    thickness_u,
                    Rgba([20, 120, 160, 255]),
                );
                draw_rect(
                    &mut img,
                    center_u.0 + 1,
                    center_u.1 - thickness_u / 2,
                    TILE_WIDTH - center_u.0 - 4,
                    thickness_u,
                    Rgba([60, 200, 230, 255]),
                );
                draw_circle(
                    &mut img,
                    (TILE_WIDTH - 4) as i32,
                    center_i.1,
                    2,
                    Rgba([150, 230, 240, 255]),
                );
            }
            _ => {}
        }
    }

    draw_circle(
        &mut img,
        center_i.0,
        center_i.1,
        3,
        Rgba([120, 210, 230, 255]),
    );
    draw_circle(
        &mut img,
        center_i.0,
        center_i.1,
        1,
        Rgba([220, 255, 255, 255]),
    );

    add_edge_darkening(&mut img, 2, 6);
    img
}
