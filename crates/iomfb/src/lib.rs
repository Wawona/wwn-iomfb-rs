//! Safe IOMFB API. Two channels:
//!
//! - **TrollStore** (default): limited userspace present. No `IOConnect`.
//! - **Jailbreak**: full reconstructed ABI (`dlopen` Apple IOMFB, 153
//!   exports, factory / HDCP / live_call). See [`channel`] and
//!   `docs/CHANNELS.md`.
//!
//! GPU present is zero-copy: Metal wraps an IOSurface.

mod call;
mod channel;
mod color;
pub mod factory;
mod gpu;
mod live;
mod present;
mod surface;
mod touch;
mod userland;

pub use call::{export_is_stub, export_ptr};
pub use channel::{
    current as current_channel, detect as detect_channel, set as set_channel, Channel,
    CHANNEL_AUTO,
};
pub use live::{live_call_name, public_export_count, public_export_name};
pub use userland::{configure as configure_userland, PresentFn};
pub use gpu::{GpuFrame, GpuSwapchain};
pub use iomfb_abi::{PixelFormat, PresentStatus, WaitOutcome, SWAPCHAIN_BUFFERS};
pub use surface::{IoSurface, MetalDevice, MetalQueue, MetalTexture};
pub use touch::{TouchEvent, TouchMap, TouchSeat, TouchSpace, TouchState};

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
    /// Confirmed export, but this helper's GPR/FPR shape does not match.
    WrongArity,
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

fn symbols_cell() -> &'static OnceLock<Option<Symbols>> {
    static TABLE: OnceLock<Option<Symbols>> = OnceLock::new();
    &TABLE
}

fn load_table() -> Result<&'static Symbols> {
    symbols_cell()
        .get_or_init(load)
        .as_ref()
        .ok_or(Error::LoadFailed)
}

/// Apple `dlsym` table. TrollStore never loads it.
pub(crate) fn table() -> Result<&'static Symbols> {
    if channel::full_re() {
        return load_table();
    }
    symbols_cell()
        .get()
        .and_then(|o| o.as_ref())
        .ok_or(Error::Absent)
}

/// Explicit jailbreak open. Still a `dlopen`.
pub(crate) fn table_force() -> Result<&'static Symbols> {
    load_table()
}

pub(crate) fn map_return(rc: i32) -> Result<()> {
    if rc == IOMFB_OK {
        Ok(())
    } else {
        Err(Error::Iomfb(rc))
    }
}

pub(crate) enum Inner {
    Userland(userland::Userland),
    Apple {
        raw: DisplayRef,
        symbols: &'static Symbols,
    },
}

/// One opened display. Product path is [`Display::userland`].
pub struct Display {
    inner: Inner,
}

impl Display {
    fn from_getter(getter: Option<iomfb_sys::GetDisplayFn>) -> Result<Self> {
        Self::from_loaded(getter, table()?)
    }

    fn from_loaded(
        getter: Option<iomfb_sys::GetDisplayFn>,
        symbols: &'static Symbols,
    ) -> Result<Self> {
        let get = getter.or(symbols.get_main_display).ok_or(Error::MissingSymbol)?;
        let mut raw = DisplayRef(core::ptr::null_mut());
        map_return(unsafe { get(&mut raw) })?;
        if raw.0.is_null() {
            return Err(Error::Iomfb(-1));
        }
        Ok(Self {
            inner: Inner::Apple { raw, symbols },
        })
    }

    pub(crate) fn userland_state(&self) -> Option<&userland::Userland> {
        match &self.inner {
            Inner::Userland(u) => Some(u),
            Inner::Apple { .. } => None,
        }
    }

    pub(crate) fn apple_parts(&self) -> Result<(DisplayRef, &'static Symbols)> {
        match self.inner {
            Inner::Apple { raw, symbols } => Ok((raw, symbols)),
            Inner::Userland(_) => Err(Error::Absent),
        }
    }

