//! GPU present types. Not an Apple header.
//!
//! On Apple, the Linux dma-buf object is an IOSurface. IOMFB
//! `SwapSetLayer` takes that IOSurface. Metal must wrap the same
//! object (`newTextureWithDescriptor:iosurface:plane:`). A CPU blit
//! is a fallback, never the Mode B Desktop path.

/// Confirmed IOMFB present fourcc from the tipa smoke (`'BGRA'`).
pub const PIXEL_FORMAT_BGRA: u32 = 0x4247_5241;

/// Triple-buffer the swapchain. Independent of IOMFB layer count (0-3).
pub const SWAPCHAIN_BUFFERS: usize = 3;

/// Present on layer 0. Desktop / full-frame apps use this layer.
pub const PRESENT_LAYER: i32 = 0;

/// Same slot mask Wawona Mode B HID uses (`touchId & 15`).
pub const TOUCH_SLOTS: usize = 16;

/// Wawona / UIKit HID sink states. 0=up, 1=down, 2=motion, 3=cancel.
pub const TOUCH_UP: i32 = 0;
pub const TOUCH_DOWN: i32 = 1;
pub const TOUCH_MOTION: i32 = 2;
pub const TOUCH_CANCEL: i32 = 3;

/// Input space for [`crate`] touch inject. 0=normalized, 1=view, 2=pixel, 3=hid.
pub const TOUCH_SPACE_NORMALIZED: i32 = 0;
pub const TOUCH_SPACE_VIEW: i32 = 1;
pub const TOUCH_SPACE_PIXEL: i32 = 2;
pub const TOUCH_SPACE_HID: i32 = 3;

/// `SwapWait` returned this after a successful set+end on guest 26.1.
/// Present itself succeeded. Treat as incomplete wait, not a failed frame.
pub const WAIT_INCOMPLETE_GUEST: i32 = -536_870_165; // 0xe000002b

/// How `SwapWait` finished after set+end returned 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WaitOutcome {
    Displayed,
    Incomplete(i32),
}

impl WaitOutcome {
    pub fn from_wait_rc(rc: i32) -> Self {
        if rc == crate::IOMFB_OK {
            Self::Displayed
        } else {
            Self::Incomplete(rc)
        }
    }

    pub fn is_displayed(self) -> bool {
        matches!(self, Self::Displayed)
    }
}

/// Result of one IOMFB commit. `set` and `end` were 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PresentStatus {
    pub token: i32,
    pub wait: WaitOutcome,
    /// Always true on the GPU path: same IOSurfaceID from producer to swap.
    pub zero_copy: bool,
}

/// Pixel format we will allocate for a new swapchain surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PixelFormat {
    Bgra8888,
}

impl PixelFormat {
    pub fn fourcc(self) -> u32 {
        match self {
            Self::Bgra8888 => PIXEL_FORMAT_BGRA,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bgra_fourcc_matches_ascii() {
        assert_eq!(PIXEL_FORMAT_BGRA, u32::from_be_bytes(*b"BGRA"));
    }

    #[test]
    fn guest_wait_is_incomplete_not_ok() {
        let w = WaitOutcome::from_wait_rc(WAIT_INCOMPLETE_GUEST);
        assert_eq!(w, WaitOutcome::Incomplete(WAIT_INCOMPLETE_GUEST));
        assert!(!w.is_displayed());
    }
}
