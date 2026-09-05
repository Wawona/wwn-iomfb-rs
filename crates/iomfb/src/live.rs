//! Call every confirmed public export at the census arity.
//!
//! Extra GPRs are a scratch buffer so out-params do not take a null
//! deref. First-arg exceptions match `docs/ABI.md`. Stubs return
//! [`Error::Absent`] without a call. Factory / HDCP / KernelTests stay
//! fail-closed (zeroed structs, null vtables).

use crate::{export_ptr, table, Display, Error, Result};
use iomfb_abi::{
    export_meta, DisplayRef, GAMMA_TABLE_SIZE, KERNEL_TESTS_SIZE, PUBLIC_EXPORTS,
};

/// ARM64 AAPCS: unused extra args sit in leftover registers.
type LiveFn = unsafe extern "C" fn(
    *mut core::ffi::c_void,
    *mut core::ffi::c_void,
    *mut core::ffi::c_void,
    *mut core::ffi::c_void,
    *mut core::ffi::c_void,
    *mut core::ffi::c_void,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
    f64,
) -> isize;

/// Confirmed 6-arg SetLayer (3 GPR + 8 FPR + flags).
type SetLayerFn = unsafe extern "C" fn(
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
) -> i32;

pub fn public_export_count() -> usize {
    PUBLIC_EXPORTS.len()
}

pub fn public_export_name(i: usize) -> Option<&'static str> {
    PUBLIC_EXPORTS.get(i).copied()
}

/// Recorded x0 for one public name. `display` may be `None` only for
/// stubs (tested off-device).
pub unsafe fn live_call_name(display: Option<&Display>, name: &str) -> Result<i32> {
    let meta = export_meta(name).ok_or(Error::Absent)?;
    if meta.stub {
        return Err(Error::Absent);
    }
    if display.map(Display::is_userland).unwrap_or(false) {
        return Err(Error::Absent);
    }
    let p = export_ptr(name)?;
    let fb = match display {
        Some(d) => d.as_raw(),
        None => return Err(Error::LoadFailed),
    };
    Ok(invoke(name, p, fb))
}

impl Display {
    /// Invoke `name` at the documented GPR/FPR count. Returns the raw
    /// code (Apple IOReturn, pointer fragment, or CFTypeID).
    pub unsafe fn live_call(&self, name: &str) -> Result<i32> {
        live_call_name(Some(self), name)
    }

    /// `IOMobileFramebufferGetTypeID`. No framebuffer argument.
    pub fn type_id() -> Result<usize> {
        let p = export_ptr("IOMobileFramebufferGetTypeID")?;
        let f: unsafe extern "C" fn() -> usize = unsafe { core::mem::transmute(p) };
        Ok(unsafe { f() })
    }

    /// `IOMobileFramebufferGetServiceObject`. Returns the IOService.
    pub fn service_object(&self) -> Result<*mut core::ffi::c_void> {
        let p = export_ptr("IOMobileFramebufferGetServiceObject")?;
        let f: unsafe extern "C" fn(DisplayRef) -> *mut core::ffi::c_void =
            unsafe { core::mem::transmute(p) };
        Ok(unsafe { f(DisplayRef(self.as_raw())) })
    }

    /// `IOMobileFramebufferOpenByName`. `cf_name` is a CFStringRef.
    pub unsafe fn open_by_name_cf(cf_name: *mut core::ffi::c_void) -> Result<Self> {
        let symbols = table()?;
        let p = export_ptr("IOMobileFramebufferOpenByName")?;
        let f: unsafe extern "C" fn(
            *mut core::ffi::c_void,
            *mut DisplayRef,
        ) -> i32 = core::mem::transmute(p);
        let mut raw = DisplayRef(core::ptr::null_mut());
        crate::map_return(f(cf_name, &mut raw))?;
        if raw.0.is_null() {
            return Err(Error::Iomfb(-1));
        }
        Ok(Self {
            inner: crate::Inner::Apple { raw, symbols },
        })
    }
}

