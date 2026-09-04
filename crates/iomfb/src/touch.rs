//! Map normalized host touches onto the IOMFB pixel grid.
//!
//! This crate does not read HID, inject `wl_touch`, or park SpringBoard.
//! TrollStore apps, jailbreak tweaks, and Wawona supply the points.

use crate::{Display, Result};

/// Display size used to convert normalized (0..1) points to pixels.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TouchMap {
    pub width: u32,
    pub height: u32,
}

impl TouchMap {
    pub fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    pub fn from_display(display: &Display) -> Result<Self> {
        let (width, height) = display.size()?;
        Ok(Self { width, height })
    }

    /// Clamp a normalized point into pixel coordinates.
    pub fn pixel(&self, nx: f64, ny: f64) -> (u32, u32) {
        let x = (nx.clamp(0.0, 1.0) * f64::from(self.width.saturating_sub(1))).round() as u32;
        let y = (ny.clamp(0.0, 1.0) * f64::from(self.height.saturating_sub(1))).round() as u32;
        (x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corners_and_clamp() {
        let map = TouchMap::new(1290, 2796);
        assert_eq!(map.pixel(0.0, 0.0), (0, 0));
        assert_eq!(map.pixel(1.0, 1.0), (1289, 2795));
        assert_eq!(map.pixel(-1.0, 2.0), (0, 2795));
    }
}
