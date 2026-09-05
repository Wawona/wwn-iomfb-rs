//! HDCP / factory / calibration / KernelTests.
//!
//! Confirmed on guest 26.1. These are not the Desktop present path.
//! Callers must be `unsafe`: factory portals can talk NVMe / HDCP.

use crate::{export_ptr, map_return, Display, Error, Result};
use iomfb_abi::{DisplayRef, IomfbReturn};

/// `IOMobileFramebufferCalibrationBegin`. Main-only. Selector stuffed as 0x14.
pub unsafe fn calibration_begin(display: &Display) -> Result<()> {
    if display.is_userland() {
        return Err(Error::Absent);
    }
    let f: unsafe extern "C" fn(DisplayRef) -> IomfbReturn =
        core::mem::transmute(export_ptr("IOMobileFramebufferCalibrationBegin")?);
    map_return(f(DisplayRef(display.as_raw())))
}

/// `IOMobileFramebufferKernelTests`. Selector `0x38`, struct `0x9c`.
/// `args` must be the guest `IOMFBKernelTestsArguments` layout.
pub unsafe fn kernel_tests(display: &Display, args: *mut core::ffi::c_void) -> Result<()> {
    if display.is_userland() {
        return Err(Error::Absent);
    }
    if args.is_null() {
        return Err(Error::NullSurface);
    }
    let f: unsafe extern "C" fn(DisplayRef, *mut core::ffi::c_void) -> IomfbReturn =
        core::mem::transmute(export_ptr("IOMobileFramebufferKernelTests")?);
    map_return(f(DisplayRef(display.as_raw()), args))
}

/// `IOMobileFramebufferFactoryPortal`. Selector `0x4b`.
pub unsafe fn factory_portal(display: &Display, arg: *mut core::ffi::c_void) -> Result<()> {
    if display.is_userland() {
        return Err(Error::Absent);
    }
    let f: unsafe extern "C" fn(DisplayRef, *mut core::ffi::c_void) -> IomfbReturn =
        core::mem::transmute(export_ptr("IOMobileFramebufferFactoryPortal")?);
    map_return(f(DisplayRef(display.as_raw()), arg))
}

/// `IOMobileFramebufferHDCPSendRequest`. Selector `0x2f`.
pub unsafe fn hdcp_send_request(
    display: &Display,
    req: *mut core::ffi::c_void,
    req_len: usize,
    reply: *mut core::ffi::c_void,
    reply_len: usize,
) -> Result<()> {
    if display.is_userland() {
        return Err(Error::Absent);
    }
    let f: unsafe extern "C" fn(
        DisplayRef,
        *mut core::ffi::c_void,
        usize,
        *mut core::ffi::c_void,
        usize,
    ) -> IomfbReturn = core::mem::transmute(export_ptr("IOMobileFramebufferHDCPSendRequest")?);
    map_return(f(
        DisplayRef(display.as_raw()),
        req,
        req_len,
        reply,
        reply_len,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_needs_framework_off_device() {
        assert!(matches!(
            Display::main(),
            Err(Error::LoadFailed) | Err(Error::MissingSymbol) | Err(Error::Absent)
        ));
    }

    #[test]
    fn factory_absent_on_userland() {
        let d = Display::userland(64, 64).unwrap();
        unsafe {
            assert!(matches!(calibration_begin(&d), Err(Error::Absent)));
            assert!(matches!(factory_portal(&d, core::ptr::null_mut()), Err(Error::Absent)));
            assert!(matches!(
                kernel_tests(&d, 0x1 as *mut core::ffi::c_void),
                Err(Error::Absent)
            ));
            assert!(matches!(
                hdcp_send_request(&d, core::ptr::null_mut(), 0, core::ptr::null_mut(), 0),
                Err(Error::Absent)
            ));
        }
    }
}
