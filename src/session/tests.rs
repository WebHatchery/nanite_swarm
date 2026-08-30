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
fn starting_a_new_campaign_cancels_pending_slot_deletion() {
    let data = GameDataContext::default();
    let mut session = GameSession::new(GameConfig::default(), data, 42);
    session.delete_confirmation_slot = Some(session.active_slot);

    session.new_campaign(GameConfig::default(), 99);

    assert_eq!(session.delete_confirmation_slot, None);
}
