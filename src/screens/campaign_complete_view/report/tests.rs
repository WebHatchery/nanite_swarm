use super::{compact_core_stage_line, CompletionWorldRow};

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
