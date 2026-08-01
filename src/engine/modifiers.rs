//! Declared stat modifiers.
//!
//! Research nodes do not reach into the simulation; they declare what they
//! change in `research.json`, and the simulation asks for a stat. Every effect
//! is one of these, so a new tech is data and a new effect is one enum variant
//! plus the one place that reads it — not a `unlocked_techs.contains("...")`
//! sprinkled wherever it happened to be needed.

use crate::data::ModifierDef;

/// A value the simulation reads and research can bend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatId {
    /// Minerals a drill produces per second.
    DrillOutput,
    /// Load a single drone can carry.
    DroneCapacity,
    /// Drones a single drill keeps in service.
    DronesPerDrill,
    /// Data produced by the Core and Server Banks.
    DataGeneration,
    /// Dust settling on buildings per second.
    DustAccumulation,
    /// Ceiling on stored minerals.
    MineralCapacity,
    /// Power drawn by every consumer.
    PowerConsumption,
    /// Data spent on the current research per second.
    ResearchRate,
}

impl StatId {
    pub const ALL: [StatId; 8] = [
        StatId::DrillOutput,
        StatId::DroneCapacity,
        StatId::DronesPerDrill,
        StatId::DataGeneration,
        StatId::DustAccumulation,
        StatId::MineralCapacity,
        StatId::PowerConsumption,
        StatId::ResearchRate,
    ];

    pub fn id(self) -> &'static str {
        match self {
            StatId::DrillOutput => "drill_output",
            StatId::DroneCapacity => "drone_capacity",
            StatId::DronesPerDrill => "drones_per_drill",
            StatId::DataGeneration => "data_generation",
            StatId::DustAccumulation => "dust_accumulation",
            StatId::MineralCapacity => "mineral_capacity",
            StatId::PowerConsumption => "power_consumption",
            StatId::ResearchRate => "research_rate",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        StatId::ALL.into_iter().find(|stat| stat.id() == id)
    }
}

/// How a modifier folds into a stat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModifierOp {
    /// Added to the base before any scaling.
    Add,
    /// Summed with every other percent on the stat, then applied once, so two
    /// techs never depend on which was researched first.
    Percent,
}

impl ModifierOp {
    pub fn id(self) -> &'static str {
        match self {
            ModifierOp::Add => "add",
            ModifierOp::Percent => "percent",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        match id {
            "add" => Some(ModifierOp::Add),
            "percent" => Some(ModifierOp::Percent),
            _ => None,
        }
    }
}

/// Read a declared modifier, naming what is wrong when it is not one.
pub fn parse_modifier(def: &ModifierDef) -> Result<(StatId, ModifierOp, f32), String> {
    let stat = StatId::from_id(&def.stat).ok_or_else(|| {
        let known: Vec<&str> = StatId::ALL.iter().map(|stat| stat.id()).collect();
        format!(
            "unknown stat \"{}\" (known stats: {})",
            def.stat,
            known.join(", ")
        )
    })?;
    let op = ModifierOp::from_id(&def.op).ok_or_else(|| {
        format!(
            "unknown modifier op \"{}\" (expected add or percent)",
            def.op
        )
    })?;
    Ok((stat, op, def.value))
}

/// The stat sheet produced by everything currently unlocked.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Stats {
    add: [f32; StatId::ALL.len()],
    percent: [f32; StatId::ALL.len()],
}

impl Stats {
    /// Fold every modifier declared by the unlocked research into one sheet.
    /// Unparseable modifiers are skipped here; they are rejected at data load.
    pub fn from_unlocked(unlocked: &[String]) -> Self {
        let mut stats = Stats::default();
        for node in &crate::data::game_data().research.nodes {
            if !unlocked.iter().any(|id| id == &node.id) {
                continue;
            }
            for modifier in &node.modifiers {
                if let Ok((stat, op, value)) = parse_modifier(modifier) {
                    stats.push(stat, op, value);
                }
            }
        }
        stats
    }

