//! `dlopen` / `dlsym` table. No Apple headers.
//!
//! On non-Apple hosts this crate still compiles. `load()` returns
//! `None`. Symbol types exist so later confirmed families can bind
//! without guessing arity in the safe crate.

use iomfb_abi::{DisplayRef, DisplaySize, IomfbReturn, FRAMEWORK_PATH};
#[cfg(all(feature = "apple-iomfb", target_vendor = "apple"))]
use iomfb_abi::PUBLIC_EXPORTS;

pub type GetDisplayFn = unsafe extern "C" fn(*mut DisplayRef) -> IomfbReturn;
pub type GetDisplaySizeFn = unsafe extern "C" fn(DisplayRef, *mut DisplaySize) -> IomfbReturn;
pub type SwapBeginFn = unsafe extern "C" fn(DisplayRef, *mut i32) -> IomfbReturn;
pub type SwapEndFn = unsafe extern "C" fn(DisplayRef) -> IomfbReturn;
pub type SwapWaitFn = unsafe extern "C" fn(DisplayRef, i32, i32) -> IomfbReturn;
pub type SwapCancelFn = unsafe extern "C" fn(DisplayRef, i32) -> IomfbReturn;
pub type GetLayerDefaultSurfaceFn =
    unsafe extern "C" fn(DisplayRef, i32, *mut *mut core::ffi::c_void) -> IomfbReturn;
pub type PowerIntFn = unsafe extern "C" fn(DisplayRef, i32) -> IomfbReturn;
pub type GetU32Fn = unsafe extern "C" fn(DisplayRef, *mut u32) -> IomfbReturn;
pub type SwapCancelAllFn = unsafe extern "C" fn(DisplayRef) -> IomfbReturn;
pub type NotifyEnableFn =
    unsafe extern "C" fn(DisplayRef, *mut core::ffi::c_void, *mut core::ffi::c_void) -> IomfbReturn;
pub type NotifyDisableFn = unsafe extern "C" fn(DisplayRef) -> IomfbReturn;

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
    pub get_id: Option<GetU32Fn>,
    pub is_main_display: Option<GetU32Fn>,
    pub swap_cancel_all: Option<SwapCancelAllFn>,
    pub swap_get_current: Option<GetU32Fn>,
    pub enable_vsync_notifications: Option<NotifyEnableFn>,
    pub disable_vsync_notifications: Option<NotifyDisableFn>,
    pub set_white_on_black: Option<PowerIntFn>,
    pub set_color_remap_mode: Option<PowerIntFn>,
    pub get_color_remap_mode: Option<GetU32Fn>,
    pub set_gamma_table: Option<unsafe extern "C" fn(DisplayRef, *const core::ffi::c_void) -> IomfbReturn>,
    pub get_gamma_table: Option<unsafe extern "C" fn(DisplayRef, *mut core::ffi::c_void) -> IomfbReturn>,
    pub set_brightness_correction: Option<PowerIntFn>,
    pub ready_for_swap: Option<SwapCancelAllFn>,
    pub wait_surface: Option<SwapCancelAllFn>,
    pub set_droppable: Option<PowerIntFn>,
    pub swap_cancel_all_get_current: Option<GetU32Fn>,
    /// Parallel to `PUBLIC_EXPORTS`. Confirmed names only.
    pub raw: Vec<Option<*mut core::ffi::c_void>>,
    /// Every public `IOMobileFramebuffer*` name that `dlsym` resolved.
    pub bound: usize,
}

unsafe impl Send for Symbols {}
unsafe impl Sync for Symbols {}

impl Symbols {
    pub fn empty() -> Self {
        Self::default()
    }

    /// Resolved pointer for a public export name, if `dlsym` found it.
    pub fn raw_named(&self, name: &str) -> Option<*mut core::ffi::c_void> {
        let i = iomfb_abi::PUBLIC_EXPORTS.iter().position(|&n| n == name)?;
        self.raw.get(i).copied().flatten()
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
        s.get_id = dlsym_fn(handle, "IOMobileFramebufferGetID");
        s.is_main_display = dlsym_fn(handle, "IOMobileFramebufferIsMainDisplay");
        s.swap_cancel_all = dlsym_fn(handle, "IOMobileFramebufferSwapCancelAll");
        s.swap_get_current = dlsym_fn(handle, "IOMobileFramebufferSwapGetCurrent");
        s.enable_vsync_notifications =
            dlsym_fn(handle, "IOMobileFramebufferEnableVSyncNotifications");
        s.disable_vsync_notifications =
            dlsym_fn(handle, "IOMobileFramebufferDisableVSyncNotifications");
        s.set_white_on_black = dlsym_fn(handle, "IOMobileFramebufferSetWhiteOnBlackMode");
        s.set_color_remap_mode = dlsym_fn(handle, "IOMobileFramebufferSetColorRemapMode");
        s.get_color_remap_mode = dlsym_fn(handle, "IOMobileFramebufferGetColorRemapMode");
        s.set_gamma_table = dlsym_fn(handle, "IOMobileFramebufferSetGammaTable");
        s.get_gamma_table = dlsym_fn(handle, "IOMobileFramebufferGetGammaTable");
        s.set_brightness_correction =
            dlsym_fn(handle, "IOMobileFramebufferSetBrightnessCorrection");
        s.ready_for_swap = dlsym_fn(handle, "IOMobileFramebufferReadyForSwap");
        s.wait_surface = dlsym_fn(handle, "IOMobileFramebufferWaitSurface");
        s.set_droppable = dlsym_fn(handle, "IOMobileFramebufferSetDroppable");
        s.swap_cancel_all_get_current =
            dlsym_fn(handle, "IOMobileFramebufferSwapCancelAllGetCurrent");
        s.raw = PUBLIC_EXPORTS
            .iter()
            .map(|name| dlsym_raw(handle, name))
            .collect();
        s.bound = s.raw.iter().filter(|p| p.is_some()).count();
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
        assert_eq!(iomfb_abi::PUBLIC_EXPORTS.len(), 153);
    }
}
