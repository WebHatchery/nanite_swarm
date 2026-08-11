//! Grid coordinate primitive

use macroquad_toolkit::grid::TilePos;
use serde::{Deserialize, Serialize};

/// Grid position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GridPos {
    pub x: i32,
    pub y: i32,
}

impl GridPos {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Check if position is within grid bounds
    pub fn in_bounds(&self, width: u32, height: u32) -> bool {
        self.x >= 0 && self.y >= 0 && (self.x as u32) < width && (self.y as u32) < height
    }

    /// Get adjacent positions (4-directional)
    pub fn neighbors(&self) -> [GridPos; 4] {
        [
            GridPos::new(self.x - 1, self.y),
            GridPos::new(self.x + 1, self.y),
            GridPos::new(self.x, self.y - 1),
            GridPos::new(self.x, self.y + 1),
        ]
    }

    /// Manhattan distance to another position
    pub fn distance(&self, other: GridPos) -> u32 {
        ((self.x - other.x).abs() + (self.y - other.y).abs()) as u32
    }

    /// Convert to array index for flat storage
    pub fn to_index(&self, width: u32) -> usize {
        (self.y as u32 * width + self.x as u32) as usize
    }

    /// Create from array index
    pub fn from_index(index: usize, width: u32) -> Self {
        Self {
            x: (index as u32 % width) as i32,
            y: (index as u32 / width) as i32,
        }
    }

    pub(crate) fn to_tile_pos(self) -> TilePos {
        TilePos::new(self.x, self.y)
    }

    pub(crate) fn from_tile_pos(pos: TilePos) -> Self {
        Self::new(pos.x, pos.y)
    }
}

#[cfg(test)]
mod tests;
