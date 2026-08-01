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
    /// How much of a world's acid rain still reaches the network.
    AcidResistance,
    /// How much of a world's cold still reaches the drones.
    FreezeResistance,
}

impl StatId {
    pub const ALL: [StatId; 10] = [
        StatId::DrillOutput,
        StatId::DroneCapacity,
        StatId::DronesPerDrill,
        StatId::DataGeneration,
        StatId::DustAccumulation,
        StatId::MineralCapacity,
        StatId::PowerConsumption,
        StatId::ResearchRate,
        StatId::AcidResistance,
        StatId::FreezeResistance,
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
            StatId::AcidResistance => "acid_resistance",
            StatId::FreezeResistance => "freeze_resistance",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        StatId::ALL.into_iter().find(|stat| stat.id() == id)
    }

    /// What this stat is called where a player can see it. Falls back to the
    /// raw id, which is what an unlabelled stat deserves.
    pub fn label(self) -> &'static str {
        crate::data::game_data()
            .research
            .stats
            .iter()
            .find(|entry| entry.id == self.id())
            .map(|entry| entry.label.as_str())
            .unwrap_or_else(|| self.id())
    }

    /// Whether a tech moving this stat down is doing the player a favour.
    pub fn lower_is_better(self) -> bool {
        crate::data::game_data()
            .research
            .stats
            .iter()
            .find(|entry| entry.id == self.id())
            .is_some_and(|entry| entry.lower_is_better)
    }
}

/// One declared modifier, said in words: what it moves, by how much, and
/// whether that is good news.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifierSummary {
    pub label: &'static str,
    pub change: String,
    /// False when the tech is charging the player something for the rest.
    pub is_gain: bool,
}

/// Describe a declared modifier for display. `None` for a modifier that does
/// not parse, which cannot happen in shipped data - it is rejected at load.
pub fn describe_modifier(def: &ModifierDef) -> Option<ModifierSummary> {
    let (stat, op, value) = parse_modifier(def).ok()?;
    let change = match op {
        ModifierOp::Percent => format!("{}{:.0}%", sign(value), value * 100.0),
        ModifierOp::Add => format!("{}{}", sign(value), trim_number(value)),
    };
    Some(ModifierSummary {
        label: stat.label(),
        change,
        is_gain: (value < 0.0) == stat.lower_is_better(),
    })
}

fn sign(value: f32) -> &'static str {
    if value < 0.0 {
        // The number already carries its minus.
        ""
    } else {
        "+"
    }
}

/// Whole numbers without a trailing `.0`, because "+1 drone" reads better
/// than "+1.0 drone".
fn trim_number(value: f32) -> String {
    if (value - value.round()).abs() < 0.001 {
        format!("{:.0}", value)
    } else {
        format!("{:.2}", value)
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

    /// Fold in declared modifiers from somewhere other than research, such as
    /// a finished Seed Ship stage. Unparseable entries are skipped here; they
    /// are rejected at data load.
    pub fn add_declared(&mut self, defs: &[ModifierDef]) {
        for def in defs {
            if let Ok((stat, op, value)) = parse_modifier(def) {
                self.push(stat, op, value);
            }
        }
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
    fn every_stat_the_game_knows_has_something_to_call_itself() {
        for stat in StatId::ALL {
            assert_ne!(
                stat.label(),
                stat.id(),
                "{} is only ever shown as its raw id",
                stat.id()
            );
        }
    }

    #[test]
    fn a_percent_is_said_as_a_percent_and_a_count_as_a_count() {
        let percent = describe_modifier(&modifier("drill_output", "percent", 0.25)).unwrap();
        assert_eq!(percent.change, "+25%");
        let count = describe_modifier(&modifier("drones_per_drill", "add", 1.0)).unwrap();
        assert_eq!(count.change, "+1");
        assert_eq!(count.label, StatId::DronesPerDrill.label());
    }

    #[test]
    fn less_of_a_bad_thing_reads_as_a_gain_and_more_of_it_does_not() {
        let quieter = describe_modifier(&modifier("dust_accumulation", "percent", -0.3)).unwrap();
        assert_eq!(quieter.change, "-30%");
        assert!(quieter.is_gain, "less dust is good news");

        let hungrier = describe_modifier(&modifier("power_consumption", "percent", 0.2)).unwrap();
        assert!(!hungrier.is_gain, "more power draw is a cost");

        // And the ordinary direction still holds for ordinary stats.
        let weaker = describe_modifier(&modifier("drill_output", "percent", -0.1)).unwrap();
        assert!(!weaker.is_gain);
    }

    #[test]
    fn a_modifier_the_game_cannot_read_is_not_described() {
        assert!(describe_modifier(&modifier("not_a_stat", "add", 1.0)).is_none());
        assert!(describe_modifier(&modifier("drill_output", "multiply", 1.0)).is_none());
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
