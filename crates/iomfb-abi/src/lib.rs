//! Reconstructed IOMFB types and claim status.
//!
//! Nothing here is an Apple header. Selectors and arities start
//! `Unconfirmed` until `docs/ABI.md` records a guest 26.1 Ghidra pass.

mod census;
mod exports;
mod gpu;

pub use census::{export as export_meta, Export, EXPORTS};
pub use exports::PUBLIC_EXPORTS;
pub use gpu::{
    PixelFormat, PresentStatus, WaitOutcome, PIXEL_FORMAT_BGRA, PRESENT_LAYER, SWAPCHAIN_BUFFERS,
    TOUCH_CANCEL, TOUCH_DOWN, TOUCH_MOTION, TOUCH_SLOTS, TOUCH_SPACE_HID, TOUCH_SPACE_NORMALIZED,
    TOUCH_SPACE_PIXEL, TOUCH_SPACE_VIEW, TOUCH_UP, WAIT_INCOMPLETE_GUEST,
};

use core::fmt;

/// Prior-art / our-finding status for one ABI row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClaimStatus {
    Unconfirmed,
    Confirmed,
    Refuted,
    Absent,
}

impl fmt::Display for ClaimStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unconfirmed => f.write_str("unconfirmed"),
            Self::Confirmed => f.write_str("confirmed"),
            Self::Refuted => f.write_str("refuted"),
            Self::Absent => f.write_str("absent"),
        }
    }
}

/// Userspace `dlopen` path. The framework directory is a stub; the
/// implementation lives in the dyld shared cache.
pub const FRAMEWORK_PATH: &str =
    "/System/Library/PrivateFrameworks/IOMobileFramebuffer.framework/IOMobileFramebuffer";

/// Kernel userclient class name for tipa entitlements.
pub const USERCLIENT_CLASS: &str = "IOMobileFramebufferUserClient";

/// Confirmed userclient selectors on guest-class iOS 26.1 / 23B85.
pub mod lead_selector {
    pub const GET_DEFAULT_SURFACE: u32 = 3;
    pub const SWAP_BEGIN: u32 = 4;
    pub const SWAP_END: u32 = 5;
    pub const SWAP_WAIT: u32 = 6;
    pub const GET_ID: u32 = 7;
    pub const GET_DISPLAY_SIZE: u32 = 8;
    /// Wiki sel 9 (`setVSyncNotifications`) is refuted. Vsync is notify type 5 / sel 0x48.
    pub const REQUEST_POWER: u32 = 12;
    pub const ENABLE_DISABLE_POWER_SAVINGS: u32 = 0x21;
    pub const NOTIFICATIONS: u32 = 0x48;
    pub const SWAP_GET_CURRENT: u32 = 0x5b;
    pub const SWAP_CANCEL_ALL: u32 = 0x51;
    pub const SWAP_CANCEL_ALL_GET_CURRENT: u32 = 0x5c;
    pub const IS_MAIN_DISPLAY: u32 = 0x12;
    /// aiaf writes `0x34`. Wawona notes wrote decimal 52. Same number.
    pub const SWAP_CANCEL: u32 = 0x34;
    pub const SET_DEBUG_FLAGS: u32 = 0xf;
    pub const SET_GAMMA_TABLE: u32 = 0x11;
    pub const SET_WHITE_ON_BLACK: u32 = 0x13;
    pub const SET_DISPLAY_DEVICE: u32 = 0x16;
    pub const GET_GAMMA_TABLE: u32 = 0x1b;
    pub const SET_BRIGHTNESS_CORRECTION: u32 = 0x32;
    pub const SET_COLOR_REMAP: u32 = 0x33;
    pub const GET_COLOR_REMAP: u32 = 0x39;
    pub const GET_BLOCK: u32 = 0xaa;
    pub const KERNEL_TESTS: u32 = 0x38;
    pub const HDCP_SEND: u32 = 0x2f;
    pub const HDCP_REPLY: u32 = 0x30;
    pub const FACTORY_PORTAL: u32 = 0x4b;
    pub const COPY_LAYER_DISPLAYED: u32 = 0x53;
}

