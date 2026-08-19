//! Texture loading and access helpers.

use crate::data;
use macroquad::prelude::*;
use macroquad_toolkit::assets::{load_texture_from_pack_or_file, AssetPack};
use std::collections::HashMap;

const ASSET_PACK_PATH: &str = "assets.zip";

pub struct TerrainTextures {
    pub by_id: HashMap<String, Texture2D>,
}

pub struct BuildingTextures {
    pub by_id: HashMap<String, Texture2D>,
    pub core_stage_1a: Texture2D,
    pub core_stage_1b: Texture2D,
    pub core_stage_1c: Texture2D,
    pub core_stage_2a: Texture2D,
    pub core_stage_2b: Texture2D,
    pub core_stage_3a: Texture2D,
    pub core_stage_3b: Texture2D,
    pub core_stage_4a: Texture2D,
    pub core_stage_4b: Texture2D,
    pub conduit_straight_h: Texture2D,
    pub conduit_straight_v: Texture2D,
    pub conduit_corner_ne: Texture2D,
    pub conduit_corner_nw: Texture2D,
    pub conduit_corner_se: Texture2D,
    pub conduit_corner_sw: Texture2D,
    pub conduit_tee_n: Texture2D,
    pub conduit_tee_e: Texture2D,
    pub conduit_tee_s: Texture2D,
    pub conduit_tee_w: Texture2D,
    pub conduit_cross: Texture2D,
}

pub struct BuildingIconTextures {
    pub by_id: HashMap<String, Texture2D>,
}

pub struct GameTextures {
    pub terrain: TerrainTextures,
    pub buildings: BuildingTextures,
    pub building_icons: BuildingIconTextures,
}

impl GameTextures {
    pub async fn load() -> Self {
        let asset_pack = AssetPack::load(ASSET_PACK_PATH).await.ok();
        let mut terrain_textures = HashMap::new();
        for def in &data::game_data().terrain {
            let texture =
                load_required_texture(asset_pack.as_ref(), &def.texture, "terrain_texture").await;
            terrain_textures.insert(def.id.clone(), texture);
        }
        let terrain = TerrainTextures {
            by_id: terrain_textures,
        };

        let mut building_textures = HashMap::new();
        for def in &data::game_data().buildings {
            let texture =
                load_required_texture(asset_pack.as_ref(), &def.texture, "building_texture").await;
            building_textures.insert(def.id.clone(), texture);
        }

        let mut building_icon_textures = HashMap::new();
        for def in &data::game_data().buildings {
            let icon_path = def.icon.as_deref().unwrap_or(&def.texture);
            let texture = match load_texture_from_pack_or_file(
                asset_pack.as_ref(),
                icon_path,
                FilterMode::Nearest,
            )
            .await
            {
                Ok(texture) => texture,
                Err(_) => {
                    println!("Icon missing for {}. Falling back to tile texture.", def.id);
                    building_textures
                        .get(&def.id)
                        .expect("building_icon_fallback")
                        .clone()
                }
            };
            building_icon_textures.insert(def.id.clone(), texture);
        }

        let buildings = BuildingTextures {
            by_id: building_textures,
            core_stage_1a: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_core_stage_1a.png",
                "building_core_stage_1a",
            )
            .await,
            core_stage_1b: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_core_stage_1b.png",
                "building_core_stage_1b",
            )
            .await,
            core_stage_1c: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_core_stage_1c.png",
                "building_core_stage_1c",
            )
            .await,
            core_stage_2a: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_core_stage_2a.png",
                "building_core_stage_2a",
            )
            .await,
            core_stage_2b: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_core_stage_2b.png",
                "building_core_stage_2b",
            )
            .await,
            core_stage_3a: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_core_stage_3a.png",
                "building_core_stage_3a",
            )
            .await,
            core_stage_3b: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_core_stage_3b.png",
                "building_core_stage_3b",
            )
            .await,
            core_stage_4a: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_core_stage_4a.png",
                "building_core_stage_4a",
            )
            .await,
            core_stage_4b: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_core_stage_4b.png",
                "building_core_stage_4b",
            )
            .await,
            conduit_straight_h: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_straight_h.png",
                "conduit_straight_h",
            )
            .await,
            conduit_straight_v: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_straight_v.png",
                "conduit_straight_v",
            )
            .await,
            conduit_corner_ne: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_corner_ne.png",
                "conduit_corner_ne",
            )
            .await,
            conduit_corner_nw: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_corner_nw.png",
                "conduit_corner_nw",
            )
            .await,
            conduit_corner_se: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_corner_se.png",
                "conduit_corner_se",
            )
            .await,
            conduit_corner_sw: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_corner_sw.png",
                "conduit_corner_sw",
            )
            .await,
            conduit_tee_n: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_tee_n.png",
                "conduit_tee_n",
            )
            .await,
            conduit_tee_e: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_tee_e.png",
                "conduit_tee_e",
            )
            .await,
            conduit_tee_s: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_tee_s.png",
                "conduit_tee_s",
            )
            .await,
            conduit_tee_w: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_tee_w.png",
                "conduit_tee_w",
            )
            .await,
            conduit_cross: load_required_texture(
                asset_pack.as_ref(),
                "assets/tiles/buildings/building_conduit_cross.png",
                "conduit_cross",
            )
            .await,
        };

        let building_icons = BuildingIconTextures {
            by_id: building_icon_textures,
        };

        set_filter_nearest(&terrain);
        set_filter_nearest_buildings(&buildings);
        set_filter_nearest_icons(&building_icons);

        Self {
            terrain,
            buildings,
            building_icons,
        }
    }
}

