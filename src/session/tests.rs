use super::*;

#[test]
fn a_new_campaign_returns_the_order_editor_to_the_starting_world() {
    let data = GameDataContext::default();
    let mut session = GameSession::new(GameConfig::default(), data, 42);
    session.shipping_edit_world = 4;

    session.new_campaign(GameConfig::default(), 99);

    assert_eq!(session.shipping_edit_world, state::STARTING_PLANET);
}

#[test]
fn a_fresh_session_is_not_saveable_until_a_campaign_is_selected() {
    let session = GameSession::new(GameConfig::default(), GameDataContext::default(), 42);

    assert!(!session.has_active_campaign);
}

#[test]
fn an_unselected_session_does_not_stamp_a_fresh_campaign_when_saved() {
    let mut session = GameSession::new(GameConfig::default(), GameDataContext::default(), 42);
    let before = session.campaign.current().last_saved_unix;

    session.save_campaign().unwrap();

    assert_eq!(session.campaign.current().last_saved_unix, before);
}

#[test]
fn starting_a_new_campaign_marks_the_session_as_saveable() {
    let data = GameDataContext::default();
    let mut session = GameSession::new(GameConfig::default(), data, 42);

    session.new_campaign(GameConfig::default(), 99);

    assert!(session.has_active_campaign);
}

#[test]
fn starting_a_new_campaign_cancels_pending_slot_deletion() {
    let data = GameDataContext::default();
    let mut session = GameSession::new(GameConfig::default(), data, 42);
    session.delete_confirmation_slot = Some(session.active_slot);

    session.new_campaign(GameConfig::default(), 99);

    assert_eq!(session.delete_confirmation_slot, None);
}
