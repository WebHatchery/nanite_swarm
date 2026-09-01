//! Planet construction and runtime data reattachment.

use super::*;

impl PlanetState {
    /// Build the world at campaign slot `planet_index`, from its entry in
    /// `assets/planets.json`.
    pub fn new(planet_index: usize, seed: u64, config: GameConfig) -> Self {
        Self::new_with_data(planet_index, seed, config, GameDataContext::default())
    }

    pub fn new_with_data(
        planet_index: usize,
        seed: u64,
        config: GameConfig,
        data: GameDataContext,
    ) -> Self {
        let def = data.data().planet(planet_index);
        let (name, width, height) = (def.name.as_str(), def.width, def.height);
        let mut grid = Grid::generate(
            width,
            height,
            seed,
            &def.terrain,
            &config.ore,
            config.grid.min_start_region,
        );
        grid.attach_data(data.clone());
        grid.initialize_forest_biomass(config.resources.forest_biomass);
        let mut features = features::generated_features(def, seed);
        features::anchor_effect_features(&mut features, def, &grid);

        // Place Core at center
        let center = GridPos::new(width as i32 / 2, height as i32 / 2);
        grid.place_building(center, BuildingType::Core);
        grid.update_power_grid();

        let mut unlocked_buildings = Vec::new();
        for def in &data.data().buildings {
            if def.start_unlocked {
                if let Some(building_type) = BuildingType::from_id(&def.id) {
                    unlocked_buildings.push(building_type);
                }
            }
        }

        let mut state = Self {
            data: data.clone(),
            name: name.to_string(),
            planet_index,
            map_seed: seed,
            resources: Resources {
                energy: config.resources.starting_energy,
                minerals: config.resources.starting_minerals,
                data: 0.0,
                biomass: 0.0,
                alloy: 0.0,
                components: 0.0,
                consumed: ConsumptionTotals::default(),
            },
            grid,
            drones: DroneManager::new(
                config.resources.drone_carry_capacity,
                config.resources.drone_speed,
            ),
            research: ResearchProgress {
                unlocked_techs: data.data().research.starting_unlocked.clone(),
                ..ResearchProgress::default()
            },
            config,
            time_played: 0.0,
            selected_building: Some(BuildingType::Drill),
            // The starting Core produces four power. Keeping the cached value
            // honest avoids one-frame milestone and HUD spikes before the
            // first simulation tick recomputes it.
            power_balance: 4.0,
            biomass_power_bonus: 0.0,
            sim_accumulator: 0.0,
            paused: false,
            time_scale: default_time_scale(),
            battery_seconds: 4.0 * 60.0 * 60.0,
            last_saved_unix: unix_seconds_now(),
            achievements: Achievements::from_definitions(achievement_definitions(data.data())),
            stats: Stats::default(),
            throughput: default_throughput(),
            delivered_since_sample: 0.0,
            throughput_timer: 0.0,
            core_stage: 0,
            power_negative_seconds: 0.0,
            power_collapse_cooldown: 0.0,
            power_collapse_shutdown: 0.0,
            power_collapse_length: 0.0,
            research_lock_timer: 0.0,
            collapse_notice_timer: 0.0,
            arrival_notice_timer: 0.0,
            save_notice_timer: 0.0,
            save_failed: false,
            restored_from_backup: false,
            restored_from_backup_generation: 0,
            forest_harvested_count: 0,
            tutorial_step: 0,
            tutorial_hidden: false,
            tutorial_done: false,
            unlocked_buildings,
            banned_buildings: def
                .banned_buildings
                .iter()
                .filter_map(|id| BuildingType::from_id(id))
                .collect(),
            hazards: def.hazards,
            features,
            seed_ship: SeedShip::with_data(data.clone()),
            export: None,
            export_cooldown: 0.0,
            pod_loads: std::collections::HashMap::new(),
            launched_pods: Vec::new(),
            pad_cargo: std::collections::HashMap::new(),
            last_offline_seconds: 0.0,
            last_offline_simulated: 0.0,
            offline_notice_timer: 0.0,
            drag_last_pos: None,
            route_resource_blocked_at: None,
            route_path_blocked_at: None,
            smart_drag_source: None,
            selected_tile: None,
            demolish_mode: false,
            camera: Camera::default(),
            touch_gesture: TouchGesture::new(),
            touch_camera_active: false,
            touch_route_active: false,
            touch_gesture_routed: false,
            camera_drag_anchor: None,
            show_help: false,
            mobile_panel: super::super::mobile::MobilePanel::Map,
            desktop_panel: super::super::mobile::DesktopPanel::None,
            focus_mode: true,
            flow_overlay: false,
            auto_clocking: false,
            factory_focus: FactoryFocus::Balanced,
            factory_deck_open: false,
            auto_clock_timer: 0.0,
            build_palette_scroll: ScrollArea::new(),
            seed_ship_scroll: ScrollArea::new(),
            log_scroll: ScrollArea::new(),
            records_scroll: ScrollArea::new(),
            notifications: NotificationManager::default(),
            particles: ParticleSystem::new(),
            particle_timer: 0.0,
            placement_anims: Vec::new(),
            output_buffers: std::collections::HashMap::new(),
            input_buffers: std::collections::HashMap::new(),
            input_hoppers: std::collections::HashMap::new(),
            traffic: std::collections::HashMap::new(),
            network_revision: 0,
            route_cache_revision: 0,
            drone_queues: std::collections::HashMap::new(),
            route_reservations: std::collections::HashMap::new(),
            collapse_history: Vec::new(),
            graph_samples: Vec::new(),
            graph_mode: GraphMode::Flow,
            factory_flow_since_sample: GraphSample::default(),
            last_offline_report: super::super::progress::OfflineReport::default(),
            audio_events: Vec::new(),
            blueprint: Vec::new(),
            relocation_source: None,
            box_select_mode: false,
            box_select_start: None,
            box_selected: Vec::new(),
            purge_armed: None,
            bulk_purge_armed: false,
            undo_history: Vec::new(),
        };
        state.refresh_stats();
        state
    }

    pub fn data(&self) -> &GameData {
        self.data.data()
    }

    pub(crate) fn attach_data(&mut self, data: GameDataContext) {
        self.grid.attach_data(data.clone());
        self.seed_ship.attach_data(data.clone());
        self.data = data;
    }
}
