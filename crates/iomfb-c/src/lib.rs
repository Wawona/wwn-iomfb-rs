//! C ABI for tipas. Names are ours, not Apple's header.
//! Swap / display / restore are confirmed on iOS 26.1 / 23B85.

use iomfb::{Display, Error, Wait};

pub const IOMFB_C_OK: i32 = 0;
pub const IOMFB_C_UNCONFIRMED: i32 = -1;
pub const IOMFB_C_MISSING: i32 = -2;
pub const IOMFB_C_LOAD: i32 = -3;
pub const IOMFB_C_ABSENT: i32 = -4;

fn map_err(e: Error) -> i32 {
    match e {
        Error::Unconfirmed => IOMFB_C_UNCONFIRMED,
        Error::MissingSymbol => IOMFB_C_MISSING,
        Error::LoadFailed => IOMFB_C_LOAD,
        Error::Iomfb(rc) => rc,
        Error::Absent => IOMFB_C_ABSENT,
    }
}

fn display_mut(p: *mut core::ffi::c_void) -> Option<&'static mut Display> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { &mut *(p as *mut Display) })
    }
}

#[no_mangle]
pub extern "C" fn iomfb_display_open_main(out: *mut *mut core::ffi::c_void) -> i32 {
    if out.is_null() {
        return IOMFB_C_MISSING;
    }
    match Display::main() {
        Ok(d) => {
            unsafe { *out = Box::into_raw(Box::new(d)) as *mut core::ffi::c_void };
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_display_size(
    display: *mut core::ffi::c_void,
    w: *mut u32,
    h: *mut u32,
) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.size() {
        Ok((ww, hh)) => {
            if !w.is_null() {
                unsafe { *w = ww };
            }
            if !h.is_null() {
                unsafe { *h = hh };
            }
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swap_begin(display: *mut core::ffi::c_void, token: *mut i32) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.swap_begin() {
        Ok(t) => {
            if !token.is_null() {
                unsafe { *token = t };
            }
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swap_set_layer(
    display: *mut core::ffi::c_void,
    layer: i32,
    surface: *mut core::ffi::c_void,
    sx: f64,
    sy: f64,
    sw: f64,
    sh: f64,
    dx: f64,
    dy: f64,
    dw: f64,
    dh: f64,
    flags: i32,
) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.swap_set_layer(layer, surface, [sx, sy, sw, sh], [dx, dy, dw, dh], flags) {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swap_end(display: *mut core::ffi::c_void) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.swap_end() {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swap_wait(
    display: *mut core::ffi::c_void,
    token: i32,
    options: i32,
) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.swap_wait(token, Wait::Options(options)) {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swap_cancel(display: *mut core::ffi::c_void, token: i32) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.swap_cancel(token) {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_bound_export_count() -> u32 {
    iomfb::bound_export_count() as u32
}

#[no_mangle]
pub extern "C" fn iomfb_restore_default_surface(display: *mut core::ffi::c_void) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.restore_default_surface() {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_display_close(display: *mut core::ffi::c_void) {
    if !display.is_null() {
        unsafe { drop(Box::from_raw(display as *mut Display)) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_fails_closed_off_device() {
        let rc = iomfb_display_open_main(core::ptr::null_mut());
        assert_eq!(rc, IOMFB_C_MISSING);
    }
}
