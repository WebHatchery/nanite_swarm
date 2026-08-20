//! Power-collapse consequences and the failure record shown to the player.

use crate::engine::{DroneState, StatId};
use crate::state::game_state::PlanetState;
use macroquad_toolkit::math::lerp;

impl PlanetState {
    /// How far along the swarm is, for anything that should cost more the
    /// more there is of it. Zero for a base of nothing, one at full scale.
    pub fn collapse_scale(&self) -> f32 {
        let full = self.config.collapse.full_scale_structures.max(1.0);
        (self.grid.total_buildings() as f32 / full).clamp(0.0, 1.0)
    }

    /// Bring the grid down. The simulation reaches this through sustained
    /// negative power, while the capture harness can stage it directly.
    pub fn trigger_power_collapse(&mut self) {
        self.record_collapse_source();
        if let Some(source) = self.latest_collapse_source().map(str::to_owned) {
            self.notifications
                .danger(format!("{} - collapse engaged", source));
        }
        self.emit_audio(crate::state::audio::AudioEvent::Collapse);
        let collapse = self.config.collapse.clone();
        let scale = self.collapse_scale();
        let shutdown = self
            .stats
            .apply(
                StatId::CollapseShutdown,
                lerp(
                    collapse.min_shutdown_seconds,
                    collapse.max_shutdown_seconds,
                    scale,
                ),
            )
            .clamp(collapse.min_shutdown_seconds, collapse.max_shutdown_seconds);
        let loss = self
            .stats
            .apply(
                StatId::CollapseDataLoss,
                lerp(collapse.min_data_loss, collapse.max_data_loss, scale),
            )
            .clamp(0.0, 1.0);

        self.power_negative_seconds = 0.0;
        self.power_collapse_cooldown = collapse.cooldown_seconds;
        self.power_collapse_shutdown = shutdown;
        self.power_collapse_length = shutdown;
        self.research_lock_timer = shutdown * collapse.research_lock_ratio.max(0.0);
        self.collapse_notice_timer = collapse.notice_seconds;
        self.network_revision = self.network_revision.wrapping_add(1);

        for drone in self.drones.drones_mut() {
            drone.carrying = 0.0;
            drone.state = DroneState::Error;
            drone.path.clear();
            drone.path_index = 0;
            drone.progress = 0.0;
            drone.target = drone.position;
        }

        self.resources.data *= 1.0 - loss;
        self.research.research_progress *= 1.0 - loss;
    }

    fn record_collapse_source(&mut self) {
        let local = self.grid.iter_tiles().find_map(|(pos, tile)| {
            let building = tile.building.as_ref()?;
            (building.powered && (building.is_dust_stalled() || building.acid_wear >= 100.0))
                .then_some((pos, building.building_type))
        });
        let (source, building, position) = if let Some((position, building)) = local {
            (
                format!("Local failure: {}", building.name()),
                Some(building),
                Some(position),
            )
        } else {
            ("Broad grid collapse: power deficit".to_string(), None, None)
        };
        self.collapse_history.push(crate::state::CollapseRecord {
            source,
            building,
            position,
            world_time: self.time_played,
        });
        if self.collapse_history.len() > 32 {
            let keep_from = self.collapse_history.len() - 32;
            self.collapse_history.drain(..keep_from);
        }
    }

    pub fn latest_collapse_source(&self) -> Option<&str> {
        self.collapse_history
            .last()
            .map(|record| record.source.as_str())
    }
}
