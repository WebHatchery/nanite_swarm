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
    campaign.update_directive(crate::directives::rotation_seconds() + 1.0);

    let announced = campaign.current().notifications.get_notifications();
    assert_eq!(announced.len(), 1);
    assert!(announced[0].message.contains("lapsed"));
    assert!(announced[0].message.contains(&goal));
}
