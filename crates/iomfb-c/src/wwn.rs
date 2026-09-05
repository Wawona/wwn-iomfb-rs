//! Wawona L4 Desktop sink ABI (`wwn_iomfb_*`).
//!
//! Own-display always uses Apple `GetMainDisplay` (jailbreak channel),
//! even when the binary lives in a TrollStore container. Generic tipas
//! that must stay userspace call `iomfb_display_open_trollstore` instead.
//! Exclusive is last-surface hold. There is no disable-others export.

use crate::{map_err, IOMFB_C_MISSING, IOMFB_C_OK};
use iomfb::GpuSwapchain;
use std::cell::UnsafeCell;
use std::ffi::c_void;

const WWN_IOMFB_OK: i32 = 0;
const WWN_IOMFB_INVALID: i32 = -1;

#[repr(C)]
pub struct WwnIomfbDamage {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[repr(C)]
pub struct WwnIomfbSurface {
    pub iosurface: *mut c_void,
    pub id: u32,
    pub width: u32,
    pub height: u32,
    pub bytes_per_row: u32,
}

struct Session {
    swapchain: GpuSwapchain,
    exclusive: bool,
    last_surface: *mut c_void,
    last_error: [u8; 192],
}

struct OpenError(UnsafeCell<[u8; 192]>);
unsafe impl Sync for OpenError {}

static LAST_OPEN_ERROR: OpenError = OpenError(UnsafeCell::new([0; 192]));

fn last_open_error_mut() -> &'static mut [u8; 192] {
    // C last_error is process-global. Callers copy immediately.
    unsafe { &mut *LAST_OPEN_ERROR.0.get() }
}

fn set_bytes(buf: &mut [u8; 192], message: &str) {
    buf.fill(0);
    let bytes = message.as_bytes();
    let n = bytes.len().min(buf.len() - 1);
    buf[..n].copy_from_slice(&bytes[..n]);
}

fn session_mut<'a>(p: *mut c_void) -> Option<&'a mut Session> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { &mut *(p as *mut Session) })
    }
}

#[no_mangle]
pub extern "C" fn wwn_iomfb_open(out_session: *mut *mut c_void) -> i32 {
    if out_session.is_null() {
        set_bytes(last_open_error_mut(), "open output is null");
        return WWN_IOMFB_INVALID;
    }
    match GpuSwapchain::jailbreak_main() {
        Ok(sw) => {
            let session = Session {
                swapchain: sw,
                exclusive: false,
                last_surface: core::ptr::null_mut(),
                last_error: [0; 192],
            };
            unsafe { *out_session = Box::into_raw(Box::new(session)) as *mut c_void };
            WWN_IOMFB_OK
        }
        Err(e) => {
            set_bytes(last_open_error_mut(), &format!("wwn_iomfb_open: {e:?}"));
            map_err(e)
        }
    }
}

#[no_mangle]
pub extern "C" fn wwn_iomfb_acquire(
    session: *mut c_void,
    out_surface: *mut WwnIomfbSurface,
) -> i32 {
    let Some(s) = session_mut(session) else {
        return WWN_IOMFB_INVALID;
    };
    if out_surface.is_null() {
        set_bytes(&mut s.last_error, "acquire output is null");
        return WWN_IOMFB_INVALID;
    }
    match s.swapchain.acquire() {
        Ok(frame) => {
            unsafe {
                (*out_surface).iosurface = frame.surface;
                (*out_surface).id = frame.iosurface_id;
                (*out_surface).width = frame.width;
                (*out_surface).height = frame.height;
                (*out_surface).bytes_per_row = frame.bytes_per_row;
            }
            WWN_IOMFB_OK
        }
        Err(e) => {
            set_bytes(&mut s.last_error, "acquire failed");
            map_err(e)
        }
    }
}

#[no_mangle]
pub extern "C" fn wwn_iomfb_present_iosurface(
    session: *mut c_void,
    iosurface: *mut c_void,
    _damage: WwnIomfbDamage,
) -> i32 {
    let Some(s) = session_mut(session) else {
        return WWN_IOMFB_INVALID;
    };
    match s.swapchain.present_external(iosurface) {
        Ok(_) => {
            s.last_surface = iosurface;
            WWN_IOMFB_OK
        }
        Err(e) => {
            set_bytes(&mut s.last_error, "present_iosurface failed");
            map_err(e)
        }
    }
}

#[no_mangle]
pub extern "C" fn wwn_iomfb_present_metal_texture(
    session: *mut c_void,
    metal_texture: *mut c_void,
    damage: WwnIomfbDamage,
) -> i32 {
    let Some(s) = session_mut(session) else {
        return WWN_IOMFB_INVALID;
    };
    if metal_texture.is_null() {
        set_bytes(&mut s.last_error, "metal texture is null");
        return IOMFB_C_MISSING;
    }
    let _ = damage;
    // Zero-copy: present the last acquired IOSurface. No Metal blit.
    let surface = s.swapchain.current_surface();
    match s.swapchain.present() {
        Ok(_) => {
            s.last_surface = surface;
            WWN_IOMFB_OK
        }
        Err(e) => {
            set_bytes(&mut s.last_error, "metal present needs an acquired IOSurface");
            map_err(e)
        }
    }
}

#[no_mangle]
pub extern "C" fn wwn_iomfb_restore(session: *mut c_void) -> i32 {
    let Some(s) = session_mut(session) else {
        return WWN_IOMFB_INVALID;
    };
    match s.swapchain.restore() {
        Ok(_) => {
            s.last_surface = core::ptr::null_mut();
            IOMFB_C_OK
        }
        Err(e) => {
            set_bytes(&mut s.last_error, "restore failed");
            map_err(e)
        }
    }
}

#[no_mangle]
pub extern "C" fn wwn_iomfb_set_exclusive(session: *mut c_void, exclusive: i32) -> i32 {
    let Some(s) = session_mut(session) else {
        return WWN_IOMFB_INVALID;
    };
    s.exclusive = exclusive != 0;
    if s.exclusive && !s.last_surface.is_null() {
        let _ = s.swapchain.present_external(s.last_surface);
    }
    WWN_IOMFB_OK
}

#[no_mangle]
pub extern "C" fn wwn_iomfb_last_error(session: *mut c_void) -> *const core::ffi::c_char {
    if let Some(s) = session_mut(session) {
        return s.last_error.as_ptr().cast();
    }
    last_open_error_mut().as_ptr().cast()
}

#[no_mangle]
pub extern "C" fn wwn_iomfb_destroy(session: *mut c_void) {
    if session.is_null() {
        return;
    }
    let boxed = unsafe { Box::from_raw(session as *mut Session) };
    let _ = boxed.swapchain.restore();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_fails_closed_off_device() {
        let rc = wwn_iomfb_open(core::ptr::null_mut());
        assert_eq!(rc, WWN_IOMFB_INVALID);
    }
}