/// Guest Get/SetGammaTable struct size.
pub const GAMMA_TABLE_SIZE: usize = 0xc0c;

/// `IOMobileFramebufferKernelTests` argument struct. Selector `0x38`.
pub const KERNEL_TESTS_SIZE: usize = 0x9c;

/// `_kern_SwapEnd` struct size on guest 26.1 / 23B85. Lives at `fb+0x18`.
pub const SWAP_ARG_SIZE: usize = 0x560;

/// Public `dlsym` names we care about first.
pub const SWAP_FAMILY: &[&str] = &[
    "IOMobileFramebufferGetMainDisplay",
    "IOMobileFramebufferGetSecondaryDisplay",
    "IOMobileFramebufferGetDisplaySize",
    "IOMobileFramebufferSwapBegin",
    "IOMobileFramebufferSwapSetLayer",
    "IOMobileFramebufferSwapEnd",
    "IOMobileFramebufferSwapWait",
    "IOMobileFramebufferSwapCancel",
    "IOMobileFramebufferGetLayerDefaultSurface",
];

pub const POWER_FAMILY: &[&str] = &[
    "IOMobileFramebufferEnableDisableVideoPowerSavings",
    "IOMobileFramebufferRequestPowerChange",
    "IOMobileFramebufferEnableVSyncNotifications",
    "IOMobileFramebufferDisableVSyncNotifications",
    "IOMobileFramebufferGetVSyncRunLoopSource",
];

/// Notify type stuffed into Enable/DisableNotifications (sel 0x48).
pub mod notify_type {
    pub const HOT_PLUG: i32 = 0;
    pub const POWER: i32 = 1;
    pub const HDCP: i32 = 2;
    pub const CRC: i32 = 4;
    pub const VSYNC: i32 = 5;
    pub const NEED_SWAP: i32 = 6;
}

/// Wawona Mode B used 3. Guest `_kern_SwapSetLayer` is `cmp w1, #4` (layers 0-3).
pub const LAYER_COUNT_WAWONA_LEAD: u32 = 3;
pub const LAYER_COUNT_CONFIRMED: u32 = 4;

/// `SwapWait` options lead: 0 means until displayed (Wawona).
pub const WAIT_UNTIL_DISPLAYED_LEAD: i32 = 0;

/// Opaque framebuffer connection.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct DisplayRef(pub *mut core::ffi::c_void);

unsafe impl Send for DisplayRef {}
unsafe impl Sync for DisplayRef {}

/// IOMFB return. Same width as `IOReturn` / `int32`.
pub type IomfbReturn = i32;

pub const IOMFB_OK: IomfbReturn = 0;

/// Pixel size. Guest may store this as `CGSize` or `{u32,u32}`.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DisplaySize {
    pub width: f64,
    pub height: f64,
}

/// Integer size lead from the wiki userclient struct.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DisplaySizeU32 {
    pub width: u32,
    pub height: u32,
}

/// Damage rectangle in pixels.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Damage {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// One acquired IOSurface.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Surface {
    pub iosurface: *mut core::ffi::c_void,
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub bytes_per_row: u32,
}

impl Default for Surface {
    fn default() -> Self {
        Self {
            iosurface: core::ptr::null_mut(),
            id: 0,
            width: 0,
            height: 0,
            bytes_per_row: 0,
        }
    }
}

/// One row in `docs/CLAIMS.md`.
#[derive(Clone, Copy, Debug)]
pub struct Claim {
    pub id: &'static str,
    pub status: ClaimStatus,
    pub summary: &'static str,
}

