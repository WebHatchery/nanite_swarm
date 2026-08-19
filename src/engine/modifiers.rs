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
    /// Tiles a drone crosses per second.
    DroneSpeed,
    /// Tiles the grid carries power before it needs a repeater.
    RepeaterRange,
    /// What harvesting a tile of terrain by hand is worth.
    HarvestYield,
    CollapseShutdown,
    CollapseDataLoss,
    DustEfficiencyThreshold,
    DustSpeedThreshold,
    DustLeakThreshold,
    DustStallThreshold,
}

impl StatId {
    pub const ALL: [StatId; 19] = [
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
        StatId::DroneSpeed,
        StatId::RepeaterRange,
        StatId::HarvestYield,
        StatId::CollapseShutdown,
        StatId::CollapseDataLoss,
        StatId::DustEfficiencyThreshold,
        StatId::DustSpeedThreshold,
        StatId::DustLeakThreshold,
        StatId::DustStallThreshold,
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
            StatId::DroneSpeed => "drone_speed",
            StatId::RepeaterRange => "repeater_range",
            StatId::HarvestYield => "harvest_yield",
            StatId::CollapseShutdown => "collapse_shutdown",
            StatId::CollapseDataLoss => "collapse_data_loss",
            StatId::DustEfficiencyThreshold => "dust_efficiency_threshold",
            StatId::DustSpeedThreshold => "dust_speed_threshold",
            StatId::DustLeakThreshold => "dust_leak_threshold",
            StatId::DustStallThreshold => "dust_stall_threshold",
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
mod tests;
