use macroquad::prelude::{Color, Vec2};
use macroquad_toolkit::fx::{BurstConfig, Particle};

use crate::engine::{BuildingType, DroneState, GridPos};

use super::game_state::PlanetState;

impl PlanetState {
    pub(super) fn spawn_particle(
        &mut self,
        position: (f32, f32),
        velocity: (f32, f32),
        life: f32,
        color: Color,
        size: f32,
    ) {
        self.particles.spawn(Particle::new(
            Vec2::new(position.0, position.1),
            Vec2::new(velocity.0, velocity.1),
            life,
            size,
            color,
        ));
    }

    pub(super) fn spawn_resource_burst(&mut self) {
        if macroquad_toolkit::settings::reduced_motion_enabled() {
            return;
        }
        let core_pos = match self.grid.find_core() {
            Some(pos) => pos,
            None => return,
        };
        let origin = Vec2::new(core_pos.x as f32, core_pos.y as f32);
        let config = BurstConfig {
            speed: (0.6, 1.2),
            size: (3.0, 3.0),
            life: (0.35, 0.6),
            colors: vec![Color::new(1.0, 0.42, 0.21, 1.0)],
            drag: 1.0,
            shrink: false,
            ..Default::default()
        };
        self.particles.spawn_burst(origin, 8, &config);
    }

    pub(super) fn spawn_place_burst(&mut self, pos: GridPos) {
        if macroquad_toolkit::settings::reduced_motion_enabled() {
            return;
        }
        let origin = Vec2::new(pos.x as f32, pos.y as f32);
        let config = BurstConfig {
            speed: (0.6, 1.4),
            size: (2.5, 2.5),
            life: (0.25, 0.5),
            colors: vec![Color::new(0.0, 0.85, 1.0, 1.0)],
            drag: 1.0,
            shrink: false,
            ..Default::default()
        };
        self.particles.spawn_burst(origin, 10, &config);
    }

    pub(super) fn spawn_harvest_burst(&mut self, pos: GridPos) {
        if macroquad_toolkit::settings::reduced_motion_enabled() {
            return;
        }
        let origin = Vec2::new(pos.x as f32, pos.y as f32);
        let config = BurstConfig {
            speed: (0.8, 1.8),
            size: (2.0, 3.5),
            life: (0.3, 0.7),
            colors: vec![Color::new(1.0, 0.55, 0.12, 1.0)],
            drag: 1.2,
            shrink: true,
            ..Default::default()
        };
        self.particles.spawn_burst(origin, 12, &config);
    }

    /// A drone waving its error flag: the route under it just went away.
    pub(super) fn spawn_route_break_burst(&mut self, pos: GridPos) {
        if macroquad_toolkit::settings::reduced_motion_enabled() {
            return;
        }
        let origin = Vec2::new(pos.x as f32, pos.y as f32);
        let config = BurstConfig {
            speed: (0.4, 1.0),
            size: (2.0, 2.0),
            life: (0.4, 0.8),
            colors: vec![Color::new(1.0, 0.3, 0.3, 1.0)],
            drag: 1.4,
            shrink: true,
            ..Default::default()
        };
        self.particles.spawn_burst(origin, 8, &config);
    }

    pub(super) fn spawn_drone_trails(&mut self, delta_time: f32) {
        self.particle_timer += delta_time;
        if self.particle_timer < 0.08 {
            return;
        }
        self.particle_timer = 0.0;

        let drone_positions: Vec<(f32, f32)> = self
            .drones
            .drones()
            .iter()
            .filter(|drone| {
                drone.state == DroneState::MovingToCore || drone.state == DroneState::MovingToDrill
            })
            .map(|drone| drone.visual_position())
            .collect();

        for (x, y) in drone_positions {
            let jitter = (
                macroquad_toolkit::rng::gen_range(-0.2, 0.2),
                macroquad_toolkit::rng::gen_range(-0.2, 0.2),
            );
            let velocity = (
                macroquad_toolkit::rng::gen_range(-0.4, 0.4),
                macroquad_toolkit::rng::gen_range(-0.4, 0.4),
            );
            let life = macroquad_toolkit::rng::gen_range(0.25, 0.5);
            let color = Color::new(0.0, 0.85, 1.0, 1.0);
            self.spawn_particle((x + jitter.0, y + jitter.1), velocity, life, color, 2.0);
        }
    }

    pub(super) fn update_particles(&mut self, delta_time: f32) {
        self.particles.update(delta_time);
    }
}
