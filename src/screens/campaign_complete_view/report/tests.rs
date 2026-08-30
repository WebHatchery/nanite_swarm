use super::{
    compact_consumption_line, compact_core_stage_line, completion_consumption, format_consumption,
    CompletionWorldRow,
};
use crate::data::GameConfig;
use crate::state::{Campaign, ConsumptionTotals};

fn row(name: &str, stage_label: &str) -> CompletionWorldRow {
    CompletionWorldRow {
        name: name.to_string(),
        stage_label: stage_label.to_string(),
        structures: 0,
    }
}

#[test]
fn compact_report_keeps_world_identity_and_core_stage_together() {
    let rows = [row("Mercury", "C2/5"), row("Saturn", "C5/5")];

    assert_eq!(compact_core_stage_line(&rows), "Me C2/5 | Sa C5/5");
}

#[test]
fn compact_report_handles_empty_world_lists() {
    assert_eq!(compact_core_stage_line(&[]), "");
}

#[test]
fn completion_report_sums_consumption_across_every_colonized_world() {
    let mut campaign = Campaign::new(GameConfig::default(), 7);
    campaign.current_mut().resources.consumed = ConsumptionTotals {
        minerals: 12.0,
        data: 3.5,
        ..Default::default()
    };
    campaign.colonize(0);
    campaign.planet_mut(0).unwrap().resources.consumed = ConsumptionTotals {
        minerals: 8.0,
        components: 2.0,
        ..Default::default()
    };

    let total = completion_consumption(&campaign);
    assert_eq!(total.minerals, 20.0);
    assert_eq!(total.data, 3.5);
    assert_eq!(total.components, 2.0);
}

#[test]
fn compact_consumption_line_keeps_all_six_sinks_visible() {
    let mut campaign = Campaign::new(GameConfig::default(), 7);
    campaign.current_mut().resources.consumed = ConsumptionTotals {
        minerals: 1_250.0,
        energy: 9.0,
        data: 3.0,
        biomass: 2.0,
        alloy: 1.0,
        components: 0.5,
    };

    assert_eq!(format_consumption(1_250.0), "1.2k");
    assert_eq!(
        compact_consumption_line(&campaign),
        "M 1.2k | P 9.00 | D 3.00 | B 2.00 | A 1.00 | C 0.50"
    );
}
