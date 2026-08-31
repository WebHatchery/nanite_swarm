//! Procedural terrain tile art.

use super::super::utils::*;
use super::save_tile;
use image::{Rgba, RgbaImage};

pub fn generate() {
    save_tile("terrain_ground", create_ground());
    save_tile("terrain_mountain", create_mountain());
    save_tile("terrain_forest", create_forest());
    save_tile("terrain_water", create_water());
    save_tile("terrain_rough", create_rough());
    save_tile("terrain_void", create_void());
}

fn create_ground() -> RgbaImage {
    let mut img = create_tile_base(Rgba([90, 80, 70, 255]));
    add_noise(&mut img, 10, 101);
    for y in 0..TILE_HEIGHT {
        for x in 0..TILE_WIDTH {
            let n = hashed_noise(x, y, 303);
            if n.is_multiple_of(37) {
                img.put_pixel(x, y, Rgba([110, 100, 85, 255]));
            } else if n.is_multiple_of(53) {
                img.put_pixel(x, y, Rgba([70, 60, 50, 255]));
            }
        }
    }
    add_edge_darkening(&mut img, 3, 10);
    img
}

fn create_mountain() -> RgbaImage {
    let mut img = create_tile_base(Rgba([70, 70, 78, 255]));
    add_noise(&mut img, 12, 202);
    // Ridge lines
    for i in 0..6 {
        let x0 = 2 + (i * 5);
        let y0 = 6 + (i * 3);
        let x1 = x0 + 12;
        let y1 = y0 + 10;
        draw_line(&mut img, x0, y0, x1, y1, Rgba([95, 95, 105, 255]));
    }
    // Highlight top-left
    for y in 0..TILE_HEIGHT {
        for x in 0..TILE_WIDTH {
            if x + y < 18 {
                let p = *img.get_pixel(x, y);
                img.put_pixel(
                    x,
                    y,
                    Rgba([
                        p[0].saturating_add(15),
                        p[1].saturating_add(15),
                        p[2].saturating_add(18),
                        255,
                    ]),
                );
            }
        }
    }
    add_edge_darkening(&mut img, 4, 12);
    img
}

fn create_forest() -> RgbaImage {
    let mut img = create_tile_base(Rgba([30, 70, 50, 255]));
    add_noise(&mut img, 12, 303);
    for y in 0..TILE_HEIGHT {
        for x in 0..TILE_WIDTH {
            let n = hashed_noise(x, y, 707);
            if n.is_multiple_of(11) {
                img.put_pixel(x, y, Rgba([40, 95, 70, 255]));
            } else if n.is_multiple_of(17) {
                img.put_pixel(x, y, Rgba([20, 55, 40, 255]));
            }
        }
    }
    // Glowing spores
    for i in 0..12 {
        let x = (hashed_noise(i, i * 3, 909) % TILE_WIDTH) as i32;
        let y = (hashed_noise(i * 7, i, 909) % TILE_HEIGHT) as i32;
        draw_circle(&mut img, x, y, 1, Rgba([120, 200, 150, 220]));
    }
    add_edge_darkening(&mut img, 3, 10);
    img
}

fn create_water() -> RgbaImage {
    let mut img = create_tile_base(Rgba([25, 55, 95, 255]));
    for y in 0..TILE_HEIGHT {
        for x in 0..TILE_WIDTH {
            let t = (x as f32 * 0.35 + y as f32 * 0.55) * 0.35;
            let wave = (t.sin() * 10.0) as i16;
            let p = *img.get_pixel(x, y);
            img.put_pixel(
                x,
                y,
                Rgba([
                    p[0].saturating_add(wave.max(0) as u8),
                    p[1].saturating_add((wave / 2).max(0) as u8),
                    p[2].saturating_add(wave.max(0) as u8),
                    255,
                ]),
            );
        }
    }
    add_noise(&mut img, 8, 404);
    add_edge_darkening(&mut img, 2, 8);
    img
}

fn create_rough() -> RgbaImage {
    let mut img = create_tile_base(Rgba([80, 70, 65, 255]));
    add_noise(&mut img, 14, 505);
    // Cracks
    for i in 0..6 {
        let x0 = (hashed_noise(i, i * 5, 606) % TILE_WIDTH) as i32;
        let y0 = (hashed_noise(i * 3, i, 606) % TILE_HEIGHT) as i32;
        let x1 = (x0 + 10).min(TILE_WIDTH as i32 - 1);
        let y1 = (y0 + 6).min(TILE_HEIGHT as i32 - 1);
        draw_line(&mut img, x0, y0, x1, y1, Rgba([45, 40, 38, 255]));
    }
    add_edge_darkening(&mut img, 3, 10);
    img
}

fn create_void() -> RgbaImage {
    let mut img = create_tile_base(Rgba([10, 10, 15, 255]));
    add_noise(&mut img, 8, 707);
    // Lava cracks
    for i in 0..8 {
        let x0 = (hashed_noise(i, i * 7, 808) % TILE_WIDTH) as i32;
        let y0 = (hashed_noise(i * 11, i, 808) % TILE_HEIGHT) as i32;
        let x1 = (x0 + 12).min(TILE_WIDTH as i32 - 1);
        let y1 = (y0 + 4).min(TILE_HEIGHT as i32 - 1);
        draw_line(&mut img, x0, y0, x1, y1, Rgba([220, 90, 20, 255]));
        draw_line(&mut img, x0, y0 + 1, x1, y1 + 1, Rgba([255, 140, 40, 200]));
    }
    add_edge_darkening(&mut img, 4, 16);
    img
}
