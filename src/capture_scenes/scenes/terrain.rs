//! Screenshot staging for terrain and named planetary features.

use crate::{engine, state, Game, GamePhase};

impl Game {
    pub(super) fn stage_feature_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        self.seed_logistics_scene();
        prepare_feature_inspection(self.campaign.current_mut());
    }

    pub(super) fn stage_iron_belt_inspection_scene(&mut self) {
        self.phase = GamePhase::Playing;
        // Keep this a real campaign transition: the Iron Belt belongs to
        // Mercury, so the still first reaches that world instead of swapping
        // the current planet's identity underneath the UI.
        self.seed_logistics_scene();
        self.campaign.colonize(0);
        self.campaign.travel_to(0);
        prepare_feature_inspection(self.campaign.current_mut());
    }
}

fn prepare_feature_inspection(planet: &mut state::PlanetState) {
    planet.tutorial_hidden = true;
    planet.tutorial_done = true;
    // The seeded network is large enough to awaken the first Core stage
    // on the next frame. Settle that authored milestone here so the
    // feature dossier, not a one-time progression toast, owns the still.
    planet.core_stage = 1;
    planet.refresh_stats();
    planet.notifications.clear();

    let Some(feature) = planet.features.first().cloned() else {
        return;
    };
    let selected = (0..planet.grid.height)
        .flat_map(|y| (0..planet.grid.width).map(move |x| engine::GridPos::new(x as i32, y as i32)))
        .find(|pos| {
            feature.contains_tile(*pos, planet.grid.width, planet.grid.height)
                && planet.grid.get(*pos).is_some_and(|tile| {
                    tile.building.is_none()
                        && (feature.id != "iron_belt" || tile.terrain.is_buildable(planet.data()))
                })
        })
        .or_else(|| {
            (0..planet.grid.height)
                .flat_map(|y| {
                    (0..planet.grid.width).map(move |x| engine::GridPos::new(x as i32, y as i32))
                })
                .find(|pos| {
                    feature.contains_tile(*pos, planet.grid.width, planet.grid.height)
                        && planet
                            .grid
                            .get(*pos)
                            .is_some_and(|tile| tile.building.is_none())
                })
        });
    let Some(pos) = selected else {
        return;
    };
    let feature_radius = (feature.bounds.2.max(feature.bounds.3)
        * planet.grid.width.max(planet.grid.height) as f32)
        .ceil() as u32
        + 1;
    planet.grid.reveal_around(pos, feature_radius.max(3));
    planet.selected_building = None;
    planet.selected_tile = Some(pos);
    planet.mobile_panel = state::MobilePanel::Info;
    planet.camera.zoom = 1.2;
}
