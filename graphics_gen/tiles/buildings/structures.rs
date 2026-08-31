//! Utility, power, factory, and freight building tile art.

use super::super::super::utils::*;
use super::super::save_building_with_icon;
use image::{Rgba, RgbaImage};

pub(super) fn generate() {
    save_building_with_icon("building_bridge", create_bridge());
    save_building_with_icon("building_power_node", create_power_node());
    save_building_with_icon("building_wind_turbine", create_wind_turbine());
    save_building_with_icon("building_geothermal_tap", create_geothermal_tap());
    save_building_with_icon("building_fusion_reactor", create_fusion_reactor());
    save_building_with_icon("building_server_bank", create_server_bank());
    save_building_with_icon("building_sweeper", create_sweeper());
    save_building_with_icon("building_heater_node", create_heater_node());
    save_building_with_icon("building_shield_generator", create_shield_generator());
    save_building_with_icon("building_smelter", create_smelter());
    save_building_with_icon("building_assembler", create_assembler());
    save_building_with_icon("building_mass_driver", create_mass_driver());
    save_building_with_icon("building_landing_pad", create_landing_pad());
    save_building_with_icon("building_storage", create_storage());
    save_building_with_icon("building_biomass_harvester", create_biomass_harvester());
}

fn create_bridge() -> RgbaImage {
    let mut img = create_tile_base(Rgba([40, 42, 48, 255]));
    add_noise(&mut img, 6, 1414);
    draw_line(&mut img, 6, 6, 26, 26, Rgba([170, 170, 180, 255]));
    draw_line(&mut img, 26, 6, 6, 26, Rgba([170, 170, 180, 255]));
    draw_rect(&mut img, 8, 8, 16, 16, Rgba([80, 85, 95, 200]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_power_node() -> RgbaImage {
    let mut img = create_tile_base(Rgba([38, 42, 50, 255]));
    add_noise(&mut img, 6, 1515);
    draw_circle(&mut img, 16, 16, 7, Rgba([30, 120, 180, 255]));
    draw_circle(&mut img, 16, 16, 4, Rgba([120, 220, 255, 255]));
    draw_rect(&mut img, 15, 5, 2, 6, Rgba([170, 170, 190, 255]));
    draw_circle(&mut img, 16, 4, 2, Rgba([120, 220, 255, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_wind_turbine() -> RgbaImage {
    let mut img = create_tile_base(Rgba([40, 45, 55, 255]));
    add_noise(&mut img, 5, 1616);
    draw_circle(&mut img, 16, 16, 3, Rgba([200, 200, 210, 255]));
    draw_line(&mut img, 16, 16, 26, 10, Rgba([210, 210, 220, 255]));
    draw_line(&mut img, 16, 16, 10, 26, Rgba([210, 210, 220, 255]));
    draw_line(&mut img, 16, 16, 6, 12, Rgba([210, 210, 220, 255]));
    draw_rect(&mut img, 14, 22, 4, 6, Rgba([90, 95, 105, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_geothermal_tap() -> RgbaImage {
    let mut img = create_tile_base(Rgba([52, 35, 28, 255]));
    add_noise(&mut img, 6, 2727);
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
    draw_rect(&mut img, 6, 8, 8, 16, Rgba([60, 65, 80, 255]));
    draw_rect(&mut img, 18, 8, 8, 16, Rgba([60, 65, 80, 255]));
    draw_rect(&mut img, 8, 10, 2, 2, Rgba([0, 220, 180, 255]));
    draw_rect(&mut img, 20, 12, 2, 2, Rgba([0, 180, 255, 255]));
    draw_rect(&mut img, 8, 16, 2, 2, Rgba([0, 220, 180, 255]));
    draw_rect(&mut img, 20, 18, 2, 2, Rgba([0, 180, 255, 255]));
    draw_rect(&mut img, 6, 24, 20, 4, Rgba([80, 85, 95, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_sweeper() -> RgbaImage {
    let mut img = create_tile_base(Rgba([35, 40, 50, 255]));
    add_noise(&mut img, 6, 1818);
    draw_rect(&mut img, 8, 18, 16, 8, Rgba([70, 75, 90, 255]));
    draw_rect(&mut img, 8, 26, 4, 2, Rgba([200, 170, 60, 255]));
    draw_rect(&mut img, 14, 26, 4, 2, Rgba([200, 170, 60, 255]));
    draw_rect(&mut img, 20, 26, 4, 2, Rgba([200, 170, 60, 255]));
    draw_circle(&mut img, 16, 14, 5, Rgba([30, 140, 180, 255]));
    draw_circle(&mut img, 16, 14, 2, Rgba([180, 240, 255, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_heater_node() -> RgbaImage {
    let mut img = create_tile_base(Rgba([48, 34, 32, 255]));
    add_noise(&mut img, 6, 2121);
    draw_rect(&mut img, 9, 9, 14, 14, Rgba([80, 60, 55, 255]));
    draw_rect(&mut img, 11, 11, 10, 10, Rgba([180, 80, 40, 255]));
    draw_circle(&mut img, 16, 16, 3, Rgba([255, 190, 110, 255]));
    draw_rect(&mut img, 7, 6, 18, 2, Rgba([110, 80, 70, 255]));
    draw_rect(&mut img, 7, 24, 18, 2, Rgba([110, 80, 70, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_shield_generator() -> RgbaImage {
    let mut img = create_tile_base(Rgba([32, 40, 48, 255]));
    add_noise(&mut img, 6, 2222);
    draw_rect(&mut img, 11, 20, 10, 7, Rgba([70, 80, 95, 255]));
    draw_circle(&mut img, 16, 15, 8, Rgba([40, 90, 130, 200]));
    draw_circle(&mut img, 16, 15, 5, Rgba([90, 190, 230, 220]));
    draw_circle(&mut img, 16, 15, 2, Rgba([220, 250, 255, 255]));
    draw_rect(&mut img, 15, 4, 2, 5, Rgba([180, 190, 200, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_smelter() -> RgbaImage {
    let mut img = create_tile_base(Rgba([44, 38, 34, 255]));
    add_noise(&mut img, 6, 2323);
    draw_rect(&mut img, 7, 12, 18, 14, Rgba([76, 68, 62, 255]));
    draw_rect(&mut img, 11, 18, 10, 6, Rgba([200, 90, 30, 255]));
    draw_rect(&mut img, 13, 20, 6, 3, Rgba([255, 200, 120, 255]));
    draw_rect(&mut img, 19, 5, 5, 8, Rgba([90, 82, 76, 255]));
    draw_circle(&mut img, 21, 4, 2, Rgba([120, 115, 110, 200]));
    draw_rect(&mut img, 8, 26, 7, 3, Rgba([210, 150, 80, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_assembler() -> RgbaImage {
    let mut img = create_tile_base(Rgba([30, 39, 48, 255]));
    add_noise(&mut img, 5, 2626);
    draw_rect(&mut img, 5, 22, 22, 6, Rgba([72, 82, 94, 255]));
    draw_rect(&mut img, 6, 8, 6, 15, Rgba([58, 82, 96, 255]));
    draw_rect(&mut img, 20, 8, 6, 15, Rgba([58, 82, 96, 255]));
    draw_line(&mut img, 9, 11, 14, 16, Rgba([40, 205, 235, 255]));
    draw_line(&mut img, 23, 11, 18, 16, Rgba([40, 205, 235, 255]));
    draw_circle(&mut img, 9, 10, 2, Rgba([155, 235, 250, 255]));
    draw_circle(&mut img, 23, 10, 2, Rgba([155, 235, 250, 255]));
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
    draw_line(&mut img, 5, 26, 27, 8, Rgba([150, 158, 172, 255]));
    draw_line(&mut img, 6, 28, 28, 10, Rgba([110, 118, 132, 255]));
    for (x, y) in [(9, 23), (14, 19), (19, 15), (24, 11)] {
        draw_circle(&mut img, x, y, 2, Rgba([70, 150, 200, 255]));
    }
    draw_circle(&mut img, 27, 8, 3, Rgba([120, 220, 255, 220]));
    draw_circle(&mut img, 27, 8, 1, Rgba([235, 250, 255, 255]));
    draw_rect(&mut img, 3, 22, 8, 8, Rgba([72, 78, 92, 255]));
    draw_rect(&mut img, 5, 24, 4, 4, Rgba([200, 140, 60, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_landing_pad() -> RgbaImage {
    let mut img = create_tile_base(Rgba([36, 40, 44, 255]));
    add_noise(&mut img, 5, 2525);
    draw_circle(&mut img, 16, 16, 12, Rgba([58, 62, 70, 255]));
    draw_circle(&mut img, 16, 16, 10, Rgba([46, 50, 58, 255]));
    draw_line(&mut img, 16, 6, 16, 11, Rgba([210, 170, 70, 255]));
    draw_line(&mut img, 16, 21, 16, 26, Rgba([210, 170, 70, 255]));
    draw_line(&mut img, 6, 16, 11, 16, Rgba([210, 170, 70, 255]));
    draw_line(&mut img, 21, 16, 26, 16, Rgba([210, 170, 70, 255]));
    draw_rect(&mut img, 12, 12, 8, 8, Rgba([84, 90, 102, 255]));
    draw_rect(&mut img, 14, 14, 4, 4, Rgba([120, 200, 230, 255]));
    for (x, y) in [(7, 7), (24, 7), (7, 24), (24, 24)] {
        draw_circle(&mut img, x, y, 1, Rgba([120, 220, 255, 255]));
    }
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_storage() -> RgbaImage {
    let mut img = create_tile_base(Rgba([40, 45, 55, 255]));
    add_noise(&mut img, 6, 1919);
    draw_rect(&mut img, 6, 16, 8, 10, Rgba([110, 90, 60, 255]));
    draw_rect(&mut img, 16, 14, 10, 12, Rgba([120, 100, 70, 255]));
    draw_rect(&mut img, 8, 10, 8, 6, Rgba([90, 75, 55, 255]));
    draw_rect(&mut img, 6, 20, 8, 1, Rgba([60, 60, 60, 255]));
    draw_rect(&mut img, 16, 20, 10, 1, Rgba([60, 60, 60, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_biomass_harvester() -> RgbaImage {
    let mut img = create_tile_base(Rgba([30, 50, 35, 255]));
    add_noise(&mut img, 6, 2020);
    draw_rect(&mut img, 6, 10, 8, 12, Rgba([40, 120, 60, 255]));
    draw_rect(&mut img, 18, 12, 8, 10, Rgba([40, 120, 60, 255]));
    draw_circle(&mut img, 10, 16, 3, Rgba([120, 220, 120, 255]));
    draw_circle(&mut img, 22, 16, 3, Rgba([120, 220, 120, 255]));
    draw_rect(&mut img, 8, 24, 16, 2, Rgba([80, 90, 80, 255]));
    add_edge_darkening(&mut img, 2, 8);
    img
}
