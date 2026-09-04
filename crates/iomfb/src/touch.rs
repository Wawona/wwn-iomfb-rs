//! Map host touches onto the IOMFB pixel grid and track a 16-slot seat.
//!
//! This crate does not read HID, inject `wl_touch`, or park SpringBoard.
//! TrollStore apps, jailbreak tweaks, and Wawona supply the points.
//! The mapping matches Wawona Mode B (`touchId`, state 0/1/2/3, 16 slots).

use crate::{Display, Result};
use iomfb_abi::{
    TOUCH_CANCEL, TOUCH_DOWN, TOUCH_MOTION, TOUCH_SLOTS, TOUCH_SPACE_HID, TOUCH_SPACE_NORMALIZED,
    TOUCH_SPACE_PIXEL, TOUCH_SPACE_VIEW, TOUCH_UP,
};

/// Wawona HID / overlay states. Same numbers as `WWNModeBHidSink`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TouchState {
    Up = TOUCH_UP,
    Down = TOUCH_DOWN,
    Motion = TOUCH_MOTION,
    Cancel = TOUCH_CANCEL,
}

impl TouchState {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            TOUCH_UP => Some(Self::Up),
            TOUCH_DOWN => Some(Self::Down),
            TOUCH_MOTION => Some(Self::Motion),
            TOUCH_CANCEL => Some(Self::Cancel),
            _ => None,
        }
    }

    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Where the incoming (x, y) lives before mapping.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TouchSpace {
    /// 0..1 of the dest rect (or the full display).
    Normalized = TOUCH_SPACE_NORMALIZED,
    /// UIKit / overlay view points. Requires [`TouchMap::set_view`].
    View = TOUCH_SPACE_VIEW,
    /// Already IOMFB display pixels.
    Pixel = TOUCH_SPACE_PIXEL,
    /// Raw digitizer / IOHID. Same heuristic as Wawona `wwn_map_hid_point`.
    Hid = TOUCH_SPACE_HID,
}

impl TouchSpace {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            TOUCH_SPACE_NORMALIZED => Some(Self::Normalized),
            TOUCH_SPACE_VIEW => Some(Self::View),
            TOUCH_SPACE_PIXEL => Some(Self::Pixel),
            TOUCH_SPACE_HID => Some(Self::Hid),
            _ => None,
        }
    }
}

/// One mapped contact in display pixels and normalized dest space.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TouchEvent {
    pub id: i32,
    pub state: TouchState,
    pub slot: u8,
    pub x: u32,
    pub y: u32,
    pub nx: f64,
    pub ny: f64,
}

/// Display size used to convert host points to IOMFB pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TouchMap {
    pub width: u32,
    pub height: u32,
    view_w: f64,
    view_h: f64,
    dest_x: f64,
    dest_y: f64,
    dest_w: f64,
    dest_h: f64,
    rotation_deg: i32,
}

