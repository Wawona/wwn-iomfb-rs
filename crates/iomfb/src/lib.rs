//! Safe IOMFB API. Swap / display / default-surface are confirmed on
//! guest-class iOS 26.1 / 23B85 (`docs/ABI.md`). Other families still
//! return [`Error::Unconfirmed`].
//!
//! GPU present is zero-copy: Metal wraps an IOSurface, IOMFB swaps
//! that same surface. See [`GpuSwapchain`] and `docs/GPU.md`.

mod gpu;
mod present;
mod surface;
mod touch;

pub use gpu::{GpuFrame, GpuSwapchain};
pub use iomfb_abi::{PixelFormat, PresentStatus, WaitOutcome, SWAPCHAIN_BUFFERS};
pub use surface::{IoSurface, MetalDevice, MetalTexture};
pub use touch::TouchMap;

use iomfb_abi::{DisplayRef, DisplaySize, IOMFB_OK};
use iomfb_sys::{load, Symbols};
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    /// `docs/CLAIMS.md` row is still unconfirmed. Do not guess.
    Unconfirmed,
    /// Symbol missing from the loaded table.
    MissingSymbol,
    /// Framework `dlopen` failed.
    LoadFailed,
    /// IOMFB returned a non-zero code.
    Iomfb(i32),
    /// Guest image has no such export.
    Absent,
    /// IOSurfaceCreate failed or Metal/IOSurface glue is missing.
    SurfaceCreateFailed,
    /// Null IOSurface passed to a present path.
    NullSurface,
    /// Size or fourcc does not match the opened display.
    IncompatibleSurface,
}

pub type Result<T> = core::result::Result<T, Error>;

/// `SwapWait` options. Confirmed: `0` is the until-displayed lead Wawona uses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wait {
    UntilDisplayed,
    Options(i32),
}

impl Wait {
    fn as_i32(self) -> i32 {
        match self {
            Self::UntilDisplayed => iomfb_abi::WAIT_UNTIL_DISPLAYED_LEAD,
            Self::Options(v) => v,
        }
    }
}

fn table() -> Result<&'static Symbols> {
    static TABLE: OnceLock<Option<Symbols>> = OnceLock::new();
    TABLE
        .get_or_init(load)
        .as_ref()
        .ok_or(Error::LoadFailed)
}

fn map_return(rc: i32) -> Result<()> {
    if rc == IOMFB_OK {
        Ok(())
    } else {
        Err(Error::Iomfb(rc))
    }
}

/// One opened display. Confirmed family: main / secondary / size / swap / restore.
pub struct Display {
    raw: DisplayRef,
    symbols: &'static Symbols,
}

impl Display {
    fn from_getter(getter: Option<iomfb_sys::GetDisplayFn>) -> Result<Self> {
        let symbols = table()?;
        let get = getter.or(symbols.get_main_display).ok_or(Error::MissingSymbol)?;
        let mut raw = DisplayRef(core::ptr::null_mut());
        map_return(unsafe { get(&mut raw) })?;
        if raw.0.is_null() {
            return Err(Error::Iomfb(-1));
        }
        Ok(Self { raw, symbols })
    }

    /// `IOMobileFramebufferGetMainDisplay`. Confirmed.
    pub fn main() -> Result<Self> {
        let symbols = table()?;
        Self::from_getter(symbols.get_main_display)
    }

    /// `IOMobileFramebufferGetSecondaryDisplay`. Confirmed present.
    pub fn secondary() -> Result<Self> {
        let symbols = table()?;
        Self::from_getter(symbols.get_secondary_display)
    }

    /// Userspace writes `CGSize` (two doubles). Kernel selector 8 is `{u32,u32}`.
    pub fn size(&self) -> Result<(u32, u32)> {
        let f = self.symbols.get_display_size.ok_or(Error::MissingSymbol)?;
        let mut size = DisplaySize::default();
        map_return(unsafe { f(self.raw, &mut size) })?;
        Ok((size.width as u32, size.height as u32))
    }

    pub fn swap_begin(&self) -> Result<i32> {
        let f = self.symbols.swap_begin.ok_or(Error::MissingSymbol)?;
        let mut token = 0i32;
        map_return(unsafe { f(self.raw, &mut token) })?;
        Ok(token)
    }

    /// Confirmed 6-arg: layer `0..3`, two `CGRect`s by value (8 doubles), flags.
    pub fn swap_set_layer(
        &self,
        layer: i32,
        surface: *mut core::ffi::c_void,
        src: [f64; 4],
        dst: [f64; 4],
        flags: i32,
    ) -> Result<()> {
        if !(0..4).contains(&layer) {
            return Err(Error::Iomfb(-1));
        }
        let f = self.symbols.swap_set_layer_6.ok_or(Error::MissingSymbol)?;
        map_return(unsafe {
            f(
                self.raw, layer, surface, src[0], src[1], src[2], src[3], dst[0], dst[1], dst[2],
                dst[3], flags,
            )
        })
    }

    /// Public wrapper is 1-arg. Confirmed.
    pub fn swap_end(&self) -> Result<()> {
        let f = self.symbols.swap_end.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw) })
    }

    pub fn swap_wait(&self, token: i32, wait: Wait) -> Result<()> {
        let f = self.symbols.swap_wait.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, token, wait.as_i32()) })
    }

    /// Confirmed: selector `0x34`. One token. No cancel-all in this method.
    pub fn swap_cancel(&self, token: i32) -> Result<()> {
        let f = self.symbols.swap_cancel.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, token) })
    }

    pub fn restore_default_surface(&self) -> Result<()> {
        let get = self
            .symbols
            .get_layer_default_surface
            .ok_or(Error::MissingSymbol)?;
        let mut surface = core::ptr::null_mut();
        let _ = unsafe { get(self.raw, 0, &mut surface) };
        // Default surface is SpringBoard CA. Restore-only. Never a render target.
        let _ = self.commit_surface(0, surface, true);
        Ok(())
    }

    pub fn request_power_on(&self) -> Result<()> {
        let f = self
            .symbols
            .request_power_change
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, 1) })
    }

    /// Polarity of `enabled` is still unconfirmed. Bind only.
    pub fn set_video_power_savings(&self, enabled: bool) -> Result<()> {
        let f = self
            .symbols
            .enable_disable_video_power_savings
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, i32::from(enabled)) })
    }
}

/// Host load. `None` off-device or without `apple-iomfb`.
pub fn symbols() -> Option<&'static Symbols> {
    table().ok()
}

/// How many of the 153 public exports `dlsym` found. `0` off-device.
pub fn bound_export_count() -> usize {
    table().map(|s| s.bound).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_needs_framework_off_device() {
        let err = Display::main().err();
        assert!(matches!(err, Some(Error::LoadFailed) | Some(Error::MissingSymbol)));
    }
}
