//! Tile generation for Nanite Swarm (2D)

mod buildings;
mod terrain;

use image::imageops::FilterType;
use image::RgbaImage;

const ICON_WIDTH: u32 = 48;
const ICON_HEIGHT: u32 = 48;

pub fn generate_tiles() {
    terrain::generate();
    buildings::generate();
}

fn save_tile(name: &str, img: RgbaImage) {
    let path = format!("assets/tiles/{}.png", name);
    img.save(&path).unwrap();
    println!("Generated: {}", path);
}

fn save_building(name: &str, img: RgbaImage) {
    let path = format!("assets/tiles/buildings/{}.png", name);
    img.save(&path).unwrap();
    println!("Generated: {}", path);
}

fn save_building_with_icon(name: &str, img: RgbaImage) {
    let icon = scale_to_icon(&img);
    save_building(name, img);
    save_building_icon(name, icon);
}

fn save_building_icon(name: &str, img: RgbaImage) {
    let path = format!("assets/ui/buildings/ui_{}.png", name);
    img.save(&path).unwrap();
    println!("Generated: {}", path);
}

fn scale_to_icon(img: &RgbaImage) -> RgbaImage {
    image::imageops::resize(img, ICON_WIDTH, ICON_HEIGHT, FilterType::Nearest)
}