impl TouchMap {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            view_w: 0.0,
            view_h: 0.0,
            dest_x: 0.0,
            dest_y: 0.0,
            dest_w: f64::from(width),
            dest_h: f64::from(height),
            rotation_deg: 0,
        }
    }

    pub fn from_display(display: &Display) -> Result<Self> {
        let (width, height) = display.size()?;
        Ok(Self::new(width, height))
    }

    /// UIKit / overlay view size in points. Used by [`TouchSpace::View`] and HID.
    pub fn set_view(&mut self, width: f64, height: f64) {
        self.view_w = width.max(0.0);
        self.view_h = height.max(0.0);
    }

    /// `SwapSetLayer` dest rect. Normalized and view points land here.
    pub fn set_dest(&mut self, x: f64, y: f64, w: f64, h: f64) {
        self.dest_x = x;
        self.dest_y = y;
        self.dest_w = w.max(1.0);
        self.dest_h = h.max(1.0);
    }

    /// `SetRenderingAngle` multiples of 90. Rotates mapped pixels.
    pub fn set_rotation(&mut self, degrees: i32) {
        let d = ((degrees % 360) + 360) % 360;
        self.rotation_deg = (d / 90) * 90;
    }

    /// Clamp a normalized point into pixel coordinates.
    pub fn pixel(&self, nx: f64, ny: f64) -> (u32, u32) {
        self.map(nx, ny, TouchSpace::Normalized)
    }

    pub fn map(&self, x: f64, y: f64, space: TouchSpace) -> (u32, u32) {
        let (vx, vy) = self.into_dest(x, y, space);
        let dw = if self.dest_w > 0.0 {
            self.dest_w
        } else {
            f64::from(self.width.max(1))
        };
        let dh = if self.dest_h > 0.0 {
            self.dest_h
        } else {
            f64::from(self.height.max(1))
        };
        let px = self.dest_x + vx.clamp(0.0, 1.0) * (dw - 1.0).max(0.0);
        let py = self.dest_y + vy.clamp(0.0, 1.0) * (dh - 1.0).max(0.0);
        let max_x = f64::from(self.width.saturating_sub(1));
        let max_y = f64::from(self.height.saturating_sub(1));
        self.rotate_pixel(
            px.round().clamp(0.0, max_x) as u32,
            py.round().clamp(0.0, max_y) as u32,
        )
    }

    pub fn normalize(&self, x: u32, y: u32) -> (f64, f64) {
        let dw = self.dest_w.max(1.0);
        let dh = self.dest_h.max(1.0);
        let nx = ((f64::from(x) - self.dest_x) / (dw - 1.0).max(1.0)).clamp(0.0, 1.0);
        let ny = ((f64::from(y) - self.dest_y) / (dh - 1.0).max(1.0)).clamp(0.0, 1.0);
        (nx, ny)
    }

    fn into_dest(&self, x: f64, y: f64, space: TouchSpace) -> (f64, f64) {
        match space {
            TouchSpace::Normalized => (x, y),
            TouchSpace::View => self.view_to_dest(x, y),
            TouchSpace::Pixel => {
                let dw = self.dest_w.max(1.0);
                let dh = self.dest_h.max(1.0);
                (
                    (x - self.dest_x) / (dw - 1.0).max(1.0),
                    (y - self.dest_y) / (dh - 1.0).max(1.0),
                )
            }
            TouchSpace::Hid => {
                let (vx, vy) = self.hid_to_view(x, y);
                self.view_to_dest(vx, vy)
            }
        }
    }

    fn view_to_dest(&self, x: f64, y: f64) -> (f64, f64) {
        let (vw, vh) = self.view_or_display();
        (x / vw.max(1.0), y / vh.max(1.0))
    }

    fn view_or_display(&self) -> (f64, f64) {
        if self.view_w >= 1.0 && self.view_h >= 1.0 {
            (self.view_w, self.view_h)
        } else {
            (f64::from(self.width.max(1)), f64::from(self.height.max(1)))
        }
    }

    /// Same three-way heuristic as Wawona `wwn_map_hid_point`.
    fn hid_to_view(&self, raw_x: f64, raw_y: f64) -> (f64, f64) {
        let (vw, vh) = self.view_or_display();
        if vw < 1.0 || vh < 1.0 {
            return (raw_x, raw_y);
        }
        if (0.0..=1.5).contains(&raw_x) && (0.0..=1.5).contains(&raw_y) {
            return (raw_x * vw, raw_y * vh);
        }
        let dw = f64::from(self.width.max(1));
        let dh = f64::from(self.height.max(1));
        if raw_x > vw * 1.5 || raw_y > vh * 1.5 {
            return (raw_x * vw / dw, raw_y * vh / dh);
        }
        (raw_x, raw_y)
    }

    fn rotate_pixel(&self, x: u32, y: u32) -> (u32, u32) {
        let w = self.width.saturating_sub(1);
        let h = self.height.saturating_sub(1);
        match self.rotation_deg {
            90 => (y, w.saturating_sub(x)),
            180 => (w.saturating_sub(x), h.saturating_sub(y)),
            270 => (h.saturating_sub(y), x),
            _ => (x, y),
        }
    }
}

/// Multi-touch tracker. Slot is `id & 15`, same as Wawona Mode B HID.
pub struct TouchSeat {
    map: TouchMap,
    down: [bool; TOUCH_SLOTS],
}

impl TouchSeat {
    pub fn new(map: TouchMap) -> Self {
        Self {
            map,
            down: [false; TOUCH_SLOTS],
        }
    }

    pub fn from_display(display: &Display) -> Result<Self> {
        Ok(Self::new(TouchMap::from_display(display)?))
    }

    pub fn map(&self) -> &TouchMap {
        &self.map
    }

    pub fn map_mut(&mut self) -> &mut TouchMap {
        &mut self.map
    }

    pub fn slot(id: i32) -> u8 {
        (id as u32 & (TOUCH_SLOTS as u32 - 1)) as u8
    }

    pub fn active_count(&self) -> usize {
        self.down.iter().filter(|d| **d).count()
    }

