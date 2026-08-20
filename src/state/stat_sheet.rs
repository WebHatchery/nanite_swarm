//! What the swarm is actually worth right now.
//!
//! Research could say what each tech *would* add, and the simulation knew what
//! everything came to, but nothing put the two together: a player could read
//! "+50% drill output" on three separate nodes and still have no idea what a
//! drill produces. This resolves every stat against the base it started from,
//! on this world, with everything currently standing counted in.

use crate::engine::StatId;

use super::game_state::PlanetState;

/// One line of the sheet: where the stat started and where it is now.
#[derive(Debug, Clone, PartialEq)]
pub struct StatReading {
    pub stat: StatId,
    /// The value with nothing unlocked and nothing built for it.
    pub base: f32,
    /// The value the simulation is using.
    pub value: f32,
    pub sources: Vec<String>,
}

impl StatReading {
    /// Whether anything has moved this stat off its base at all.
    pub fn is_changed(&self) -> bool {
        (self.value - self.base).abs() > 0.0001
    }

    /// Whether where it has moved to is good news, using the stat's own idea
    /// of which direction that is.
    pub fn is_gain(&self) -> bool {
        (self.value < self.base) == self.stat.lower_is_better()
    }
}

/// How a value should read to a player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatUnit {
    /// A rate: shown per second.
    PerSecond,
    /// A whole number of things.
    Count,
    /// A fraction of something, shown as a percentage.
    Share,
    /// A plain quantity.
    Flat,
}

impl StatUnit {
    /// Declared alongside the stat's label in `research.json`.
    pub fn of(stat: StatId) -> Self {
        let declared = crate::data::game_data()
            .research
            .stats
            .iter()
            .find(|entry| entry.id == stat.id())
            .map(|entry| entry.unit.as_str())
            .unwrap_or_default();
        match declared {
            "per_second" => StatUnit::PerSecond,
            "count" => StatUnit::Count,
            "share" => StatUnit::Share,
            _ => StatUnit::Flat,
        }
    }

    pub fn format(self, value: f32) -> String {
        match self {
            StatUnit::PerSecond => format!("{:.2}/s", value),
            StatUnit::Count => format!("{:.0}", value),
            StatUnit::Share => format!("{:.0}%", value * 100.0),
            StatUnit::Flat => format!("{:.0}", value),
        }
    }
}

impl PlanetState {
    /// What one drill cuts per second, after research. The HUD used to show
    /// the raw config number, so the readout stopped agreeing with the
    /// simulation the moment anything was researched.
    pub fn drill_output_rate(&self) -> f32 {
        self.stats
            .apply(StatId::DrillOutput, self.config.buildings.drill_output_rate)
            * self.factory_focus_multiplier(crate::engine::BuildingType::Drill)
    }

    /// Every stat, resolved against this world. In `StatId::ALL` order, so the
    /// sheet does not reshuffle itself as things are researched.
    pub fn stat_sheet(&self) -> Vec<StatReading> {
        StatId::ALL
            .into_iter()
            .map(|stat| {
                let (base, value) = self.stat_reading(stat);
                let sources = self.stat_sources(stat);
                StatReading {
                    stat,
                    base,
                    value,
                    sources,
                }
            })
            .collect()
    }

    /// Names of research and earned stages that contribute to a resolved stat.
    /// The inspector can show provenance without duplicating modifier folding.
    pub fn stat_sources(&self, stat: StatId) -> Vec<String> {
        let mut sources = Vec::new();
        for node in &crate::data::game_data().research.nodes {
            if self.research.unlocked_techs.iter().any(|id| id == &node.id)
                && node
                    .modifiers
                    .iter()
                    .any(|modifier| modifier.stat == stat.id())
            {
                sources.push(node.name.clone());
            }
        }
        for stage in self.seed_ship.standing_stages() {
            if stage
                .modifiers
                .iter()
                .any(|modifier| modifier.stat == stat.id())
            {
                sources.push(stage.name.clone());
            }
        }
        for stage in self.core_stages_reached() {
            if stage
                .modifiers
                .iter()
                .any(|modifier| modifier.stat == stat.id())
            {
                sources.push(format!("Core: {}", stage.name));
            }
        }
        sources
    }

    /// The base and the resolved value for one stat. Several of these have a
    /// base that depends on what is standing (storage, power draw) or on the
    /// world itself (the hazards), which is exactly why the sheet is per
    /// planet rather than per campaign.
    fn stat_reading(&self, stat: StatId) -> (f32, f32) {
        let resources = &self.config.resources;
        let base = match stat {
            StatId::DrillOutput => self.config.buildings.drill_output_rate,
            StatId::DroneCapacity => resources.drone_carry_capacity,
            StatId::DronesPerDrill => resources.drones_per_drill,
            StatId::DataGeneration => resources.server_data_rate,
            StatId::DustAccumulation => self.config.upkeep.dust_rate,
            StatId::MineralCapacity => self.built_mineral_capacity(),
            StatId::ProcessorPadCapacity => self
                .config
                .logistics
                .pad_depth
                .max(self.drones.drone_capacity * 3.0),
            StatId::PowerConsumption => self.grid.total_power_consumption(),
            StatId::ResearchRate => resources.research_rate,
            StatId::AcidResistance => self.hazards.acid_rain,
            StatId::FreezeResistance => self.hazards.freeze,
            StatId::DroneSpeed => resources.drone_speed,
            StatId::RepeaterRange => self.config.buildings.repeater_range as f32,
            // A multiplier on whatever the terrain declares, so the base is
            // "what the ground is worth" and the sheet reads it as a share.
            StatId::HarvestYield => 1.0,
            StatId::CollapseShutdown => self.config.collapse.max_shutdown_seconds,
            StatId::CollapseDataLoss => self.config.collapse.max_data_loss,
            StatId::DustEfficiencyThreshold => {
                self.config.upkeep.dust_response.efficiency_threshold
            }
            StatId::DustSpeedThreshold => self.config.upkeep.dust_response.speed_threshold,
            StatId::DustLeakThreshold => self.config.upkeep.dust_response.leak_threshold,
            StatId::DustStallThreshold => self.config.upkeep.dust_response.stall_threshold,
        };
        // The hazards are not scaled from a base, they are eaten into: a
        // counter reduces how much of the world still reaches the swarm.
        let value = match stat {
            StatId::AcidResistance => self.acid_strength(),
            StatId::FreezeResistance => self.freeze_strength(),
            _ => self.stats.apply(stat, base),
        };
        (base, value)
    }
}

#[cfg(test)]
mod tests;
