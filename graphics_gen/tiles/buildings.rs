//! Procedural building tile art.

use super::super::utils::*;
use super::save_building_with_icon;
use image::{Rgba, RgbaImage};

pub fn generate() {
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
    save_building_with_icon("building_bridge", create_bridge());
    save_building_with_icon("building_power_node", create_power_node());
    save_building_with_icon("building_wind_turbine", create_wind_turbine());
    save_building_with_icon("building_geothermal_tap", create_geothermal_tap());
    save_building_with_icon("building_fusion_reactor", create_fusion_reactor());
    save_building_with_icon("building_server_bank", create_server_bank());
    save_building_with_icon("building_sweeper", create_sweeper());
    save_building_with_icon("building_storage", create_storage());
    save_building_with_icon("building_biomass_harvester", create_biomass_harvester());
    save_building_with_icon("building_heater_node", create_heater_node());
    save_building_with_icon("building_shield_generator", create_shield_generator());
    save_building_with_icon("building_smelter", create_smelter());
    save_building_with_icon("building_assembler", create_assembler());
    save_building_with_icon("building_mass_driver", create_mass_driver());
    save_building_with_icon("building_landing_pad", create_landing_pad());
}

fn create_core_stage_1a() -> RgbaImage {
    let mut img = create_tile_base(Rgba([40, 45, 55, 255]));
    add_noise(&mut img, 6, 1111);
    // Outer ring
    draw_circle(&mut img, 16, 16, 12, Rgba([20, 120, 160, 255]));
    draw_circle(&mut img, 16, 16, 9, Rgba([15, 70, 95, 255]));
    // Inner core
    draw_circle(&mut img, 16, 16, 5, Rgba([120, 230, 255, 255]));
    draw_circle(&mut img, 16, 16, 2, Rgba([255, 255, 255, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_core_stage_1b() -> RgbaImage {
    let mut img = create_core_stage_1a();
    // Add arms
    draw_rect(&mut img, 6, 14, 4, 4, Rgba([80, 100, 120, 255]));
    draw_rect(&mut img, 22, 14, 4, 4, Rgba([80, 100, 120, 255]));
    draw_rect(&mut img, 14, 6, 4, 4, Rgba([80, 100, 120, 255]));
    draw_rect(&mut img, 14, 22, 4, 4, Rgba([80, 100, 120, 255]));
    img
}

fn create_core_stage_1c() -> RgbaImage {
    let mut img = create_core_stage_1b();
    // Drone bays
    draw_rect(&mut img, 10, 24, 4, 3, Rgba([30, 180, 210, 255]));
    draw_rect(&mut img, 18, 24, 4, 3, Rgba([30, 180, 210, 255]));
    img
}

fn create_core_stage_2a() -> RgbaImage {
    let mut img = create_core_stage_1c();
    // Scaffold frame
    draw_rect(&mut img, 4, 4, 4, 24, Rgba([120, 120, 130, 255]));
    draw_rect(&mut img, 24, 4, 4, 24, Rgba([120, 120, 130, 255]));
    img
}

fn create_core_stage_2b() -> RgbaImage {
    let mut img = create_core_stage_2a();
    // Factory towers
    draw_rect(&mut img, 8, 2, 4, 8, Rgba([140, 150, 160, 255]));
    draw_rect(&mut img, 20, 2, 4, 10, Rgba([140, 150, 160, 255]));
    draw_rect(&mut img, 12, 20, 8, 6, Rgba([90, 110, 130, 255]));
    img
}

fn create_core_stage_3a() -> RgbaImage {
    let mut img = create_core_stage_2b();
    // A wider command ring and four orbital pylons.
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
    // Base platform
    draw_rect(&mut img, 6, 20, 20, 8, Rgba([80, 80, 90, 255]));
    // Drill column
    draw_rect(&mut img, 13, 8, 6, 14, Rgba([110, 110, 120, 255]));
    // Drill head
    draw_circle(&mut img, 16, 6, 5, Rgba([180, 160, 60, 255]));
    draw_circle(&mut img, 16, 6, 2, Rgba([230, 210, 90, 255]));
    // Caution stripes
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

    // Central node
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

fn create_bridge() -> RgbaImage {
    let mut img = create_tile_base(Rgba([40, 42, 48, 255]));
    add_noise(&mut img, 6, 1414);
    // Cross brace
    draw_line(&mut img, 6, 6, 26, 26, Rgba([170, 170, 180, 255]));
    draw_line(&mut img, 26, 6, 6, 26, Rgba([170, 170, 180, 255]));
    // Frame
    draw_rect(&mut img, 8, 8, 16, 16, Rgba([80, 85, 95, 200]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_power_node() -> RgbaImage {
    let mut img = create_tile_base(Rgba([38, 42, 50, 255]));
    add_noise(&mut img, 6, 1515);
    draw_circle(&mut img, 16, 16, 7, Rgba([30, 120, 180, 255]));
    draw_circle(&mut img, 16, 16, 4, Rgba([120, 220, 255, 255]));
    // Small antenna
    draw_rect(&mut img, 15, 5, 2, 6, Rgba([170, 170, 190, 255]));
    draw_circle(&mut img, 16, 4, 2, Rgba([120, 220, 255, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_wind_turbine() -> RgbaImage {
    let mut img = create_tile_base(Rgba([40, 45, 55, 255]));
    add_noise(&mut img, 5, 1616);
    // Hub
    draw_circle(&mut img, 16, 16, 3, Rgba([200, 200, 210, 255]));
    // Blades
    draw_line(&mut img, 16, 16, 26, 10, Rgba([210, 210, 220, 255]));
    draw_line(&mut img, 16, 16, 10, 26, Rgba([210, 210, 220, 255]));
    draw_line(&mut img, 16, 16, 6, 12, Rgba([210, 210, 220, 255]));
    // Base
    draw_rect(&mut img, 14, 22, 4, 6, Rgba([90, 95, 105, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_geothermal_tap() -> RgbaImage {
    let mut img = create_tile_base(Rgba([52, 35, 28, 255]));
    add_noise(&mut img, 6, 2727);
    // The orange wellhead is sunk into a cracked volcanic plate.
    draw_line(&mut img, 4, 25, 12, 18, Rgba([150, 70, 38, 255]));
    draw_line(&mut img, 28, 25, 20, 18, Rgba([150, 70, 38, 255]));
    draw_line(&mut img, 7, 28, 14, 22, Rgba([205, 92, 36, 220]));
    draw_line(&mut img, 25, 28, 18, 22, Rgba([205, 92, 36, 220]));
    draw_rect(&mut img, 9, 18, 14, 8, Rgba([92, 62, 54, 255]));
    draw_rect(&mut img, 12, 11, 8, 10, Rgba([170, 78, 38, 255]));
    draw_rect(&mut img, 14, 7, 4, 6, Rgba([225, 125, 55, 255]));
    draw_circle(&mut img, 16, 12, 3, Rgba([255, 196, 105, 255]));
    draw_circle(&mut img, 16, 12, 1, Rgba([255, 245, 190, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_fusion_reactor() -> RgbaImage {
    let mut img = create_tile_base(Rgba([30, 35, 58, 255]));
    add_noise(&mut img, 5, 2828);
    // A cold-world reactor: armored ring, control pylons, and a blue-white core.
    draw_rect(&mut img, 7, 21, 18, 6, Rgba([72, 79, 105, 255]));
    draw_circle(&mut img, 16, 15, 10, Rgba([65, 78, 118, 255]));
    draw_circle(&mut img, 16, 15, 7, Rgba([24, 45, 88, 255]));
    draw_circle(&mut img, 16, 15, 4, Rgba([65, 180, 235, 255]));
    draw_circle(&mut img, 16, 15, 2, Rgba([220, 250, 255, 255]));
    for (x, y) in [(7, 9), (25, 9), (7, 21), (25, 21)] {
        draw_rect(&mut img, x, y, 3, 5, Rgba([145, 155, 185, 255]));
    }
    draw_line(&mut img, 10, 7, 13, 11, Rgba([130, 220, 255, 255]));
    draw_line(&mut img, 22, 7, 19, 11, Rgba([130, 220, 255, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_server_bank() -> RgbaImage {
    let mut img = create_tile_base(Rgba([30, 35, 45, 255]));
    add_noise(&mut img, 6, 1717);
    // Racks
    draw_rect(&mut img, 6, 8, 8, 16, Rgba([60, 65, 80, 255]));
    draw_rect(&mut img, 18, 8, 8, 16, Rgba([60, 65, 80, 255]));
    // Status lights
    draw_rect(&mut img, 8, 10, 2, 2, Rgba([0, 220, 180, 255]));
    draw_rect(&mut img, 20, 12, 2, 2, Rgba([0, 180, 255, 255]));
    draw_rect(&mut img, 8, 16, 2, 2, Rgba([0, 220, 180, 255]));
    draw_rect(&mut img, 20, 18, 2, 2, Rgba([0, 180, 255, 255]));
    // Base plate
    draw_rect(&mut img, 6, 24, 20, 4, Rgba([80, 85, 95, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_sweeper() -> RgbaImage {
    let mut img = create_tile_base(Rgba([35, 40, 50, 255]));
    add_noise(&mut img, 6, 1818);
    // Chassis
    draw_rect(&mut img, 8, 18, 16, 8, Rgba([70, 75, 90, 255]));
    // Brushes
    draw_rect(&mut img, 8, 26, 4, 2, Rgba([200, 170, 60, 255]));
    draw_rect(&mut img, 14, 26, 4, 2, Rgba([200, 170, 60, 255]));
    draw_rect(&mut img, 20, 26, 4, 2, Rgba([200, 170, 60, 255]));
    // Sensor dome
    draw_circle(&mut img, 16, 14, 5, Rgba([30, 140, 180, 255]));
    draw_circle(&mut img, 16, 14, 2, Rgba([180, 240, 255, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_heater_node() -> RgbaImage {
    let mut img = create_tile_base(Rgba([48, 34, 32, 255]));
    add_noise(&mut img, 6, 2121);
    // Element stack, glowing from the middle out
    draw_rect(&mut img, 9, 9, 14, 14, Rgba([80, 60, 55, 255]));
    draw_rect(&mut img, 11, 11, 10, 10, Rgba([180, 80, 40, 255]));
    draw_circle(&mut img, 16, 16, 3, Rgba([255, 190, 110, 255]));
    // Vents
    draw_rect(&mut img, 7, 6, 18, 2, Rgba([110, 80, 70, 255]));
    draw_rect(&mut img, 7, 24, 18, 2, Rgba([110, 80, 70, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_shield_generator() -> RgbaImage {
    let mut img = create_tile_base(Rgba([32, 40, 48, 255]));
    add_noise(&mut img, 6, 2222);
    // Emitter base
    draw_rect(&mut img, 11, 20, 10, 7, Rgba([70, 80, 95, 255]));
    // Dome
    draw_circle(&mut img, 16, 15, 8, Rgba([40, 90, 130, 200]));
    draw_circle(&mut img, 16, 15, 5, Rgba([90, 190, 230, 220]));
    draw_circle(&mut img, 16, 15, 2, Rgba([220, 250, 255, 255]));
    // Mast
    draw_rect(&mut img, 15, 4, 2, 5, Rgba([180, 190, 200, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_smelter() -> RgbaImage {
    let mut img = create_tile_base(Rgba([44, 38, 34, 255]));
    add_noise(&mut img, 6, 2323);
    // Furnace body
    draw_rect(&mut img, 7, 12, 18, 14, Rgba([76, 68, 62, 255]));
    // Mouth, with the melt showing through
    draw_rect(&mut img, 11, 18, 10, 6, Rgba([200, 90, 30, 255]));
    draw_rect(&mut img, 13, 20, 6, 3, Rgba([255, 200, 120, 255]));
    // Chimney and its plume
    draw_rect(&mut img, 19, 5, 5, 8, Rgba([90, 82, 76, 255]));
    draw_circle(&mut img, 21, 4, 2, Rgba([120, 115, 110, 200]));
    // Poured ingot
    draw_rect(&mut img, 8, 26, 7, 3, Rgba([210, 150, 80, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_assembler() -> RgbaImage {
    let mut img = create_tile_base(Rgba([30, 39, 48, 255]));
    add_noise(&mut img, 5, 2626);

    // Heavy bed and two opposed tool carriages make it visually distinct
    // from the furnace and server racks at map scale.
    draw_rect(&mut img, 5, 22, 22, 6, Rgba([72, 82, 94, 255]));
    draw_rect(&mut img, 6, 8, 6, 15, Rgba([58, 82, 96, 255]));
    draw_rect(&mut img, 20, 8, 6, 15, Rgba([58, 82, 96, 255]));
    draw_line(&mut img, 9, 11, 14, 16, Rgba([40, 205, 235, 255]));
    draw_line(&mut img, 23, 11, 18, 16, Rgba([40, 205, 235, 255]));
    draw_circle(&mut img, 9, 10, 2, Rgba([155, 235, 250, 255]));
    draw_circle(&mut img, 23, 10, 2, Rgba([155, 235, 250, 255]));

    // The finished hexagonal component sits between the paired tools.
    draw_circle(&mut img, 16, 18, 5, Rgba([198, 142, 68, 255]));
    draw_circle(&mut img, 16, 18, 2, Rgba([250, 220, 150, 255]));
    draw_rect(&mut img, 14, 5, 4, 5, Rgba([90, 105, 120, 255]));
    draw_rect(&mut img, 15, 6, 2, 2, Rgba([30, 220, 190, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_mass_driver() -> RgbaImage {
    let mut img = create_tile_base(Rgba([34, 38, 46, 255]));
    add_noise(&mut img, 6, 2424);
    // Rail, climbing away to the north-east so the tile reads as a launcher
    // rather than another box.
    draw_line(&mut img, 5, 26, 27, 8, Rgba([150, 158, 172, 255]));
    draw_line(&mut img, 6, 28, 28, 10, Rgba([110, 118, 132, 255]));
    // Accelerator coils along the rail
    for (x, y) in [(9, 23), (14, 19), (19, 15), (24, 11)] {
        draw_circle(&mut img, x, y, 2, Rgba([70, 150, 200, 255]));
    }
    // Muzzle glow at the top of the run
    draw_circle(&mut img, 27, 8, 3, Rgba([120, 220, 255, 220]));
    draw_circle(&mut img, 27, 8, 1, Rgba([235, 250, 255, 255]));
    // Breech and its footing
    draw_rect(&mut img, 3, 22, 8, 8, Rgba([72, 78, 92, 255]));
    draw_rect(&mut img, 5, 24, 4, 4, Rgba([200, 140, 60, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_landing_pad() -> RgbaImage {
    let mut img = create_tile_base(Rgba([36, 40, 44, 255]));
    add_noise(&mut img, 5, 2525);
    // Apron, with the scorched ring a pod leaves behind
    draw_circle(&mut img, 16, 16, 12, Rgba([58, 62, 70, 255]));
    draw_circle(&mut img, 16, 16, 10, Rgba([46, 50, 58, 255]));
    // Approach markings
    draw_line(&mut img, 16, 6, 16, 11, Rgba([210, 170, 70, 255]));
    draw_line(&mut img, 16, 21, 16, 26, Rgba([210, 170, 70, 255]));
    draw_line(&mut img, 6, 16, 11, 16, Rgba([210, 170, 70, 255]));
    draw_line(&mut img, 21, 16, 26, 16, Rgba([210, 170, 70, 255]));
    // Catch cradle
    draw_rect(&mut img, 12, 12, 8, 8, Rgba([84, 90, 102, 255]));
    draw_rect(&mut img, 14, 14, 4, 4, Rgba([120, 200, 230, 255]));
    // Corner beacons
    for (x, y) in [(7, 7), (24, 7), (7, 24), (24, 24)] {
        draw_circle(&mut img, x, y, 1, Rgba([120, 220, 255, 255]));
    }
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_storage() -> RgbaImage {
    let mut img = create_tile_base(Rgba([40, 45, 55, 255]));
    add_noise(&mut img, 6, 1919);
    // Crate stack
    draw_rect(&mut img, 6, 16, 8, 10, Rgba([110, 90, 60, 255]));
    draw_rect(&mut img, 16, 14, 10, 12, Rgba([120, 100, 70, 255]));
    draw_rect(&mut img, 8, 10, 8, 6, Rgba([90, 75, 55, 255]));
    // Straps
    draw_rect(&mut img, 6, 20, 8, 1, Rgba([60, 60, 60, 255]));
    draw_rect(&mut img, 16, 20, 10, 1, Rgba([60, 60, 60, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_biomass_harvester() -> RgbaImage {
    let mut img = create_tile_base(Rgba([30, 50, 35, 255]));
    add_noise(&mut img, 6, 2020);
    // Bio vats
    draw_rect(&mut img, 6, 10, 8, 12, Rgba([40, 120, 60, 255]));
    draw_rect(&mut img, 18, 12, 8, 10, Rgba([40, 120, 60, 255]));
    // Glow cores
    draw_circle(&mut img, 10, 16, 3, Rgba([120, 220, 120, 255]));
    draw_circle(&mut img, 22, 16, 3, Rgba([120, 220, 120, 255]));
    // Pipes
    draw_rect(&mut img, 8, 24, 16, 2, Rgba([80, 90, 80, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}