    /// Map and update seat state. `None` if an up/cancel has no matching down
    /// (same swallow as Wawona HID).
    pub fn inject(
        &mut self,
        id: i32,
        state: TouchState,
        x: f64,
        y: f64,
        space: TouchSpace,
    ) -> Option<TouchEvent> {
        let slot = Self::slot(id) as usize;
        let was = self.down[slot];
        let state = match state {
            TouchState::Motion if !was => TouchState::Down,
            TouchState::Down if was => TouchState::Motion,
            TouchState::Up | TouchState::Cancel if !was => return None,
            other => other,
        };
        match state {
            TouchState::Down | TouchState::Motion => self.down[slot] = true,
            TouchState::Up | TouchState::Cancel => self.down[slot] = false,
        }
        let (px, py) = self.map.map(x, y, space);
        let (nx, ny) = self.map.normalize(px, py);
        Some(TouchEvent {
            id,
            state,
            slot: slot as u8,
            x: px,
            y: py,
            nx,
            ny,
        })
    }

    pub fn cancel_all(&mut self) -> Vec<TouchEvent> {
        let mut out = Vec::new();
        for slot in 0..TOUCH_SLOTS {
            if !self.down[slot] {
                continue;
            }
            self.down[slot] = false;
            let id = slot as i32;
            let (px, py) = self.map.pixel(0.0, 0.0);
            let (nx, ny) = self.map.normalize(px, py);
            out.push(TouchEvent {
                id,
                state: TouchState::Cancel,
                slot: slot as u8,
                x: px,
                y: py,
                nx,
                ny,
            });
        }
        out
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

    #[test]
    fn view_space_maps_to_display() {
        let mut map = TouchMap::new(1290, 2796);
        map.set_view(430.0, 932.0);
        let (x, y) = map.map(215.0, 466.0, TouchSpace::View);
        assert!((640..=650).contains(&x), "{x}");
        assert!((1395..=1405).contains(&y), "{y}");
    }

    #[test]
    fn hid_normalized_matches_wawona() {
        let mut map = TouchMap::new(1290, 2796);
        map.set_view(430.0, 932.0);
        assert_eq!(map.map(0.0, 0.0, TouchSpace::Hid), map.pixel(0.0, 0.0));
        let (x, y) = map.map(1.0, 1.0, TouchSpace::Hid);
        assert_eq!((x, y), map.pixel(1.0, 1.0));
    }

    #[test]
    fn hid_pixel_scales_into_view() {
        let mut map = TouchMap::new(1290, 2796);
        map.set_view(430.0, 932.0);
        let (x, y) = map.map(1290.0, 2796.0, TouchSpace::Hid);
        assert_eq!((x, y), map.pixel(1.0, 1.0));
    }

    #[test]
    fn dest_rect_letterbox() {
        let mut map = TouchMap::new(1290, 2796);
        map.set_dest(0.0, 348.0, 1290.0, 2100.0);
        let (x, y) = map.pixel(0.5, 0.0);
        assert!((640..=650).contains(&x));
        assert!((347..=349).contains(&y), "{y}");
    }

    #[test]
    fn rotation_90() {
        let mut map = TouchMap::new(100, 200);
        map.set_rotation(90);
        assert_eq!(map.pixel(0.0, 0.0), (0, 99));
        assert_eq!(map.pixel(1.0, 0.0), (0, 0));
    }

    #[test]
    fn seat_down_move_up() {
        let mut seat = TouchSeat::new(TouchMap::new(1290, 2796));
        let down = seat
            .inject(3, TouchState::Down, 0.25, 0.5, TouchSpace::Normalized)
            .unwrap();
        assert_eq!(down.slot, 3);
        assert_eq!(down.state, TouchState::Down);
        assert_eq!(seat.active_count(), 1);
        let mv = seat
            .inject(3, TouchState::Motion, 0.5, 0.5, TouchSpace::Normalized)
            .unwrap();
        assert_eq!(mv.state, TouchState::Motion);
        let up = seat
            .inject(3, TouchState::Up, 0.5, 0.5, TouchSpace::Normalized)
            .unwrap();
        assert_eq!(up.state, TouchState::Up);
        assert_eq!(seat.active_count(), 0);
        assert!(seat
            .inject(3, TouchState::Up, 0.5, 0.5, TouchSpace::Normalized)
            .is_none());
    }

    #[test]
    fn seat_motion_implies_down() {
        let mut seat = TouchSeat::new(TouchMap::new(100, 100));
        let ev = seat
            .inject(1, TouchState::Motion, 0.1, 0.1, TouchSpace::Normalized)
            .unwrap();
        assert_eq!(ev.state, TouchState::Down);
        assert_eq!(seat.active_count(), 1);
    }

    #[test]
    fn seat_cancel_all() {
        let mut seat = TouchSeat::new(TouchMap::new(100, 100));
        let _ = seat.inject(1, TouchState::Down, 0.0, 0.0, TouchSpace::Normalized);
        let _ = seat.inject(2, TouchState::Down, 1.0, 1.0, TouchSpace::Normalized);
        let cancelled = seat.cancel_all();
        assert_eq!(cancelled.len(), 2);
        assert_eq!(seat.active_count(), 0);
    }
}