async fn load_required_texture(
    asset_pack: Option<&AssetPack>,
    path: &str,
    label: &str,
) -> Texture2D {
    load_texture_from_pack_or_file(asset_pack, path, FilterMode::Nearest)
        .await
        .expect(label)
}

fn set_filter_nearest(terrain: &TerrainTextures) {
    for texture in terrain.by_id.values() {
        texture.set_filter(FilterMode::Nearest);
    }
}

fn set_filter_nearest_buildings(buildings: &BuildingTextures) {
    for texture in buildings.by_id.values() {
        texture.set_filter(FilterMode::Nearest);
    }
    buildings.core_stage_1a.set_filter(FilterMode::Nearest);
    buildings.core_stage_1b.set_filter(FilterMode::Nearest);
    buildings.core_stage_1c.set_filter(FilterMode::Nearest);
    buildings.core_stage_2a.set_filter(FilterMode::Nearest);
    buildings.core_stage_2b.set_filter(FilterMode::Nearest);
    buildings.core_stage_3a.set_filter(FilterMode::Nearest);
    buildings.core_stage_3b.set_filter(FilterMode::Nearest);
    buildings.core_stage_4a.set_filter(FilterMode::Nearest);
    buildings.core_stage_4b.set_filter(FilterMode::Nearest);
    buildings.conduit_straight_h.set_filter(FilterMode::Nearest);
    buildings.conduit_straight_v.set_filter(FilterMode::Nearest);
    buildings.conduit_corner_ne.set_filter(FilterMode::Nearest);
    buildings.conduit_corner_nw.set_filter(FilterMode::Nearest);
    buildings.conduit_corner_se.set_filter(FilterMode::Nearest);
    buildings.conduit_corner_sw.set_filter(FilterMode::Nearest);
    buildings.conduit_tee_n.set_filter(FilterMode::Nearest);
    buildings.conduit_tee_e.set_filter(FilterMode::Nearest);
    buildings.conduit_tee_s.set_filter(FilterMode::Nearest);
    buildings.conduit_tee_w.set_filter(FilterMode::Nearest);
    buildings.conduit_cross.set_filter(FilterMode::Nearest);
}

fn set_filter_nearest_icons(icons: &BuildingIconTextures) {
    for texture in icons.by_id.values() {
        texture.set_filter(FilterMode::Nearest);
    }
}