    fn push(&mut self, stat: StatId, op: ModifierOp, value: f32) {
        let index = stat as usize;
        match op {
            ModifierOp::Add => self.add[index] += value,
            ModifierOp::Percent => self.percent[index] += value,
        }
    }

    /// Apply the stat to a base value: `(base + adds) * (1 + percents)`.
    pub fn apply(&self, stat: StatId, base: f32) -> f32 {
        let index = stat as usize;
        (base + self.add[index]) * (1.0 + self.percent[index])
    }

    /// The stat as a pure multiplier, for values with no meaningful base.
    pub fn multiplier(&self, stat: StatId) -> f32 {
        self.apply(stat, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn modifier(stat: &str, op: &str, value: f32) -> ModifierDef {
        ModifierDef {
            stat: stat.to_string(),
            op: op.to_string(),
            value,
        }
    }

    #[test]
    fn stat_ids_round_trip_through_their_names() {
        for stat in StatId::ALL {
            assert_eq!(StatId::from_id(stat.id()), Some(stat));
        }
        assert_eq!(StatId::from_id("not_a_stat"), None);
    }

    #[test]
    fn parse_modifier_names_the_unknown_stat_and_lists_the_known_ones() {
        let error = parse_modifier(&modifier("wobble", "add", 1.0)).unwrap_err();
        assert!(error.contains("wobble"), "{error}");
        assert!(error.contains("drill_output"), "{error}");
    }

    #[test]
    fn parse_modifier_rejects_an_unknown_op() {
        let error = parse_modifier(&modifier("drill_output", "divide", 2.0)).unwrap_err();
        assert!(error.contains("divide"), "{error}");
    }

    #[test]
    fn an_empty_sheet_leaves_every_base_value_alone() {
        let stats = Stats::default();
        for stat in StatId::ALL {
            assert_eq!(stats.apply(stat, 7.0), 7.0);
            assert_eq!(stats.multiplier(stat), 1.0);
        }
    }

    #[test]
    fn percents_sum_before_they_apply_so_order_never_matters() {
        let mut forward = Stats::default();
        forward.push(StatId::DrillOutput, ModifierOp::Percent, 0.5);
        forward.push(StatId::DrillOutput, ModifierOp::Percent, 0.25);

        let mut backward = Stats::default();
        backward.push(StatId::DrillOutput, ModifierOp::Percent, 0.25);
        backward.push(StatId::DrillOutput, ModifierOp::Percent, 0.5);

        assert_eq!(forward, backward);
        // 10 * (1 + 0.75), not 10 * 1.5 * 1.25.
        assert_eq!(forward.apply(StatId::DrillOutput, 10.0), 17.5);
    }

    #[test]
    fn adds_land_before_percents() {
        let mut stats = Stats::default();
        stats.push(StatId::MineralCapacity, ModifierOp::Add, 50.0);
        stats.push(StatId::MineralCapacity, ModifierOp::Percent, 1.0);
        assert_eq!(stats.apply(StatId::MineralCapacity, 100.0), 300.0);
    }

    #[test]
    fn a_negative_percent_reduces_the_stat() {
        let mut stats = Stats::default();
        stats.push(StatId::PowerConsumption, ModifierOp::Percent, -0.25);
        assert_eq!(stats.apply(StatId::PowerConsumption, 8.0), 6.0);
    }

    #[test]
    fn modifiers_only_count_once_their_node_is_unlocked() {
        let locked = Stats::from_unlocked(&[]);
        assert_eq!(locked.multiplier(StatId::DrillOutput), 1.0);

        let unlocked = Stats::from_unlocked(&["efficient_drills".to_string()]);
        assert!(unlocked.multiplier(StatId::DrillOutput) > 1.0);
    }

    #[test]
    fn the_shipped_research_tree_declares_only_valid_modifiers() {
        for node in &crate::data::game_data().research.nodes {
            for modifier in &node.modifiers {
                assert!(
                    parse_modifier(modifier).is_ok(),
                    "node {} declares an invalid modifier: {:?}",
                    node.id,
                    modifier
                );
            }
        }
    }
}
