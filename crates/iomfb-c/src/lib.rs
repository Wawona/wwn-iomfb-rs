//! C ABI for tipas. Names are ours, not Apple's header.
//! Swap / display / restore are confirmed on iOS 26.1 / 23B85.

use iomfb::{
    Display, Error, GpuSwapchain, TouchMap, TouchSeat, TouchSpace, TouchState, Wait,
};

pub const IOMFB_C_OK: i32 = 0;
pub const IOMFB_C_UNCONFIRMED: i32 = -1;
pub const IOMFB_C_MISSING: i32 = -2;
pub const IOMFB_C_LOAD: i32 = -3;
pub const IOMFB_C_ABSENT: i32 = -4;
pub const IOMFB_C_SURFACE: i32 = -5;
pub const IOMFB_C_NULL_SURFACE: i32 = -6;
pub const IOMFB_C_INCOMPATIBLE: i32 = -7;

fn map_err(e: Error) -> i32 {
    match e {
        Error::Unconfirmed => IOMFB_C_UNCONFIRMED,
        Error::MissingSymbol => IOMFB_C_MISSING,
        Error::LoadFailed => IOMFB_C_LOAD,
        Error::Iomfb(rc) => rc,
        Error::Absent => IOMFB_C_ABSENT,
        Error::SurfaceCreateFailed => IOMFB_C_SURFACE,
        Error::NullSurface => IOMFB_C_NULL_SURFACE,
        Error::IncompatibleSurface => IOMFB_C_INCOMPATIBLE,
        Error::WrongArity => IOMFB_C_UNCONFIRMED,
    }
}

fn display_mut<'a>(p: *mut core::ffi::c_void) -> Option<&'a mut Display> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { &mut *(p as *mut Display) })
    }
}

fn swapchain_mut<'a>(p: *mut core::ffi::c_void) -> Option<&'a mut GpuSwapchain> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { &mut *(p as *mut GpuSwapchain) })
    }
}

fn touch_mut<'a>(p: *mut core::ffi::c_void) -> Option<&'a mut TouchSeat> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { &mut *(p as *mut TouchSeat) })
    }
}

#[repr(C)]
pub struct IomfbTouchEvent {
    pub id: i32,
    pub state: i32,
    pub x: u32,
    pub y: u32,
    pub nx: f64,
    pub ny: f64,
    pub slot: u8,
}

#[repr(C)]
pub struct IomfbPresentInfo {
    pub token: i32,
    pub wait_rc: i32,
    pub displayed: u8,
    pub zero_copy: u8,
}

