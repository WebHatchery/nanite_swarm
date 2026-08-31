//! Procedural building tile art.

mod core;
mod structures;

pub fn generate() {
    core::generate();
    structures::generate();
}
