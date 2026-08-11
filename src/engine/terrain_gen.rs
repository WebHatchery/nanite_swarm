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

/// A smooth deterministic field over the grid, in roughly 0..1.
///
/// Value noise: a hash at each lattice corner, smoothly interpolated between.
/// `scale` is how many tiles a feature spans — larger is smoother.
pub fn field(seed: u64, x: i32, y: i32, scale: f32) -> f32 {
    let scale = scale.max(1.0);
    let fx = x as f32 / scale;
    let fy = y as f32 / scale;
    let x0 = fx.floor();
    let y0 = fy.floor();
    let tx = smooth(fx - x0);
    let ty = smooth(fy - y0);

    let (x0, y0) = (x0 as i64, y0 as i64);
    let top = lerp(corner(seed, x0, y0), corner(seed, x0 + 1, y0), tx);
    let bottom = lerp(corner(seed, x0, y0 + 1), corner(seed, x0 + 1, y0 + 1), tx);
    lerp(top, bottom, ty)
}

/// Smoothstep, so the field has no creases along the lattice lines.
fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// A stable hash of a lattice corner, in 0..1.
fn corner(seed: u64, x: i64, y: i64) -> f32 {
    let mut h = seed
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^= h >> 33;
    (h >> 40) as f32 / (1u32 << 24) as f32
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
