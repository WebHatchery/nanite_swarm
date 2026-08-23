//! Scenes staged for the screenshot harness.
//!
//! `NANITE_SWARM_CAPTURE_PATH` boots the game straight into one of these,
//! simulates a fixed number of frames and writes a PNG, so a screen can be
//! looked at without playing up to it. Everything here exists to put the game
//! in a state worth photographing; none of it runs in a real session.

mod fixtures;

#[allow(dead_code)]
pub const REQUIRED_CAPTURE_SCENES: [&str; 8] = [
    "mainmenu",
    "research",
    "logistics",
    "assembly",
    "focus",
    "freight",
    "flow",
    "factory_deck",
];

mod scenes;

#[cfg(test)]
mod tests;
