use super::alert_flow_label;

#[test]
fn flow_button_only_appears_for_diagnosable_alerts() {
    assert_eq!(alert_flow_label(false, false), None);
    assert_eq!(alert_flow_label(true, false), Some("FLOW"));
    assert_eq!(alert_flow_label(true, true), Some("FLOW ON"));
}