    pub fn is_userland(&self) -> bool {
        matches!(self.inner, Inner::Userland(_))
    }

    /// Channel this display is running. Apple inner is always jailbreak.
    pub fn channel(&self) -> Channel {
        if self.is_userland() {
            Channel::TrollStore
        } else {
            Channel::Jailbreak
        }
    }

    pub fn full_re(&self) -> bool {
        !self.is_userland()
    }

    /// TrollStore / userspace display. Never opens the IOMFB userclient.
    pub fn userland(width: u32, height: u32) -> Result<Self> {
        Ok(Self {
            inner: Inner::Userland(userland::Userland::new(width, height, true)?),
        })
    }

    /// Same as [`Self::userland`]. Name is the TrollStore contract.
    pub fn trollstore(width: u32, height: u32) -> Result<Self> {
        Self::userland(width, height)
    }

    /// Resolve by [`channel::current`]. TrollStore is userspace. Jailbreak
    /// is Apple `GetMainDisplay` (full RE).
    pub fn main() -> Result<Self> {
        match channel::current() {
            Channel::Jailbreak => Self::jailbreak_main(),
            Channel::TrollStore => {
                let (w, h) = userland::resolve_geometry()?;
                Self::trollstore(w, h)
            }
        }
    }

    /// Jailbreak full RE: `dlopen` Apple IOMFB and `GetMainDisplay`.
    pub fn jailbreak_main() -> Result<Self> {
        Self::from_loaded(None, table_force()?)
    }

    /// Jailbreak secondary display. Absent on TrollStore.
    pub fn secondary() -> Result<Self> {
        if !channel::full_re() {
            return Err(Error::Absent);
        }
        let symbols = table()?;
        Self::from_getter(symbols.get_secondary_display)
    }

    /// Alias for [`Self::jailbreak_main`].
    pub fn apple_main() -> Result<Self> {
        Self::jailbreak_main()
    }

    pub fn set_present(&self, f: Option<PresentFn>, ctx: *mut core::ffi::c_void) {
        if let Some(u) = self.userland_state() {
            u.set_present(f, ctx);
        }
    }

    pub fn last_surface(&self) -> *mut core::ffi::c_void {
        self.userland_state()
            .map(userland::Userland::last_surface)
            .unwrap_or(core::ptr::null_mut())
    }

