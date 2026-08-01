//! Scenes staged for the screenshot harness.
//!
//! `NANITE_SWARM_CAPTURE_PATH` boots the game straight into one of these,
//! simulates a fixed number of frames and writes a PNG, so a screen can be
//! looked at without playing up to it. Everything here exists to put the game
//! in a state worth photographing; none of it runs in a real session.

use crate::state::{Campaign, LaunchSequence};
use crate::{data, engine, state, Game, GamePhase};

impl Game {
    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.capture_still = true;
        match scene {
            "mainmenu" => self.phase = GamePhase::MainMenu,
            "research" => {
                self.phase = GamePhase::Research;
                // A node that both moves a stat and opens a building, so the
                // panel shows everything it can say about one.
                self.research_state.unlocked.push("power_grid".to_string());
                self.research_state
                    .unlocked
                    .push("data_processing".to_string());
                // Two techs that actually move numbers, so the swarm sheet
                // shows changed lines and not only untouched ones.
                self.research_state
                    .unlocked
                    .push("efficient_drills".to_string());
                self.research_state
                    .unlocked
                    .push("drone_capacity".to_string());
                self.research_state.current_research = Some("self_cleaning_servos".to_string());
                self.sync_research_to_planet();
                // Meet the standing directive for real, so the toast on this
                // screen is one the game actually raised.
                self.campaign.directive.completed = true;
                self.campaign.update_directive(0.1);
            }
            "logistics" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                // Select the drill's own tile, so the inspector is reading
                // ground rather than a palette choice.
                let planet = self.campaign.current_mut();
                let drill = planet
                    .grid
                    .find_buildings(engine::BuildingType::Drill)
                    .pop();
                planet.selected_tile = drill;
                // Nothing in hand: the quiet state of the ore overlay.
                planet.selected_building = None;
            }
            // The same world with a Drill in hand, which brings the ground up.
            "prospect" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                self.campaign
                    .current_mut()
                    .select_building(engine::BuildingType::Drill);
            }
            // A Core that has grown past its landing pod.
            "core_stage" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                if let Some(core) = planet.grid.find_core() {
                    planet.grid.reveal_around(core, 24);
                    // Enough standing to reach the Foundry and then the
                    // Fortress, through the real path.
                    for step in 1..=26 {
                        let pos =
                            engine::GridPos::new(core.x - 1 - step % 8, core.y + 1 + step / 8);
                        if let Some(tile) = planet.grid.get_mut(pos) {
                            tile.terrain = engine::TerrainType::Empty;
                        }
                        planet
                            .grid
                            .place_building(pos, engine::BuildingType::Conduit);
                    }
                }
                for tech in ["a", "b", "c", "d", "e", "f"] {
                    planet.research.unlocked_techs.push(tech.to_string());
                }
                planet.grid.update_power_grid();
                planet.step(state::TICK_SECONDS, false);
            }
            // A run cut in the middle, with drones stalled beyond the break.
            "severed" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                // Long enough for a drill to fill a load and put a drone out
                // on the run; cutting it before that strands nobody.
                for _ in 0..300 {
                    planet.step(state::TICK_SECONDS, false);
                }
                // ...then take a piece of it away.
                if let Some(core) = planet.grid.find_core() {
                    let cut = engine::GridPos::new(core.x + 3, core.y);
                    planet.grid.remove_building(cut);
                    planet.grid.update_power_grid();
                }
                for _ in 0..10 {
                    planet.step(state::TICK_SECONDS, false);
                }
            }
            // The same mid-build ship, seen from the ground it is eating.
            "skyline" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                for stage in &data::game_data().seed_ship.stages {
                    if let Some(tech) = stage.requires.as_deref() {
                        self.research_state.unlocked.push(tech.to_string());
                    }
                }
                self.sync_research_to_planet();
                let planet = self.campaign.current_mut();
                planet.config.resources.base_mineral_cap = 100_000.0;
                planet.resources.minerals = 100_000.0;
                planet.resources.data = 10_000.0;
                planet.resources.biomass = 10_000.0;
                planet.resources.alloy = 10_000.0;
                planet.toggle_seed_ship_commitment();
                for _ in 0..60 {
                    planet.update_seed_ship(1.0);
                }
            }
            "records" => {
                self.phase = GamePhase::Records;
                self.seed_logistics_scene();
                // A world part-way through the set, so the screen shows earned
                // rows, locked rows and a few half-finished bars at once.
                let planet = self.campaign.current_mut();
                planet.config.resources.base_mineral_cap = 100_000.0;
                planet.resources.minerals = 260.0;
                planet.resources.data = 30.0;
                planet.resources.alloy = 12.0;
                planet.forest_harvested_count = 2;
                for tech in ["power_grid", "data_processing", "efficient_drills"] {
                    planet.research.unlocked_techs.push(tech.to_string());
                }
                planet.refresh_stats();
                // A step is what fires them, the same as in play.
                for _ in 0..4 {
                    planet.step(state::TICK_SECONDS, false);
                }
            }
            "seedship" => {
                self.phase = GamePhase::SeedShip;
                self.seed_logistics_scene();
                self.campaign.current_mut().resources.alloy = 80.0;
                // Mid-build, with the swarm diverting production into the yard.
                let planet = self.campaign.current_mut();
                planet.config.resources.base_mineral_cap = 100_000.0;
                planet.resources.minerals = 400.0;
                planet.resources.data = 120.0;
                planet.toggle_seed_ship_commitment();
                for _ in 0..20 {
                    planet.update_seed_ship(1.0);
                }
            }
            // The two beats of a launch worth looking at in a still frame: the
            // ship clearing the world it was built on, and the one it reaches.
            "launch" | "arrival" => {
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                let mut sequence = LaunchSequence::new(0, 1);
                sequence.advance(if scene == "launch" {
                    LaunchSequence::beat_start(state::LaunchBeat::Ascent) + 1.2
                } else {
                    LaunchSequence::beat_start(state::LaunchBeat::Arrival) + 1.5
                });
                self.launch = Some(sequence);
                self.phase = GamePhase::Launch;
            }
            "venus" => {
                self.phase = GamePhase::Playing;
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                self.research_state
                    .unlocked
                    .push("ceramic_plating".to_string());
                self.research_state
                    .unlocked
                    .push("heater_nodes".to_string());
                let planet = self.campaign.current_mut();
                // Everything researched, so the palette shows what this world
                // refuses rather than what the swarm has not reached yet.
                for def in &data::game_data().buildings {
                    if let Some(building_type) = engine::BuildingType::from_id(&def.id) {
                        planet.unlock_building(building_type);
                    }
                }
                if let Some(core) = planet.grid.find_core() {
                    planet.grid.reveal_around(core, 12);
                    // Venus is the world of gaps, so bridge one: it is the
                    // only piece of network that can stand on void.
                    let void: Vec<engine::GridPos> = planet
                        .grid
                        .iter_tiles()
                        .filter(|(_, tile)| tile.terrain == engine::TerrainType::Void)
                        .map(|(pos, _)| pos)
                        .filter(|pos| (pos.x - core.x).abs() + (pos.y - core.y).abs() < 6)
                        .take(3)
                        .collect();
                    for pos in void {
                        planet
                            .grid
                            .place_building(pos, engine::BuildingType::Bridge);
                    }
                    planet.grid.update_power_grid();
                }
            }
            "upkeep" => {
                self.phase = GamePhase::Playing;
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                self.research_state
                    .unlocked
                    .push("ceramic_plating".to_string());
                let planet = self.campaign.current_mut();
                planet.resources.minerals = 10_000.0;
                planet.resources.energy = 10_000.0;
                planet.config.resources.max_energy = 10_000.0;
                for def in &data::game_data().buildings {
                    if let Some(building_type) = engine::BuildingType::from_id(&def.id) {
                        planet.unlock_building(building_type);
                    }
                }
                let Some(core) = planet.grid.find_core() else {
                    return;
                };
                planet.grid.reveal_around(core, 14);

                // A run heading east, with a shield covering only its first half.
                for step in 1..=10 {
                    let pos = engine::GridPos::new(core.x + step, core.y);
                    if let Some(tile) = planet.grid.get_mut(pos) {
                        tile.terrain = engine::TerrainType::Empty;
                        tile.building = None;
                    }
                    planet.select_building(engine::BuildingType::Conduit);
                    planet.try_place_building(pos);
                }
                let shield = engine::GridPos::new(core.x + 2, core.y + 1);
                if let Some(tile) = planet.grid.get_mut(shield) {
                    tile.terrain = engine::TerrainType::Empty;
                    tile.building = None;
                }
                planet.select_building(engine::BuildingType::ShieldGenerator);
                planet.try_place_building(shield);
                planet.grid.update_power_grid();

                // Long enough for the acid to bite where it is not held off.
                for _ in 0..90 {
                    planet.step(1.0, false);
                }
                // Leave the shield selected so its coverage is on screen.
                planet.select_building(engine::BuildingType::ShieldGenerator);
                planet.selected_tile = Some(shield);
            }
            "smelting" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.unlock_building(engine::BuildingType::Smelter);
                let Some(core) = planet.grid.find_core() else {
                    return;
                };
                // A smelter on the run, so the drill's ore is refined on the
                // way in rather than reaching the pool.
                let smelter = engine::GridPos::new(core.x + 1, core.y - 1);
                if let Some(tile) = planet.grid.get_mut(smelter) {
                    tile.terrain = engine::TerrainType::Empty;
                    tile.building = None;
                }
                planet.select_building(engine::BuildingType::Smelter);
                planet.try_place_building(smelter);

                // A smelter costs more power than the Core makes on its own,
                // so the base needs generation before it can refine anything.
                planet.unlock_building(engine::BuildingType::WindTurbine);
                for offset in [(-1, 0), (-1, -1)] {
                    let pos = engine::GridPos::new(core.x + offset.0, core.y + offset.1);
                    if let Some(tile) = planet.grid.get_mut(pos) {
                        tile.terrain = engine::TerrainType::Empty;
                        tile.building = None;
                    }
                    planet.select_building(engine::BuildingType::WindTurbine);
                    planet.try_place_building(pos);
                }
                planet.grid.update_power_grid();
                for _ in 0..600 {
                    planet.step(0.1, false);
                }
                planet.selected_tile = Some(smelter);
            }
            "paused" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                for _ in 0..200 {
                    planet.step(state::TICK_SECONDS, false);
                }
                planet.change_speed(true);
                planet.toggle_pause();
            }
            "saved" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                // Long enough to have earned an autosave, then take it.
                for _ in 0..70 {
                    self.campaign.current_mut().step(1.0, false);
                    self.campaign.update_directive(1.0);
                }
                self.campaign.mark_saved();
            }
            "demolish" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.toggle_demolish_mode();
            }
            "toasts" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                // The drill in the seeded scene earns its own achievement
                // toast; these are the ones a longer session would have shown.
                planet
                    .notifications
                    .success("Research complete: Wind Power");
                planet.notifications.info("Available: Wind Turbine");
                planet
                    .notifications
                    .warning("Seed Ship: Ion Spine under way");
            }
            "ending" => {
                self.phase = GamePhase::CampaignComplete;
                // Play the campaign out the way it is actually played: every
                // world reached by building a ship and riding it there, so the
                // numbers on the ending screen are real.
                // The later stages are gated on research, so the scene has to
                // have done it, the same as a player would.
                for stage in &data::game_data().seed_ship.stages {
                    if let Some(tech) = stage.requires.as_deref() {
                        self.research_state.unlocked.push(tech.to_string());
                    }
                }
                self.sync_research_to_planet();

                let build_ship = |campaign: &mut Campaign| {
                    let planet = campaign.current_mut();
                    planet.config.resources.base_mineral_cap = 1_000_000.0;
                    planet.resources.minerals = 100_000.0;
                    planet.resources.data = 100_000.0;
                    planet.resources.biomass = 100_000.0;
                    planet.resources.alloy = 100_000.0;
                    if !planet.seed_ship.committed {
                        planet.toggle_seed_ship_commitment();
                    }
                    for _ in 0..2_000 {
                        planet.update_seed_ship(1.0);
                        planet.step(1.0, false);
                    }
                };

                for _ in 0..state::PLANET_COUNT {
                    build_ship(&mut self.campaign);
                    let target =
                        (0..state::PLANET_COUNT).find(|index| !self.campaign.is_colonized(*index));
                    match target {
                        Some(index) => {
                            self.campaign.launch_seed_ship(index);
                        }
                        // Nowhere left: the last ship is the ending.
                        None => break,
                    }
                }
            }
            // The banner reports a shutdown that scales with the base, so it
            // has to be looked at on a base that has some size to it.
            "collapse" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                let planet = self.campaign.current_mut();
                planet.resources.data = 400.0;
                planet.trigger_power_collapse();
            }
            "congestion" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                // A deliberately undersized run, so the saturation readout and
                // the tile outlines are visible in a still frame.
                let planet = self.campaign.current_mut();
                for _ in 0..120 {
                    planet.step(state::TICK_SECONDS, false);
                }
                // Pile a shift of drones onto one run so the tile is over its
                // limit, and crawl them so the still frame catches the jam.
                let (Some(core), Some(drill)) = (
                    planet.grid.find_core(),
                    planet
                        .grid
                        .find_buildings(engine::BuildingType::Drill)
                        .first()
                        .copied(),
                ) else {
                    return;
                };
                if let Some(route) = engine::route_over_network(&planet.grid, drill, core) {
                    for _ in 0..3 {
                        let id = planet.drones.spawn_drone(drill);
                        if let Some(drone) = planet.drones.get_drone_mut(id) {
                            drone.dispatch(
                                core,
                                route.clone(),
                                5.0,
                                engine::ResourceType::Minerals,
                            );
                        }
                    }
                }
                planet.drones.drone_speed = 0.05;
            }
            "camera" => {
                self.phase = GamePhase::Playing;
                self.seed_logistics_scene();
                // Framed as if the player had zoomed in and dragged the map.
                let camera = &mut self.campaign.current_mut().camera;
                camera.zoom = 1.8;
                camera.pan_x = -420.0;
                camera.pan_y = -180.0;
            }
            "interplanetary" => {
                self.phase = GamePhase::Interplanetary;
                // Every tech the ship's later stages are gated on, or the yard
                // stalls on the Spine and the map's "Seed Ship: READY" is a
                // promise the scene cannot keep.
                for tech in ["efficient_drills", "advanced_research", "mass_driver"] {
                    self.research_state.unlocked.push(tech.to_string());
                }
                self.sync_research_to_planet();
                self.campaign.colonize(4);
                // Something producing on the world left behind, so the map has
                // a stockpile to report.
                if let Some(core) = self
                    .campaign
                    .stockpile(4)
                    .and(self.campaign.current().grid.find_core())
                {
                    self.campaign.travel_to(4);
                    let away = self.campaign.current_mut();
                    away.config.resources.base_mineral_cap = 100_000.0;
                    let drill = engine::GridPos::new(core.x + 1, core.y);
                    if let Some(tile) = away.grid.get_mut(drill) {
                        tile.terrain = engine::TerrainType::Empty;
                    }
                    away.grid.reveal_around(drill, 1);
                    away.select_building(engine::BuildingType::Drill);
                    away.try_place_building(drill);
                    away.grid.update_power_grid();
                    self.campaign.travel_to(2);
                    for _ in 0..400 {
                        self.campaign.update_background(1.0);
                    }
                }
                // A ship on the pad, so the map shows a launch is possible.
                let planet = self.campaign.current_mut();
                planet.config.resources.base_mineral_cap = 1_000_000.0;
                planet.resources.minerals = 100_000.0;
                planet.resources.data = 100_000.0;
                planet.resources.biomass = 100_000.0;
                planet.resources.alloy = 100_000.0;
                planet.toggle_seed_ship_commitment();
                for _ in 0..2_000 {
                    planet.update_seed_ship(1.0);
                }
                self.seed_shipping_scene(4);
            }
            _ => {
                // Default: jump straight into gameplay on the starting planet.
                self.phase = GamePhase::Playing;
            }
        }
    }

    /// A Mass Driver on the current world under a standing order, with a
    /// half-loaded pod and two more already crossing the system, so the map's
    /// shipping panel has something real to show.
    fn seed_shipping_scene(&mut self, target: usize) {
        use engine::{BuildingType, GridPos, ResourceType};

        let planet = self.campaign.current_mut();
        let Some(core) = planet.grid.find_core() else {
            return;
        };
        // The yard drank the world's energy building the ship, and a Mass
        // Driver is not cheap to put up.
        planet.config.resources.max_energy = 1_000.0;
        planet.resources.energy = 1_000.0;

        // A driver draws more than the Core generates, so the scene pays for
        // it rather than quietly browning out the yard it is standing next to.
        planet.unlock_building(BuildingType::WindTurbine);
        planet.select_building(BuildingType::WindTurbine);
        for offset in [-1, 1] {
            let pos = GridPos::new(core.x, core.y + offset);
            if let Some(tile) = planet.grid.get_mut(pos) {
                tile.terrain = engine::TerrainType::Empty;
            }
            planet.grid.reveal_around(pos, 1);
            planet.try_place_building(pos);
        }

        let driver = GridPos::new(core.x - 1, core.y);
        if let Some(tile) = planet.grid.get_mut(driver) {
            tile.terrain = engine::TerrainType::Empty;
        }
        planet.grid.reveal_around(driver, 1);
        planet.unlock_building(BuildingType::MassDriver);
        planet.select_building(BuildingType::MassDriver);
        planet.try_place_building(driver);
        planet.grid.update_power_grid();

        planet.export = Some(state::ExportOrder {
            target,
            cargo: ResourceType::Minerals,
        });
        let capacity = planet.config.mass_driver.pod_capacity;
        planet
            .pod_loads
            .insert((driver.x, driver.y), capacity * 0.45);
        let transit = state::transit_seconds(
            planet.planet_index,
            target,
            &planet.config.mass_driver.clone(),
        );
        // Two pods, thrown a while apart, so the map shows a stream rather
        // than one dot with another hiding behind it.
        for spent in [0.55, 0.2] {
            planet.launched_pods.push(state::Shipment {
                from: planet.planet_index,
                to: target,
                cargo: ResourceType::Minerals,
                amount: capacity,
                remaining: transit * (1.0 - spent),
                transit,
            });
        }
        // Something on the far end to catch them, or the panel is a warning
        // rather than a working route.
        if let Some(receiver) = self.campaign.planet_mut(target) {
            if let Some(core) = receiver.grid.find_core() {
                let pad = GridPos::new(core.x, core.y + 1);
                if let Some(tile) = receiver.grid.get_mut(pad) {
                    tile.terrain = engine::TerrainType::Empty;
                }
                receiver.resources.energy = 500.0;
                receiver.config.resources.max_energy = 500.0;
                receiver.grid.reveal_around(pad, 1);
                receiver.unlock_building(BuildingType::LandingPad);
                receiver.select_building(BuildingType::LandingPad);
                receiver.try_place_building(pad);
                receiver.grid.update_power_grid();
            }
        }

        self.campaign.update_shipments(0.0);
    }

    /// A working conduit run with a drill on the end of it, so drone routing
    /// can be eyeballed without playing up to it.
    fn seed_logistics_scene(&mut self) {
        use engine::{BuildingType, GridPos};

        let state = self.campaign.current_mut();
        let Some(core) = state.grid.find_core() else {
            return;
        };
        state.grid.reveal_around(core, 12);
        state.resources.minerals = 500.0;
        state.resources.energy = 500.0;
        state.config.resources.max_energy = 500.0;
        state.unlock_building(BuildingType::Conduit);
        state.unlock_building(BuildingType::PowerNode);

        // An L-shaped run: five tiles east, then four north, drill on the end.
        let mut run: Vec<GridPos> = (1..=5).map(|x| GridPos::new(core.x + x, core.y)).collect();
        run.extend((1..=4).map(|y| GridPos::new(core.x + 5, core.y - y)));

        for (index, pos) in run.iter().enumerate() {
            if let Some(tile) = state.grid.get_mut(*pos) {
                tile.terrain = engine::TerrainType::Empty;
                tile.building = None;
            }
            let piece = if index == 4 {
                BuildingType::PowerNode
            } else {
                BuildingType::Conduit
            };
            state.select_building(piece);
            state.try_place_building(*pos);
        }

        let drill = GridPos::new(core.x + 5, core.y - 5);
        if let Some(tile) = state.grid.get_mut(drill) {
            tile.terrain = engine::TerrainType::Empty;
            tile.building = None;
        }
        state.select_building(BuildingType::Drill);
        state.try_place_building(drill);
        state.grid.update_power_grid();
    }
}
