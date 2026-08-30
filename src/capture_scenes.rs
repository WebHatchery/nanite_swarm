//! Scenes staged for the screenshot harness.
//!
//! `NANITE_SWARM_CAPTURE_PATH` boots the game straight into one of these,
//! simulates a fixed number of frames and writes a PNG, so a screen can be
//! looked at without playing up to it. Everything here exists to put the game
//! in a state worth photographing; none of it runs in a real session.

mod fixtures;
mod research;

#[allow(dead_code)]
pub const REQUIRED_CAPTURE_SCENES: [&str; 29] = [
    "mainmenu",
    "research",
    "mobile_research",
    "mobile_research_narrow",
    "mobile_seedship",
    "logistics",
    "assembly",
    "focus",
    "freight",
    "flow",
    "factory_deck",
    "mobile_factory_deck",
    "mobile_build",
    "mobile_info",
    "mobile_help",
    "mobile_narrow",
    "mobile_launch",
    "mobile_arrival",
    "hibernation",
    "offline",
    "mobile_offline",
    "geothermal",
    "fusion",
    "interplanetary_remote_orders",
    "mobile_remote_orders",
    "mobile_interplanetary_rules",
    "mobile_interplanetary_alert",
    "mobile_upkeep",
    "mobile_shipping_reserve",
];

mod scenes;

#[cfg(test)]
mod tests;