fn fill_present(out: *mut IomfbPresentInfo, status: iomfb::PresentStatus) {
    if out.is_null() {
        return;
    }
    let (wait_rc, displayed) = match status.wait {
        iomfb::WaitOutcome::Displayed => (0, 1),
        iomfb::WaitOutcome::Incomplete(rc) => (rc, 0),
    };
    unsafe {
        (*out).token = status.token;
        (*out).wait_rc = wait_rc;
        (*out).displayed = displayed;
        (*out).zero_copy = u8::from(status.zero_copy);
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
pub extern "C" fn iomfb_export(name: *const core::ffi::c_char) -> *mut core::ffi::c_void {
    if name.is_null() {
        return core::ptr::null_mut();
    }
    let Ok(s) = unsafe { core::ffi::CStr::from_ptr(name) }.to_str() else {
        return core::ptr::null_mut();
    };
    iomfb::export_ptr(s).unwrap_or(core::ptr::null_mut())
}

#[no_mangle]
pub extern "C" fn iomfb_export_is_stub(name: *const core::ffi::c_char) -> i32 {
    if name.is_null() {
        return 0;
    }
    let Ok(s) = unsafe { core::ffi::CStr::from_ptr(name) }.to_str() else {
        return 0;
    };
    i32::from(iomfb::export_is_stub(s))
}

#[no_mangle]
pub extern "C" fn iomfb_call_unary(
    display: *mut core::ffi::c_void,
    name: *const core::ffi::c_char,
) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    if name.is_null() {
        return IOMFB_C_MISSING;
    }
    let Ok(s) = unsafe { core::ffi::CStr::from_ptr(name) }.to_str() else {
        return IOMFB_C_MISSING;
    };
    match unsafe { d.call_unary(s) } {
        Ok(rc) => rc,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_call_int(
    display: *mut core::ffi::c_void,
    name: *const core::ffi::c_char,
    value: i32,
) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    if name.is_null() {
        return IOMFB_C_MISSING;
    }
    let Ok(s) = unsafe { core::ffi::CStr::from_ptr(name) }.to_str() else {
        return IOMFB_C_MISSING;
    };
    match unsafe { d.call_int(s, value) } {
        Ok(rc) => rc,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_call_ptr(
    display: *mut core::ffi::c_void,
    name: *const core::ffi::c_char,
    ptr: *mut core::ffi::c_void,
) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    if name.is_null() {
        return IOMFB_C_MISSING;
    }
    let Ok(s) = unsafe { core::ffi::CStr::from_ptr(name) }.to_str() else {
        return IOMFB_C_MISSING;
    };
    match unsafe { d.call_ptr(s, ptr) } {
        Ok(rc) => rc,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_set_white_on_black(display: *mut core::ffi::c_void, on: i32) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.set_white_on_black(on != 0) {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_set_color_remap_mode(display: *mut core::ffi::c_void, mode: i32) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.set_color_remap_mode(mode) {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_get_color_remap_mode(
    display: *mut core::ffi::c_void,
    out: *mut i32,
) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.get_color_remap_mode() {
        Ok(mode) => {
            if !out.is_null() {
                unsafe { *out = mode as i32 };
            }
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_factory_calibration_begin(display: *mut core::ffi::c_void) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match unsafe { iomfb::factory::calibration_begin(d) } {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
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

#[no_mangle]
pub extern "C" fn iomfb_present_iosurface(
    display: *mut core::ffi::c_void,
    surface: *mut core::ffi::c_void,
    out: *mut IomfbPresentInfo,
) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.present_iosurface(surface) {
        Ok(status) => {
            fill_present(out, status);
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swapchain_open(out: *mut *mut core::ffi::c_void) -> i32 {
    if out.is_null() {
        return IOMFB_C_MISSING;
    }
    match GpuSwapchain::main() {
        Ok(sw) => {
            unsafe { *out = Box::into_raw(Box::new(sw)) as *mut core::ffi::c_void };
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swapchain_size(
    swapchain: *mut core::ffi::c_void,
    w: *mut u32,
    h: *mut u32,
) -> i32 {
    let Some(sw) = swapchain_mut(swapchain) else {
        return IOMFB_C_MISSING;
    };
    let (ww, hh) = sw.size();
    if !w.is_null() {
        unsafe { *w = ww };
    }
    if !h.is_null() {
        unsafe { *h = hh };
    }
    IOMFB_C_OK
}

#[no_mangle]
pub extern "C" fn iomfb_swapchain_has_metal(swapchain: *mut core::ffi::c_void) -> i32 {
    swapchain_mut(swapchain).map(|sw| i32::from(sw.has_metal())).unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn iomfb_swapchain_acquire(
    swapchain: *mut core::ffi::c_void,
    out_surface: *mut *mut core::ffi::c_void,
    out_metal: *mut *mut core::ffi::c_void,
    out_id: *mut u32,
) -> i32 {
    let Some(sw) = swapchain_mut(swapchain) else {
        return IOMFB_C_MISSING;
    };
    match sw.acquire() {
        Ok(frame) => {
            if !out_surface.is_null() {
                unsafe { *out_surface = frame.surface };
            }
            if !out_metal.is_null() {
                unsafe { *out_metal = frame.metal_texture };
            }
            if !out_id.is_null() {
                unsafe { *out_id = frame.iosurface_id };
            }
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swapchain_present(
    swapchain: *mut core::ffi::c_void,
    out: *mut IomfbPresentInfo,
) -> i32 {
    let Some(sw) = swapchain_mut(swapchain) else {
        return IOMFB_C_MISSING;
    };
    match sw.present() {
        Ok(status) => {
            fill_present(out, status);
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swapchain_present_external(
    swapchain: *mut core::ffi::c_void,
    surface: *mut core::ffi::c_void,
    out: *mut IomfbPresentInfo,
) -> i32 {
    let Some(sw) = swapchain_mut(swapchain) else {
        return IOMFB_C_MISSING;
    };
    match sw.present_external(surface) {
        Ok(status) => {
            fill_present(out, status);
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swapchain_clear(
    swapchain: *mut core::ffi::c_void,
    r: f32,
    g: f32,
    b: f32,
    a: f32,
) -> i32 {
    let Some(sw) = swapchain_mut(swapchain) else {
        return IOMFB_C_MISSING;
    };
    match sw.clear([r, g, b, a]) {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_display_id(display: *mut core::ffi::c_void, out: *mut u32) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.id() {
        Ok(id) => {
            if !out.is_null() {
                unsafe { *out = id };
            }
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_display_is_main(display: *mut core::ffi::c_void, out: *mut u32) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.is_main() {
        Ok(v) => {
            if !out.is_null() {
                unsafe { *out = u32::from(v) };
            }
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swap_cancel_all(display: *mut core::ffi::c_void) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.swap_cancel_all() {
        Ok(()) => IOMFB_C_OK,
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swap_get_current(display: *mut core::ffi::c_void, out: *mut u32) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match d.swap_get_current() {
        Ok(token) => {
            if !out.is_null() {
                unsafe { *out = token };
            }
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_swapchain_close(swapchain: *mut core::ffi::c_void) {
    if !swapchain.is_null() {
        unsafe { drop(Box::from_raw(swapchain as *mut GpuSwapchain)) };
    }
}

#[no_mangle]
pub extern "C" fn iomfb_touch_map(
    display: *mut core::ffi::c_void,
    nx: f64,
    ny: f64,
    out_x: *mut u32,
    out_y: *mut u32,
) -> i32 {
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match TouchMap::from_display(d) {
        Ok(map) => {
            let (x, y) = map.pixel(nx, ny);
            if !out_x.is_null() {
                unsafe { *out_x = x };
            }
            if !out_y.is_null() {
                unsafe { *out_y = y };
            }
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_touch_open(
    display: *mut core::ffi::c_void,
    out: *mut *mut core::ffi::c_void,
) -> i32 {
    if out.is_null() {
        return IOMFB_C_MISSING;
    }
    let Some(d) = display_mut(display) else {
        return IOMFB_C_MISSING;
    };
    match TouchSeat::from_display(d) {
        Ok(seat) => {
            unsafe { *out = Box::into_raw(Box::new(seat)) as *mut core::ffi::c_void };
            IOMFB_C_OK
        }
        Err(e) => map_err(e),
    }
}

#[no_mangle]
pub extern "C" fn iomfb_touch_open_swapchain(
    swapchain: *mut core::ffi::c_void,
    out: *mut *mut core::ffi::c_void,
) -> i32 {
    if out.is_null() {
        return IOMFB_C_MISSING;
    }
    let Some(sw) = swapchain_mut(swapchain) else {
        return IOMFB_C_MISSING;
    };
    unsafe { *out = Box::into_raw(Box::new(sw.touch_seat())) as *mut core::ffi::c_void };
    IOMFB_C_OK
}

#[no_mangle]
pub extern "C" fn iomfb_touch_set_view(
    touch: *mut core::ffi::c_void,
    view_w: f64,
    view_h: f64,
) -> i32 {
    let Some(seat) = touch_mut(touch) else {
        return IOMFB_C_MISSING;
    };
    seat.map_mut().set_view(view_w, view_h);
    IOMFB_C_OK
}

#[no_mangle]
pub extern "C" fn iomfb_touch_set_dest(
    touch: *mut core::ffi::c_void,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
) -> i32 {
    let Some(seat) = touch_mut(touch) else {
        return IOMFB_C_MISSING;
    };
    seat.map_mut().set_dest(x, y, w, h);
    IOMFB_C_OK
}

#[no_mangle]
pub extern "C" fn iomfb_touch_set_rotation(touch: *mut core::ffi::c_void, degrees: i32) -> i32 {
    let Some(seat) = touch_mut(touch) else {
        return IOMFB_C_MISSING;
    };
    seat.map_mut().set_rotation(degrees);
    IOMFB_C_OK
}

#[no_mangle]
pub extern "C" fn iomfb_touch_inject(
    touch: *mut core::ffi::c_void,
    id: i32,
    state: i32,
    x: f64,
    y: f64,
    space: i32,
    out: *mut IomfbTouchEvent,
) -> i32 {
    let Some(seat) = touch_mut(touch) else {
        return IOMFB_C_MISSING;
    };
    let Some(st) = TouchState::from_i32(state) else {
        return IOMFB_C_INCOMPATIBLE;
    };
    let Some(sp) = TouchSpace::from_i32(space) else {
        return IOMFB_C_INCOMPATIBLE;
    };
    match seat.inject(id, st, x, y, sp) {
        Some(ev) => {
            if !out.is_null() {
                unsafe {
                    *out = IomfbTouchEvent {
                        id: ev.id,
                        state: ev.state.as_i32(),
                        x: ev.x,
                        y: ev.y,
                        nx: ev.nx,
                        ny: ev.ny,
                        slot: ev.slot,
                    };
                }
            }
            IOMFB_C_OK
        }
        None => IOMFB_C_ABSENT,
    }
}

#[no_mangle]
pub extern "C" fn iomfb_touch_active_count(touch: *mut core::ffi::c_void) -> i32 {
    touch_mut(touch)
        .map(|s| s.active_count() as i32)
        .unwrap_or(IOMFB_C_MISSING)
}

#[no_mangle]
pub extern "C" fn iomfb_touch_cancel_all(touch: *mut core::ffi::c_void) -> i32 {
    let Some(seat) = touch_mut(touch) else {
        return IOMFB_C_MISSING;
    };
    seat.cancel_all().len() as i32
}

#[no_mangle]
pub extern "C" fn iomfb_touch_close(touch: *mut core::ffi::c_void) {
    if !touch.is_null() {
        unsafe { drop(Box::from_raw(touch as *mut TouchSeat)) };
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

    #[test]
    fn touch_open_needs_display() {
        let rc = iomfb_touch_open(core::ptr::null_mut(), core::ptr::null_mut());
        assert_eq!(rc, IOMFB_C_MISSING);
    }
}