/// Ingest table. Status matches `docs/CLAIMS.md` after the 26.1 pass.
pub const CLAIMS: &[Claim] = &[
    Claim {
        id: "S1-main",
        status: ClaimStatus::Confirmed,
        summary: "GetMainDisplay(fb*)",
    },
    Claim {
        id: "S1-secondary",
        status: ClaimStatus::Confirmed,
        summary: "GetSecondaryDisplay fallback",
    },
    Claim {
        id: "S1-size-cgsize",
        status: ClaimStatus::Confirmed,
        summary: "GetDisplaySize writes CGSize",
    },
    Claim {
        id: "S1-swap-begin",
        status: ClaimStatus::Confirmed,
        summary: "SwapBegin(fb, int* token)",
    },
    Claim {
        id: "S1-swap-end-1arg",
        status: ClaimStatus::Confirmed,
        summary: "Public SwapEnd(fb) is 1-arg",
    },
    Claim {
        id: "S1-swap-wait-0",
        status: ClaimStatus::Confirmed,
        summary: "SwapWait options 0 = until displayed (guest 0xe000002b is incomplete)",
    },
    Claim {
        id: "S1-setlayer-6",
        status: ClaimStatus::Confirmed,
        summary: "SwapSetLayer is 6-arg with two CGRects",
    },
    Claim {
        id: "S1-default-surface",
        status: ClaimStatus::Confirmed,
        summary: "GetLayerDefaultSurface layer 0 is SpringBoard CA",
    },
    Claim {
        id: "S1-power-save-0",
        status: ClaimStatus::Confirmed,
        summary: "EnableDisableVideoPowerSavings(0) disables savings",
    },
    Claim {
        id: "S1-power-change-1",
        status: ClaimStatus::Confirmed,
        summary: "RequestPowerChange(1) means on",
    },
    Claim {
        id: "S1-hold",
        status: ClaimStatus::Refuted,
        summary: "Exclusive disable-others IOMFB export exists",
    },
    Claim {
        id: "S1-layers-3",
        status: ClaimStatus::Refuted,
        summary: "Userspace layer count is 3",
    },
    Claim {
        id: "S2-setlayer-3",
        status: ClaimStatus::Refuted,
        summary: "Legacy SwapSetLayer is 3-arg",
    },
    Claim {
        id: "S3-layers-4",
        status: ClaimStatus::Confirmed,
        summary: "Wiki NUM_LAYERS is 4 since iOS 7",
    },
    Claim {
        id: "S4-swapend-sel-5",
        status: ClaimStatus::Confirmed,
        summary: "_kern_SwapEnd is userclient method 5",
    },
    Claim {
        id: "S4-cancel-0x34",
        status: ClaimStatus::Confirmed,
        summary: "SwapCancel selector is 0x34",
    },
    Claim {
        id: "S5-trampoline",
        status: ClaimStatus::Confirmed,
        summary: "Public Swap* are cbz/ldr/braaz trampolines",
    },
    Claim {
        id: "S2-color",
        status: ClaimStatus::Confirmed,
        summary: "Gamma / remap / white-on-black / matrix still exist",
    },
    Claim {
        id: "S3-sel-17",
        status: ClaimStatus::Confirmed,
        summary: "SetGammaTable is selector 0x11",
    },
    Claim {
        id: "S5-virt",
        status: ClaimStatus::Confirmed,
        summary: "_virt_* is in-process, no IOConnect",
    },
    Claim {
        id: "S4-sel-0x14",
        status: ClaimStatus::Confirmed,
        summary: "Public SwapSignal is a stub; kern sel 0x14 exists",
    },
];

pub fn claim(id: &str) -> Option<&'static Claim> {
    CLAIMS.iter().find(|c| c.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settled_s6_rows_are_not_unconfirmed() {
        for id in [
            "S1-setlayer-6",
            "S2-setlayer-3",
            "S1-layers-3",
            "S3-layers-4",
            "S1-swap-end-1arg",
            "S4-cancel-0x34",
            "S2-color",
            "S3-sel-17",
            "S5-virt",
        ] {
            let c = claim(id).expect(id);
            assert_ne!(c.status, ClaimStatus::Unconfirmed, "{id}");
        }
    }

    #[test]
    fn cancel_lead_is_52_decimal() {
        assert_eq!(lead_selector::SWAP_CANCEL, 52);
    }
}
