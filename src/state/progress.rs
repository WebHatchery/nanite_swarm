use crate::data::DustResponseConfig;
use crate::engine::{BuildingType, GridPos, StatId};

use super::game_state::PlanetState;

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct OfflineReport {
    pub elapsed_seconds: f32,
    pub capped_seconds: f32,
    pub tamper_guarded: bool,
    pub minerals_gained: f32,
    pub alloy_gained: f32,
    pub data_gained: f32,
    pub power_gained: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NextEvent {
    pub label: &'static str,
    pub seconds: f32,
}

pub(crate) fn hazard_field_strength(
    pos: GridPos,
    hazard: &str,
    broad: f32,
    fields: &[crate::data::HazardFieldDef],
    width: u32,
    height: u32,
) -> f32 {
    let width = width.max(1) as f32;
    let height = height.max(1) as f32;
    let x = pos.x as f32 / width;
    let y = pos.y as f32 / height;
    let field = fields
        .iter()
        .filter(|field| field.hazard == hazard && field.radius > 0.0)
        .map(|field| {
            let dx = (x - field.center[0]) * width;
            let dy = (y - field.center[1]) * height;
            let distance = (dx * dx + dy * dy).sqrt();
            (field.strength * (1.0 - distance / (field.radius * width.min(height)))).max(0.0)
        })
        .fold(0.0, f32::max);
    broad.max(field)
}

impl PlanetState {
    /// Research can move the response points without changing the data file
    /// that describes ordinary upkeep. Keeping this resolver on the state
    /// makes every simulation consumer read the same values.
    pub fn resolved_dust_response(&self) -> DustResponseConfig {
        let base = &self.config.upkeep.dust_response;
        DustResponseConfig {
            efficiency_threshold: self
                .stats
                .apply(StatId::DustEfficiencyThreshold, base.efficiency_threshold)
                .max(0.0),
            efficiency: base.efficiency,
            speed_threshold: self
                .stats
                .apply(StatId::DustSpeedThreshold, base.speed_threshold)
                .max(0.0),
            speed_multiplier: base.speed_multiplier,
            leak_threshold: self
                .stats
                .apply(StatId::DustLeakThreshold, base.leak_threshold)
                .max(0.0),
            leak: base.leak,
            stall_threshold: self
                .stats
                .apply(StatId::DustStallThreshold, base.stall_threshold)
                .max(0.0),
        }
    }
    /// Select a building type for placement.
    pub fn select_building(&mut self, building_type: BuildingType) {
        if self.is_building_unlocked(building_type) {
            self.selected_building = Some(building_type);
            // Building and demolishing are the same click, so they cannot both
            // be armed.
            self.demolish_mode = false;
        }
    }

    /// Arm or disarm demolition. Picking it up puts the build cursor down.
    pub fn toggle_demolish_mode(&mut self) {
        self.demolish_mode = !self.demolish_mode;
        if self.demolish_mode {
            self.selected_building = None;
        }
    }

    /// Clear building selection.
    pub fn clear_selection(&mut self) {
        self.selected_building = None;
    }

    pub fn is_building_unlocked(&self, building_type: BuildingType) -> bool {
        self.is_building_researched(building_type) && !self.is_building_banned(building_type)
    }

    /// Research has opened this building up, whether or not this particular
    /// world will accept it.
    pub fn is_building_researched(&self, building_type: BuildingType) -> bool {
        matches!(building_type, BuildingType::Core)
            || self.unlocked_buildings.contains(&building_type)
    }

    pub fn active_planet_condition(&self) -> Option<&'static str> {
        if self.hazards.acid_rain > 0.0
            || crate::data::game_data()
                .planet(self.planet_index)
                .hazard_fields
                .iter()
                .any(|field| field.hazard == "acid")
        {
            Some("acid")
        } else if self.hazards.freeze > 0.0
            || crate::data::game_data()
                .planet(self.planet_index)
                .hazard_fields
                .iter()
                .any(|field| field.hazard == "cold")
        {
            Some("cold")
        } else {
            None
        }
    }

    /// Strength of a named spatial field at a tile, including the world's
    /// broad hazard value. Radial falloff keeps field edges readable and lets
    /// one definition work on every supported map size.
    pub fn hazard_strength_at(&self, pos: GridPos, hazard: &str) -> f32 {
        let broad = match hazard {
            "acid" => self.acid_strength(),
            "cold" => self.freeze_strength(),
            _ => 0.0,
        };
        let def = crate::data::game_data().planet(self.planet_index);
        hazard_field_strength(
            pos,
            hazard,
            broad,
            &def.hazard_fields,
            self.grid.width,
            self.grid.height,
        )
    }

    pub fn planet_constraint_status(&self) -> (bool, String) {
        let constraints = &crate::data::game_data()
            .planet(self.planet_index)
            .constraints;
        let missing: Vec<_> = constraints
            .required_buildings
            .iter()
            .filter(|id| {
                BuildingType::from_id(id)
                    .is_none_or(|kind| self.grid.find_buildings(kind).is_empty())
            })
            .cloned()
            .collect();
        let generation_ok = self.power_generation() >= constraints.minimum_power_generation;
        let balance_ok = self.power_balance >= constraints.minimum_power_balance;
        let mut reasons = Vec::new();
        if !missing.is_empty() {
            reasons.push(format!("needs {}", missing.join(", ")));
        }
        if !generation_ok {
            reasons.push(format!(
                "needs {:.0} power generation",
                constraints.minimum_power_generation
            ));
        }
        if !balance_ok {
            reasons.push(format!(
                "needs {:.0} power surplus",
                constraints.minimum_power_balance
            ));
        }
        (reasons.is_empty(), reasons.join("; "))
    }

    /// Positive world rules gate the infrastructure that makes a campaign-wide
    /// route meaningful. Ordinary infrastructure must remain placeable while
    /// the player is satisfying that rule; only the gated Mass Driver checks
    /// the world's prerequisite buildings and power floor.
    pub fn constraints_allow_building(&self, building_type: BuildingType) -> bool {
        let constraints = &crate::data::game_data()
            .planet(self.planet_index)
            .constraints;
        if !constraints.required_research.iter().all(|required| {
            self.research
                .unlocked_techs
                .iter()
                .any(|known| known == required)
        }) {
            return false;
        }
        if building_type == BuildingType::MassDriver {
            let required_buildings_met = constraints.required_buildings.iter().all(|required| {
                BuildingType::from_id(required).is_none_or(|kind| {
                    kind == building_type || !self.grid.find_buildings(kind).is_empty()
                })
            });
            let power_floor_met = self.power_generation() >= constraints.minimum_power_generation;
            if !required_buildings_met || !power_floor_met {
                return false;
            }
        }
        true
    }

    /// This world refuses the building outright, however much research the
    /// swarm has done.
    pub fn is_building_banned(&self, building_type: BuildingType) -> bool {
        self.banned_buildings.contains(&building_type)
    }

    pub fn unlock_building(&mut self, building_type: BuildingType) {
        if !self.unlocked_buildings.contains(&building_type) {
            self.unlocked_buildings.push(building_type);
        }
    }

    pub fn mineral_capacity(&self) -> f32 {
        self.stats
            .apply(StatId::MineralCapacity, self.built_mineral_capacity())
    }

    /// Storage before research touches it: the base plus whatever has been
    /// built for it. The stat sheet needs the two halves separately.
    pub fn built_mineral_capacity(&self) -> f32 {
        let storage_count = self.grid.find_buildings(BuildingType::Storage).len() as f32;
        self.config.resources.base_mineral_cap + storage_count * self.config.resources.storage_bonus
    }

    /// Power produced this tick, including whatever the biomass harvesters
    /// burned. Generation and consumption are asked for separately because the
    /// HUD shows both halves.
    pub fn power_generation(&self) -> f32 {
        self.grid.total_power_generation() + self.biomass_power_bonus
    }

    /// Power drawn this tick, after research efficiencies.
    pub fn power_consumption(&self) -> f32 {
        self.stats.apply(
            StatId::PowerConsumption,
            self.grid.total_power_consumption(),
        )
    }

    pub fn net_power(&self) -> f32 {
        self.power_generation() - self.power_consumption()
    }

    /// Acid still reaching the network after research counters it.
    pub fn acid_strength(&self) -> f32 {
        (self.hazards.acid_rain * self.stats.multiplier(StatId::AcidResistance)).max(0.0)
    }

    /// Share of drone speed the cold still takes, after research. Capped so a
    /// world can slow the swarm to a crawl but never stop it dead.
    pub fn freeze_strength(&self) -> f32 {
        (self.hazards.freeze * self.stats.multiplier(StatId::FreezeResistance)).clamp(0.0, 0.9)
    }

    /// Alloy the working smelters would produce per second, for the readout.
    pub fn alloy_rate(&self) -> f32 {
        crate::data::game_data()
            .buildings
            .iter()
            .filter_map(|def| {
                let out = def.recipe.outputs.get("alloy").copied().unwrap_or(0.0);
                (out > 0.0).then_some((def, out))
            })
            .filter_map(|(def, out)| BuildingType::from_id(&def.id).map(|kind| (kind, out)))
            .map(|(kind, out)| {
                self.grid
                    .find_buildings(kind)
                    .into_iter()
                    .filter_map(|pos| self.grid.get(pos).and_then(|tile| tile.building.as_ref()))
                    .filter(|building| building.powered && !building.is_dust_stalled())
                    .map(|building| out * building.dust_efficiency())
                    .sum::<f32>()
            })
            .sum()
    }

    /// Stop and start the world.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
        // Drop the part-finished step: resuming should not owe time.
        self.sim_accumulator = 0.0;
    }

    /// Step to the next speed up or down the ladder, clamped at both ends.
    pub fn change_speed(&mut self, faster: bool) {
        let scales = crate::state::TIME_SCALES;
        let current = scales
            .iter()
            .position(|scale| (*scale - self.time_scale).abs() < 1e-3)
            .unwrap_or(1);
        let next = if faster {
            (current + 1).min(scales.len() - 1)
        } else {
            current.saturating_sub(1)
        };
        self.time_scale = scales[next];
    }

    pub fn next_interesting_event(&self) -> Option<NextEvent> {
        [
            (self.power_collapse_shutdown, "Power recovery"),
            (self.power_collapse_cooldown, "Collapse recovery"),
            (self.research_lock_timer, "Research unlock"),
            (self.export_cooldown, "Mass Driver ready"),
        ]
        .into_iter()
        .filter(|(seconds, _)| *seconds > 0.01 && seconds.is_finite())
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map(|(seconds, label)| NextEvent { label, seconds })
    }

    pub fn fast_forward_to_next_event(&mut self) -> bool {
        let Some(event) = self.next_interesting_event() else {
            return false;
        };
        self.step(event.seconds, false);
        self.sim_accumulator = 0.0;
        true
    }

    /// A short label for whatever this world is doing to the machinery, for
    /// the HUD. Empty when the world is merely somewhere to build.
    pub fn hazard_label(&self) -> String {
        let mut parts = Vec::new();
        if self.acid_strength() > 0.0 {
            parts.push("ACID RAIN");
        }
        if self.freeze_strength() > 0.0 {
            parts.push("DEEP FREEZE");
        }
        parts.join(" / ")
    }

    /// What this world has to say for itself, shown on arrival.
    pub fn arrival_line(&self) -> &'static str {
        crate::data::game_data()
            .planet(self.planet_index)
            .arrival
            .as_str()
    }

    pub fn battery_time_left(&self) -> (i32, i32) {
        let total = self.battery_seconds.max(0.0) as i32;
        let hours = total / 3600;
        let minutes = (total % 3600) / 60;
        (hours, minutes)
    }

    pub fn apply_offline_progress(&mut self, offline_seconds: f32) {
        if offline_seconds <= 0.0 {
            self.last_offline_seconds = 0.0;
            self.last_offline_simulated = 0.0;
            return;
        }

        let capped = offline_seconds.min(self.config.offline.max_elapsed_seconds.max(0.0));
        let active_seconds = capped.min(self.battery_seconds.max(0.0));
        let hibernation_seconds = (capped - active_seconds).max(0.0) * 0.1;
        let simulated = active_seconds + hibernation_seconds;
        let drill_count = self.powered_positions(BuildingType::Drill).len() as f32;
        let mineral_rate = drill_count * self.drill_output_rate();
        let alloy_rate = self.alloy_rate();
        let data_rate = self.stats.apply(
            StatId::DataGeneration,
            self.config.resources.core_data_rate
                + self.config.resources.server_data_rate
                    * self.grid.find_buildings(BuildingType::ServerBank).len() as f32,
        );
        let power_rate = self.net_power();
        let cap = self.config.offline.max_resource_gain.max(0.0);
        let minerals = (mineral_rate * simulated).clamp(-cap, cap);
        let alloy = (alloy_rate * simulated).clamp(0.0, cap);
        let data = (data_rate * simulated).clamp(0.0, cap);
        let power = (power_rate * simulated).clamp(-cap, cap);
        self.resources.minerals = (self.resources.minerals + minerals).min(self.mineral_capacity());
        self.resources.alloy = (self.resources.alloy + alloy).min(cap.max(1000.0));
        self.resources.data = (self.resources.data + data).min(1000.0);
        self.resources.energy =
            (self.resources.energy + power).clamp(0.0, self.config.resources.max_energy);
        self.time_played += simulated as f64;
        self.battery_seconds = (self.battery_seconds - capped).max(0.0);
        self.last_offline_report = OfflineReport {
            elapsed_seconds: offline_seconds,
            capped_seconds: capped,
            tamper_guarded: false,
            minerals_gained: minerals,
            alloy_gained: alloy,
            data_gained: data,
            power_gained: power,
        };
        self.last_offline_seconds = offline_seconds;
        self.last_offline_simulated = simulated;
        self.offline_notice_timer = 8.0;
    }

    pub fn achievements_progress(&self) -> (usize, usize) {
        self.achievements.progress()
    }

    pub fn placement_scale(&self, pos: GridPos) -> f32 {
        if macroquad_toolkit::settings::reduced_motion_enabled() {
            return 1.0;
        }
        let Some(anim) = self
            .placement_anims
            .iter()
            .find(|anim| anim.position == pos)
        else {
            return 1.0;
        };
        let progress = (anim.timer / 0.3).clamp(0.0, 1.0);
        let bounce_phase = (1.0 - progress) * std::f32::consts::PI * 2.0;
        1.0 + (bounce_phase.sin() * 0.12)
    }

    /// Unlock an achievement, and say so if it had not already fired.
    ///
    /// `unlock` returns whether this was the moment it happened, which is the
    /// only reason the toast is not repeated every tick.
    pub fn announce_achievement(&mut self, id: &str) {
        if !self.achievements.unlock(id) {
            return;
        }
        let name = self
            .achievements
            .get(id)
            .map(|achievement| achievement.name.clone())
            .unwrap_or_else(|| id.to_string());
        self.emit_audio(super::audio::AudioEvent::Achievement);
        self.notifications.success(format!("Achievement: {}", name));
    }
}

#[cfg(test)]
mod tests;