    pub fn size(&self) -> Result<(u32, u32)> {
        if let Some(u) = self.userland_state() {
            return Ok(u.size());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.get_display_size.ok_or(Error::MissingSymbol)?;
        let mut size = DisplaySize::default();
        map_return(unsafe { f(raw, &mut size) })?;
        Ok((size.width as u32, size.height as u32))
    }

    pub fn swap_begin(&self) -> Result<i32> {
        if let Some(u) = self.userland_state() {
            return Ok(u.swap_begin());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.swap_begin.ok_or(Error::MissingSymbol)?;
        let mut token = 0i32;
        map_return(unsafe { f(raw, &mut token) })?;
        Ok(token)
    }

    /// Confirmed 6-arg on Apple. Userland stores layer + IOSurface.
    pub fn swap_set_layer(
        &self,
        layer: i32,
        surface: *mut core::ffi::c_void,
        src: [f64; 4],
        dst: [f64; 4],
        flags: i32,
    ) -> Result<()> {
        if let Some(u) = self.userland_state() {
            let _ = (src, dst, flags);
            return u.swap_set_layer(layer, surface);
        }
        if !(0..4).contains(&layer) {
            return Err(Error::Iomfb(-1));
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.swap_set_layer_6.ok_or(Error::MissingSymbol)?;
        map_return(unsafe {
            f(
                raw, layer, surface, src[0], src[1], src[2], src[3], dst[0], dst[1], dst[2],
                dst[3], flags,
            )
        })
    }

    pub fn swap_end(&self) -> Result<()> {
        if let Some(u) = self.userland_state() {
            return u.swap_end();
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.swap_end.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw) })
    }

    pub fn swap_wait(&self, token: i32, wait: Wait) -> Result<()> {
        if let Some(u) = self.userland_state() {
            return u.swap_wait(token, wait);
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.swap_wait.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, token, wait.as_i32()) })
    }

    pub fn swap_cancel(&self, token: i32) -> Result<()> {
        if let Some(u) = self.userland_state() {
            return u.swap_cancel(token);
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.swap_cancel.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, token) })
    }

    pub fn restore_default_surface(&self) -> Result<()> {
        if let Some(u) = self.userland_state() {
            return u.restore();
        }
        let (raw, symbols) = self.apple_parts()?;
        let get = symbols
            .get_layer_default_surface
            .ok_or(Error::MissingSymbol)?;
        let mut surface = core::ptr::null_mut();
        let _ = unsafe { get(raw, 0, &mut surface) };
        let _ = self.commit_surface(0, surface, true);
        Ok(())
    }

    pub fn request_power_on(&self) -> Result<()> {
        if self.is_userland() {
            return Ok(());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.request_power_change.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, 1) })
    }

    pub fn set_video_power_savings(&self, enabled: bool) -> Result<()> {
        if self.is_userland() {
            return Ok(());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols
            .enable_disable_video_power_savings
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, i32::from(enabled)) })
    }

    pub fn id(&self) -> Result<u32> {
        if let Some(u) = self.userland_state() {
            return Ok(u.id());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.get_id.ok_or(Error::MissingSymbol)?;
        let mut id = 0u32;
        map_return(unsafe { f(raw, &mut id) })?;
        Ok(id)
    }

    pub fn is_main(&self) -> Result<bool> {
        if let Some(u) = self.userland_state() {
            return Ok(u.is_main());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.is_main_display.ok_or(Error::MissingSymbol)?;
        let mut v = 0u32;
        map_return(unsafe { f(raw, &mut v) })?;
        Ok(v != 0)
    }

    pub fn swap_cancel_all(&self) -> Result<()> {
        if let Some(u) = self.userland_state() {
            return u.swap_cancel_all();
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.swap_cancel_all.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw) })
    }

    pub fn swap_get_current(&self) -> Result<u32> {
        if let Some(u) = self.userland_state() {
            return Ok(u.swap_get_current());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.swap_get_current.ok_or(Error::MissingSymbol)?;
        let mut token = 0u32;
        map_return(unsafe { f(raw, &mut token) })?;
        Ok(token)
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
    fn main_needs_geometry_off_device() {
        let err = Display::main().err();
        assert!(matches!(
            err,
            Some(Error::LoadFailed) | Some(Error::MissingSymbol) | Some(Error::Absent)
        ));
    }

    #[test]
    fn userland_open_needs_no_framework() {
        let d = Display::userland(1290, 2796).unwrap();
        assert!(d.is_userland());
        assert_eq!(d.size().unwrap(), (1290, 2796));
        assert_eq!(d.id().unwrap(), 1);
        assert!(d.is_main().unwrap());
        let token = d.swap_begin().unwrap();
        d.swap_set_layer(0, core::ptr::null_mut(), [0.0; 4], [0.0; 4], 0)
            .unwrap();
        d.swap_end().unwrap();
        d.swap_wait(token, Wait::UntilDisplayed).unwrap();
        assert!(d.present_iosurface(core::ptr::null_mut()).is_err());
    }

    #[test]
    fn apple_symbols_stay_unloaded_by_default() {
        assert_eq!(channel::current(), Channel::TrollStore);
        assert!(!channel::full_re());
        assert_eq!(bound_export_count(), 0);
        assert!(matches!(
            export_ptr("IOMobileFramebufferSwapEnd"),
            Err(Error::Absent)
        ));
    }

    #[test]
    fn trollstore_alias_is_userland() {
        let d = Display::trollstore(64, 64).unwrap();
        assert_eq!(d.channel(), Channel::TrollStore);
        assert!(!d.full_re());
        assert!(d.is_userland());
    }
}
