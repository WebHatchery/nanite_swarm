use super::alert_flow_label;

#[test]
fn alert_card_offers_flow_only_when_a_problem_exists() {
    assert_eq!(alert_flow_label(0, false), None);
    assert_eq!(alert_flow_label(1, false), Some("FLOW"));
    assert_eq!(alert_flow_label(2, true), Some("FLOW ON"));
}
