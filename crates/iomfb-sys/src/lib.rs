//! `dlopen` / `dlsym` table. No Apple headers.
//!
//! On non-Apple hosts this crate still compiles. `load()` returns
//! `None`. Symbol types exist so later confirmed families can bind
//! without guessing arity in the safe crate.

use iomfb_abi::{DisplayRef, DisplaySize, IomfbReturn, FRAMEWORK_PATH, PUBLIC_EXPORTS};

pub type GetDisplayFn = unsafe extern "C" fn(*mut DisplayRef) -> IomfbReturn;
pub type GetDisplaySizeFn = unsafe extern "C" fn(DisplayRef, *mut DisplaySize) -> IomfbReturn;
pub type SwapBeginFn = unsafe extern "C" fn(DisplayRef, *mut i32) -> IomfbReturn;
pub type SwapEndFn = unsafe extern "C" fn(DisplayRef) -> IomfbReturn;
pub type SwapWaitFn = unsafe extern "C" fn(DisplayRef, i32, i32) -> IomfbReturn;
pub type SwapCancelFn = unsafe extern "C" fn(DisplayRef, i32) -> IomfbReturn;
pub type GetLayerDefaultSurfaceFn =
    unsafe extern "C" fn(DisplayRef, i32, *mut *mut core::ffi::c_void) -> IomfbReturn;
pub type PowerIntFn = unsafe extern "C" fn(DisplayRef, i32) -> IomfbReturn;

/// Confirmed 6-arg on iOS 26.1. Two `CGRect`s by value (8 doubles), then flags.
pub type SwapSetLayer6Fn = unsafe extern "C" fn(
    DisplayRef,
    i32,
    *mut core::ffi::c_void,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    i32,
) -> IomfbReturn;

/// Resolved table. Every pointer starts `None`.
#[derive(Default)]
pub struct Symbols {
    pub get_main_display: Option<GetDisplayFn>,
    pub get_secondary_display: Option<GetDisplayFn>,
    pub get_display_size: Option<GetDisplaySizeFn>,
    pub swap_begin: Option<SwapBeginFn>,
    pub swap_end: Option<SwapEndFn>,
    pub swap_wait: Option<SwapWaitFn>,
    pub swap_cancel: Option<SwapCancelFn>,
    pub swap_set_layer_6: Option<SwapSetLayer6Fn>,
    pub get_layer_default_surface: Option<GetLayerDefaultSurfaceFn>,
    pub enable_disable_video_power_savings: Option<PowerIntFn>,
    pub request_power_change: Option<PowerIntFn>,
    /// Every public `IOMobileFramebuffer*` name that `dlsym` resolved.
    pub bound: usize,
}

impl Symbols {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn has_swap_family(&self) -> bool {
        self.get_main_display.is_some()
            && self.get_display_size.is_some()
            && self.swap_begin.is_some()
            && self.swap_end.is_some()
            && self.swap_wait.is_some()
            && self.swap_set_layer_6.is_some()
    }
}

/// Load the framework. Returns `None` unless built with `apple-iomfb`
/// on an Apple target that actually has the private framework.
pub fn load() -> Option<Symbols> {
    #[cfg(all(feature = "apple-iomfb", target_vendor = "apple"))]
    {
        load_apple()
    }
    #[cfg(not(all(feature = "apple-iomfb", target_vendor = "apple")))]
    {
        let _ = FRAMEWORK_PATH;
        None
    }
}

#[cfg(all(feature = "apple-iomfb", target_vendor = "apple"))]
fn load_apple() -> Option<Symbols> {
    use std::ffi::CString;

    unsafe {
        let path = CString::new(FRAMEWORK_PATH).ok()?;
        let handle = dlopen(path.as_ptr(), 1);
        if handle.is_null() {
            return None;
        }
        let mut s = Symbols::empty();
        s.get_main_display = dlsym_fn(handle, "IOMobileFramebufferGetMainDisplay");
        s.get_secondary_display = dlsym_fn(handle, "IOMobileFramebufferGetSecondaryDisplay");
        s.get_display_size = dlsym_fn(handle, "IOMobileFramebufferGetDisplaySize");
        s.swap_begin = dlsym_fn(handle, "IOMobileFramebufferSwapBegin");
        s.swap_end = dlsym_fn(handle, "IOMobileFramebufferSwapEnd");
        s.swap_wait = dlsym_fn(handle, "IOMobileFramebufferSwapWait");
        s.swap_cancel = dlsym_fn(handle, "IOMobileFramebufferSwapCancel");
        s.swap_set_layer_6 = dlsym_fn(handle, "IOMobileFramebufferSwapSetLayer");
        s.get_layer_default_surface = dlsym_fn(handle, "IOMobileFramebufferGetLayerDefaultSurface");
        s.enable_disable_video_power_savings =
            dlsym_fn(handle, "IOMobileFramebufferEnableDisableVideoPowerSavings");
        s.request_power_change = dlsym_fn(handle, "IOMobileFramebufferRequestPowerChange");
        s.bound = PUBLIC_EXPORTS
            .iter()
            .filter(|name| dlsym_raw(handle, name).is_some())
            .count();
        Some(s)
    }
}

#[cfg(all(feature = "apple-iomfb", target_vendor = "apple"))]
unsafe fn dlsym_raw(handle: *mut core::ffi::c_void, name: &str) -> Option<*mut core::ffi::c_void> {
    use std::ffi::CString;
    let c = CString::new(name).ok()?;
    let p = dlsym(handle, c.as_ptr());
    if p.is_null() {
        None
    } else {
        Some(p)
    }
}

#[cfg(all(feature = "apple-iomfb", target_vendor = "apple"))]
unsafe fn dlsym_fn<T>(handle: *mut core::ffi::c_void, name: &str) -> Option<T> {
    dlsym_raw(handle, name).map(|p| std::mem::transmute_copy(&p))
}

#[cfg(all(feature = "apple-iomfb", target_vendor = "apple"))]
extern "C" {
    fn dlopen(path: *const i8, mode: i32) -> *mut core::ffi::c_void;
    fn dlsym(handle: *mut core::ffi::c_void, name: *const i8) -> *mut core::ffi::c_void;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_is_none_without_feature() {
        if cfg!(not(feature = "apple-iomfb")) {
            assert!(load().is_none());
        }
    }

    #[test]
    fn empty_has_no_swap_family() {
        assert!(!Symbols::empty().has_swap_family());
        assert_eq!(Symbols::empty().bound, 0);
    }

    #[test]
    fn public_export_census_is_complete() {
        assert_eq!(PUBLIC_EXPORTS.len(), 153);
    }
}
