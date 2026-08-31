use super::*;

#[test]
fn a_completed_directive_pays_out_and_rotates() {
    let mut campaign = campaign();
    campaign.directive.completed = true;
    campaign.directive.reward_data = 25.0;
    let before = campaign.current().resources.data;

    campaign.update_directive(0.1);

    assert_eq!(campaign.current().resources.data, before + 25.0);
    assert!(!campaign.directive.completed);
    assert_eq!(campaign.directive_tier, 1);
}

#[test]
fn a_directive_that_is_met_says_so_instead_of_rotating_in_silence() {
    let mut campaign = campaign();
    campaign.directive.completed = true;
    campaign.directive.reward_data = 25.0;
    let goal = campaign.directive.description.clone();

    campaign.update_directive(0.1);

    let announced = campaign.current().notifications.get_notifications();
    assert_eq!(announced.len(), 1, "the directive rotated in silence");
    assert!(
        announced[0].message.contains(&goal),
        "the toast did not say which directive: {}",
        announced[0].message
    );
    assert!(announced[0].message.contains("25"), "no reward mentioned");
}

#[test]
fn a_directive_that_runs_out_of_time_is_reported_as_a_loss() {
    let mut campaign = campaign();
    let goal = campaign.directive.description.clone();
    campaign.update_directive(crate::directives::rotation_seconds(campaign.data().data()) + 1.0);

    let announced = campaign.current().notifications.get_notifications();
    assert_eq!(announced.len(), 1);
    assert!(announced[0].message.contains("lapsed"));
    assert!(announced[0].message.contains(&goal));
}

#[test]
fn offline_time_completes_and_pays_a_standing_directive() {
    let mut campaign = campaign();
    campaign.directive.kind = crate::directives::DirectiveKind::PowerSurplus;
    campaign.directive.target = 1;
    campaign.directive.hold_seconds = 2.0;
    campaign.directive.duration = 10.0;
    campaign.directive.reward_data = 25.0;
    campaign.directive.description = "Hold the line".to_string();
    campaign.current_mut().power_balance = 1.0;
    let before_data = campaign.current().resources.data;

    let completed = campaign.apply_offline_directive_progress(3.0);

    assert_eq!(completed, 1);
    assert_eq!(campaign.current().resources.data, before_data + 25.0);
    assert_eq!(campaign.directive_tier, 1);
    assert!(campaign.directive_history.last().unwrap().completed);
}
