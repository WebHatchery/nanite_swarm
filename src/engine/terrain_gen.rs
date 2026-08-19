//! Giving a world a shape.
//!
//! Terrain used to be an independent roll per tile, so every planet came out
//! as the same confetti: mountains one tile wide, forests that were never
//! twice the same size, and no reason to read the map as anything but a
//! backdrop. Ground is laid down from smooth seeded fields instead, so what
//! comes out is ridges, basins and belts.
//!
//! The shares are held exactly where they were. A field is thresholded at the
//! quantile that gives the declared weight, rather than at a fixed value, so a
//! world with 30% mountain still gets 30% mountain — in ridges rather than in
//! grains. Terrain weights are balance, and clustering must not quietly retune
//! them.

/// A smooth deterministic field over the grid, in roughly 0..1. Terrain owns
/// the meaning of the field; the generic seeded noise lives in the toolkit.
pub fn field(seed: u64, x: i32, y: i32, scale: f32) -> f32 {
    macroquad_toolkit::noise::seeded_value(seed, x, y, scale)
}

/// The value with `share` of `values` above it.
///
/// This is what keeps a clustered world honest: the field decides *where* the
/// high ground is, and this decides how much of it there is, so the declared
/// weights come out unchanged.
pub fn threshold_for_share(values: &mut [f32], share: f32) -> f32 {
    if values.is_empty() {
        return f32::INFINITY;
    }
    let share = share.clamp(0.0, 1.0);
    if share <= 0.0 {
        return f32::INFINITY;
    }
    if share >= 1.0 {
        return f32::NEG_INFINITY;
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let index = ((1.0 - share) * values.len() as f32).floor() as usize;
    values[index.min(values.len() - 1)]
}

#[cfg(test)]
mod tests;
