use super::*;

#[test]
fn a_newly_colonized_world_already_knows_what_the_swarm_knows() {
    let mut campaign = campaign();
    campaign
        .research
        .unlocked_techs
        .push("efficient_drills".to_string());
    campaign.sync_research();

    assert!(campaign.colonize(0));
    campaign.travel_to(0);
    assert!(
        campaign
            .current()
            .stats
            .multiplier(crate::engine::StatId::DrillOutput)
            > 1.0,
        "the new world landed on starting-tier research"
    );
}

#[test]
fn research_reaches_every_world_not_just_the_one_underfoot() {
    let mut campaign = campaign();
    campaign.colonize(0);
    campaign.colonize(3);

    campaign
        .research
        .unlocked_techs
        .push("efficient_drills".to_string());
    campaign.sync_research();

    for index in [STARTING_PLANET, 0, 3] {
        campaign.travel_to(index);
        let planet = campaign.current();
        assert!(
            planet.stats.multiplier(crate::engine::StatId::DrillOutput) > 1.0,
            "world {index} is still running on stale research"
        );
    }
}

#[test]
fn a_building_unlocked_by_research_is_unlocked_everywhere() {
    let mut campaign = campaign();
    campaign.colonize(0);
    campaign
        .research
        .unlocked_techs
        .push("wind_power".to_string());
    campaign.sync_research();

    campaign.travel_to(0);
    assert!(campaign
        .current()
        .is_building_researched(BuildingType::WindTurbine));
}

#[test]
fn a_save_that_kept_research_on_the_planet_is_adopted_once() {
    let mut campaign = campaign();
    // What an older save looks like: the planet knows, the campaign does not.
    campaign.research.unlocked_techs.clear();
    campaign
        .current_mut()
        .research
        .unlocked_techs
        .push("efficient_drills".to_string());

    campaign.adopt_planet_research();

    assert!(campaign
        .research
        .unlocked_techs
        .iter()
        .any(|id| id == "efficient_drills"));
    assert!(
        campaign
            .current()
            .stats
            .multiplier(crate::engine::StatId::DrillOutput)
            > 1.0
    );

    // And it does not clobber research the campaign already has.
    let mut newer = campaign;
    newer.research.unlocked_techs = vec!["wind_power".to_string()];
    newer.adopt_planet_research();
    assert_eq!(
        newer.research.unlocked_techs,
        vec!["wind_power".to_string()]
    );
}