unsafe fn invoke(name: &str, p: *mut core::ffi::c_void, fb: *mut core::ffi::c_void) -> i32 {
    let mut scratch = [0u8; 4096];
    let s = scratch.as_mut_ptr().cast();
    let z = core::ptr::null_mut();
    let mut kernel_args = [0u8; KERNEL_TESTS_SIZE];
    let mut gamma = [0u8; GAMMA_TABLE_SIZE];
    let mut out_fb = core::ptr::null_mut::<core::ffi::c_void>();

    let (a0, a1, a2, a3, a4, a5) = match name {
        "IOMobileFramebufferGetTypeID" => (z, z, z, z, z, z),
        "IOMobileFramebufferGetMainDisplay" | "IOMobileFramebufferGetSecondaryDisplay" => {
            ((&mut out_fb as *mut *mut core::ffi::c_void).cast(), z, z, z, z, z)
        }
        "IOMobileFramebufferGetFrameworkInfo" => (s, z, z, z, z, z),
        "IOMobileFramebufferCreateDisplayList" => (z, z, z, z, z, z),
        "IOMobileFramebufferOpenByName" => {
            (z, (&mut out_fb as *mut *mut core::ffi::c_void).cast(), z, z, z, z)
        }
        "IOMobileFramebufferOpen" => {
            let a = open_args(fb, &mut out_fb);
            (a[0], a[1], a[2], a[3], a[4], a[5])
        }
        "IOMobileFramebufferInstallVirtualDisplays" => (z, z, z, z, z, z),
        "IOMobileFramebufferInstallVirtualDisplay" => (z, z, z, z, z, z),
        "IOMobileFramebufferKernelTests" => {
            (fb, kernel_args.as_mut_ptr().cast(), z, z, z, z)
        }
        "IOMobileFramebufferGetGammaTable"
        | "IOMobileFramebufferSetGammaTable"
        | "IOMobileFramebufferSwapSetGammaTable" => {
            (fb, gamma.as_mut_ptr().cast(), s, s, s, s)
        }
        "IOMobileFramebufferHDCPSendRequest" => (fb, z, z, z, z, z),
        "IOMobileFramebufferCopyProperty" => (fb, z, z, z, z, z),
        "IOMobileFramebufferGetBlock"
        | "IOMobileFramebufferGetBufBlock"
        | "IOMobileFramebufferSetBlock"
        | "IOMobileFramebufferSetIdleBuffer"
        | "IOMobileFramebufferSetIdleBufferEvent"
        | "IOMobileFramebufferSwapSetEventSignal"
        | "IOMobileFramebufferSwapSetEventSignalOnGlass"
        | "IOMobileFramebufferSwapSetEventWait" => (fb, z, z, z, z, z),
        "IOMobileFramebufferSwapSetLayer" => {
            return set_layer_zeros(p, fb);
        }
        "IOMobileFramebufferReadyForSwap" => (fb, z, z, z, z, z),
        "IOMobileFramebufferSwapWait" | "IOMobileFramebufferSwapWaitWithTimeout" => {
            (fb, z, z, z, z, z)
        }
        "IOMobileFramebufferSwapCancel" => (fb, z, z, z, z, z),
        "IOMobileFramebufferEnableCRCNotifications"
        | "IOMobileFramebufferEnableHotPlugDetectNotifications"
        | "IOMobileFramebufferEnableNeedSwapNotifications"
        | "IOMobileFramebufferEnablePowerNotifications"
        | "IOMobileFramebufferEnableVSyncNotifications"
        | "IOMobileFramebufferSetBrightnessControlCallback"
        | "IOMobileFramebufferScheduleWithDispatchQueue"
        | "IOMobileFramebufferUnscheduleFromDispatchQueue" => (fb, z, z, z, z, z),
        _ => (fb, s, s, s, s, s),
    };

    let f: LiveFn = core::mem::transmute(p);
    f(a0, a1, a2, a3, a4, a5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0) as i32
}

fn open_args(
    fb: *mut core::ffi::c_void,
    out_fb: &mut *mut core::ffi::c_void,
) -> [*mut core::ffi::c_void; 6] {
    let service = match export_ptr("IOMobileFramebufferGetServiceObject") {
        Ok(p) => unsafe {
            let f: unsafe extern "C" fn(*mut core::ffi::c_void) -> *mut core::ffi::c_void =
                core::mem::transmute(p);
            f(fb)
        },
        Err(_) => core::ptr::null_mut(),
    };
    [
        service,
        core::ptr::null_mut(),
        core::ptr::null_mut(),
        (out_fb as *mut *mut core::ffi::c_void).cast(),
        core::ptr::null_mut(),
        core::ptr::null_mut(),
    ]
}

fn set_layer_zeros(p: *mut core::ffi::c_void, fb: *mut core::ffi::c_void) -> i32 {
    let f: SetLayerFn = unsafe { core::mem::transmute(p) };
    unsafe {
        f(
            DisplayRef(fb),
            0,
            core::ptr::null_mut(),
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stubs_are_absent_without_a_display() {
        assert!(matches!(
            unsafe { live_call_name(None, "IOMobileFramebufferSwapSignal") },
            Err(Error::Absent)
        ));
        assert!(matches!(
            unsafe { live_call_name(None, "IOMobileFramebufferSwapSetUISubRegion") },
            Err(Error::Absent)
        ));
    }

    #[test]
    fn unknown_name_is_absent() {
        assert!(matches!(
            unsafe { live_call_name(None, "IOMobileFramebufferDoesNotExist") },
            Err(Error::Absent)
        ));
    }

    #[test]
    fn live_needs_a_display_off_device() {
        assert!(matches!(
            unsafe { live_call_name(None, "IOMobileFramebufferSwapEnd") },
            Err(Error::LoadFailed) | Err(Error::MissingSymbol)
        ));
    }

    #[test]
    fn census_ready_for_swap_is_three_gpr() {
        let meta = export_meta("IOMobileFramebufferReadyForSwap").unwrap();
        assert_eq!(meta.gpr, 3);
        assert!(!meta.stub);
    }

    #[test]
    fn public_export_count_is_153() {
        assert_eq!(public_export_count(), 153);
        assert_eq!(public_export_name(0), Some("IOMobileFramebufferAnnounceNextSwapTimestamp"));
        assert!(public_export_name(153).is_none());
    }
}
