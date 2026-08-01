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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatReading {
    pub stat: StatId,
    /// The value with nothing unlocked and nothing built for it.
    pub base: f32,
    /// The value the simulation is using.
    pub value: f32,
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
    }

    /// Every stat, resolved against this world. In `StatId::ALL` order, so the
    /// sheet does not reshuffle itself as things are researched.
    pub fn stat_sheet(&self) -> Vec<StatReading> {
        StatId::ALL
            .into_iter()
            .map(|stat| {
                let (base, value) = self.stat_reading(stat);
                StatReading { stat, base, value }
            })
            .collect()
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
            StatId::DustAccumulation => super::simulation::DUST_RATE,
            StatId::MineralCapacity => self.built_mineral_capacity(),
            StatId::PowerConsumption => self.grid.total_power_consumption(),
            StatId::ResearchRate => resources.research_rate,
            StatId::AcidResistance => self.hazards.acid_rain,
            StatId::FreezeResistance => self.hazards.freeze,
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
mod tests {
    use super::*;
    use crate::data::GameConfig;

    fn reading(state: &PlanetState, stat: StatId) -> StatReading {
        state
            .stat_sheet()
            .into_iter()
            .find(|reading| reading.stat == stat)
            .expect("every stat is on the sheet")
    }

    fn state() -> PlanetState {
        PlanetState::new(0, 7, GameConfig::default())
    }

    #[test]
    fn a_swarm_that_knows_nothing_sits_on_its_bases() {
        let state = state();
        for reading in state.stat_sheet() {
            assert!(
                !reading.is_changed(),
                "{} moved with nothing researched: {} from {}",
                reading.stat.id(),
                reading.value,
                reading.base
            );
        }
    }

    #[test]
    fn the_sheet_covers_every_stat_once_and_keeps_its_order() {
        let sheet = state().stat_sheet();
        assert_eq!(sheet.len(), StatId::ALL.len());
        for (index, stat) in StatId::ALL.into_iter().enumerate() {
            assert_eq!(sheet[index].stat, stat);
        }
    }

    #[test]
    fn research_moves_the_number_the_simulation_is_actually_using() {
        let mut state = state();
        state
            .research
            .unlocked_techs
            .push("efficient_drills".to_string());
        state.refresh_stats();

        let drills = reading(&state, StatId::DrillOutput);
        assert!(drills.is_changed());
        assert!(drills.is_gain());
        assert!(
            drills.value > drills.base,
            "{} is not more than {}",
            drills.value,
            drills.base
        );
    }

    #[test]
    fn less_of_a_bad_thing_still_counts_as_a_gain() {
        let mut state = state();
        state
            .research
            .unlocked_techs
            .push("self_cleaning_servos".to_string());
        state.refresh_stats();

        let dust = reading(&state, StatId::DustAccumulation);
        assert!(dust.value < dust.base, "dust did not fall");
        assert!(dust.is_gain(), "less dust read as a loss");
    }

    #[test]
    fn a_world_with_no_acid_reads_zero_rather_than_a_counter_working() {
        // Mars has no hazards, so both halves of the line are nothing and the
        // sheet must not claim the swarm is holding anything off.
        let state = state();
        let acid = reading(&state, StatId::AcidResistance);
        assert_eq!(acid.base, 0.0);
        assert_eq!(acid.value, 0.0);
        assert!(!acid.is_changed());
    }

    #[test]
    fn a_stats_units_come_from_the_same_file_that_names_it() {
        assert_eq!(StatUnit::of(StatId::DrillOutput), StatUnit::PerSecond);
        assert_eq!(StatUnit::of(StatId::DronesPerDrill), StatUnit::Count);
        assert_eq!(StatUnit::of(StatId::AcidResistance), StatUnit::Share);
        assert_eq!(StatUnit::PerSecond.format(7.5), "7.50/s");
        assert_eq!(StatUnit::Count.format(2.0), "2");
        assert_eq!(StatUnit::Share.format(0.4), "40%");
    }
}
