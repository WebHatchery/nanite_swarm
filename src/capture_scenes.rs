//! Scenes staged for the screenshot harness.
//!
//! `NANITE_SWARM_CAPTURE_PATH` boots the game straight into one of these,
//! simulates a fixed number of frames and writes a PNG, so a screen can be
//! looked at without playing up to it. Everything here exists to put the game
//! in a state worth photographing; none of it runs in a real session.

mod fixtures;
mod research;

#[allow(dead_code)]
pub const REQUIRED_CAPTURE_SCENES: [&str; 66] = [
    "mainmenu",
    "desktop_mainmenu_short",
    "desktop_settings_short",
    "desktop_settings_audio_short",
    "mobile_mainmenu_narrow",
    "mobile_settings_narrow",
    "mobile_settings_audio_narrow",
    "research",
    "mobile_research",
    "mobile_research_narrow",
    "desktop_seedship_short",
    "mobile_seedship",
    "logistics",
    "core_ring",
    "core_ring_focus",
    "mobile_core_ring",
    "assembly",
    "focus",
    "desktop_build",
    "desktop_info",
    "freight",
    "flow",
    "factory_deck",
    "mobile_factory_deck",
    "mobile_build",
    "mobile_info",
    "mobile_help",
    "mobile_help_narrow",
    "mobile_narrow",
    "mobile_launch",
    "mobile_arrival",
    "hibernation",
    "offline",
    "mobile_offline",
    "geothermal",
    "fusion",
    "ending",
    "mobile_ending_very_short",
    "desktop_ending",
    "interplanetary_remote_orders",
    "interplanetary_core_stages",
    "mobile_interplanetary_core_stages",
    "desktop_interplanetary_short",
    "mobile_remote_orders",
    "mobile_remote_recovery",
    "mobile_interplanetary_narrow",
    "mobile_interplanetary_rules",
    "mobile_interplanetary_alert",
    "mobile_records_narrow",
    "mobile_upkeep",
    "mobile_shipping_reserve",
    "feature_inspection",
    "partial_feature_inspection",
    "mobile_feature_inspection",
    "iron_belt_inspection",
    "iron_belt_drill_inspection",
    "mountain_wind_inspection",
    "forest_harvester_inspection",
    "filter_forest_inspection",
    "mobile_filter_forest_inspection",
    "mobile_mountain_wind_inspection",
    "mobile_forest_harvester_inspection",
    "acid_sea_inspection",
    "ring_shadow_inspection",
    "ice_rift_inspection",
    "landing_crater_inspection",
];

mod scenes;

#[cfg(test)]
mod tests;
