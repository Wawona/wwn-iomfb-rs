//! TrollStore userspace backend. No `IOConnect`, no userclient.
//!
//! Jailbreak full RE is [`crate::channel::Channel::Jailbreak`]. Present
//! here is an IOSurface plus an optional host callback (CALayer / iland
//! / CAMetalLayer). Hardware families (HDCP, factory, KernelTests) stay
//! [`Error::Absent`].

use crate::{Error, Result, Wait};
use iomfb_abi::GAMMA_TABLE_SIZE;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

/// C present hook. `surface` may be null on restore.
pub type PresentFn = unsafe extern "C" fn(
    ctx: *mut core::ffi::c_void,
    surface: *mut core::ffi::c_void,
    width: u32,
    height: u32,
    token: i32,
    layer: i32,
);

static CONFIGURED: OnceLock<Mutex<Option<(u32, u32)>>> = OnceLock::new();

fn configured() -> &'static Mutex<Option<(u32, u32)>> {
    CONFIGURED.get_or_init(|| Mutex::new(None))
}

/// Pin userland geometry before [`crate::Display::main`].
pub fn configure(width: u32, height: u32) -> Result<()> {
    if width == 0 || height == 0 {
        return Err(Error::IncompatibleSurface);
    }
    *configured().lock().unwrap_or_else(|e| e.into_inner()) = Some((width, height));
    Ok(())
}

pub fn resolve_geometry() -> Result<(u32, u32)> {
    if let Some(pair) = configured()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .copied()
    {
        return Ok(pair);
    }
    if let (Ok(w), Ok(h)) = (std::env::var("WWN_IOMFB_WIDTH"), std::env::var("WWN_IOMFB_HEIGHT")) {
        if let (Ok(ww), Ok(hh)) = (w.parse::<u32>(), h.parse::<u32>()) {
            if ww > 0 && hh > 0 {
                return Ok((ww, hh));
            }
        }
    }
    let mut w = 0u32;
    let mut h = 0u32;
    if unsafe { crate::surface::host_size(&mut w, &mut h) } == 0 && w > 0 && h > 0 {
        return Ok((w, h));
    }
    Err(Error::LoadFailed)
}

pub struct Userland {
    width: u32,
    height: u32,
    id: u32,
    is_main: bool,
    next_token: AtomicI32,
    current: AtomicI32,
    pending_token: AtomicI32,
    pending_layer: AtomicI32,
    pending_surface: AtomicUsize,
    last_surface: AtomicUsize,
    droppable: AtomicBool,
    white_on_black: AtomicBool,
    color_remap: AtomicU32,
    gamma: Mutex<Vec<u8>>,
    present: Mutex<Option<(PresentFn, usize)>>,
}

impl Userland {
    pub fn new(width: u32, height: u32, is_main: bool) -> Result<Self> {
        if width == 0 || height == 0 {
            return Err(Error::IncompatibleSurface);
        }
        Ok(Self {
            width,
            height,
            id: 1,
            is_main,
            next_token: AtomicI32::new(1),
            current: AtomicI32::new(0),
            pending_token: AtomicI32::new(0),
            pending_layer: AtomicI32::new(0),
            pending_surface: AtomicUsize::new(0),
            last_surface: AtomicUsize::new(0),
            droppable: AtomicBool::new(false),
            white_on_black: AtomicBool::new(false),
            color_remap: AtomicU32::new(0),
            gamma: Mutex::new(vec![0u8; GAMMA_TABLE_SIZE]),
            present: Mutex::new(None),
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn is_main(&self) -> bool {
        self.is_main
    }

    pub fn last_surface(&self) -> *mut core::ffi::c_void {
        self.last_surface.load(Ordering::SeqCst) as *mut core::ffi::c_void
    }

    pub fn set_present(&self, f: Option<PresentFn>, ctx: *mut core::ffi::c_void) {
        *self.present.lock().unwrap_or_else(|e| e.into_inner()) =
            f.map(|fp| (fp, ctx as usize));
    }

    pub fn swap_begin(&self) -> i32 {
        let token = self.next_token.fetch_add(1, Ordering::SeqCst);
        self.pending_token.store(token, Ordering::SeqCst);
        self.pending_layer.store(0, Ordering::SeqCst);
        self.pending_surface.store(0, Ordering::SeqCst);
        token
    }

    pub fn swap_set_layer(&self, layer: i32, surface: *mut core::ffi::c_void) -> Result<()> {
        if !(0..4).contains(&layer) {
            return Err(Error::Iomfb(-1));
        }
        self.pending_layer.store(layer, Ordering::SeqCst);
        self.pending_surface
            .store(surface as usize, Ordering::SeqCst);
        Ok(())
    }

    pub fn swap_end(&self) -> Result<()> {
        let token = self.pending_token.load(Ordering::SeqCst);
        let layer = self.pending_layer.load(Ordering::SeqCst);
        let surface = self.pending_surface.load(Ordering::SeqCst);
        self.current.store(token, Ordering::SeqCst);
        self.last_surface.store(surface, Ordering::SeqCst);
        if let Some((f, ctx)) = *self.present.lock().unwrap_or_else(|e| e.into_inner()) {
            unsafe {
                f(
                    ctx as *mut core::ffi::c_void,
                    surface as *mut core::ffi::c_void,
                    self.width,
                    self.height,
                    token,
                    layer,
                );
            }
        }
        Ok(())
    }

    pub fn swap_wait(&self, token: i32, _wait: Wait) -> Result<()> {
        if token == 0 || token == self.current.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err(Error::Iomfb(iomfb_abi::WAIT_INCOMPLETE_GUEST))
        }
    }

    pub fn swap_cancel(&self, token: i32) -> Result<()> {
        if self.pending_token.load(Ordering::SeqCst) == token {
            self.pending_surface.store(0, Ordering::SeqCst);
        }
        Ok(())
    }

    pub fn swap_cancel_all(&self) -> Result<()> {
        self.pending_surface.store(0, Ordering::SeqCst);
        Ok(())
    }

    pub fn swap_get_current(&self) -> u32 {
        self.current.load(Ordering::SeqCst) as u32
    }

    pub fn restore(&self) -> Result<()> {
        self.pending_surface.store(0, Ordering::SeqCst);
        self.last_surface.store(0, Ordering::SeqCst);
        if let Some((f, ctx)) = *self.present.lock().unwrap_or_else(|e| e.into_inner()) {
            unsafe {
                f(
                    ctx as *mut core::ffi::c_void,
                    core::ptr::null_mut(),
                    self.width,
                    self.height,
                    0,
                    0,
                );
            }
        }
        Ok(())
    }

    pub fn set_white_on_black(&self, on: bool) {
        self.white_on_black.store(on, Ordering::SeqCst);
    }

    pub fn set_color_remap_mode(&self, mode: i32) {
        self.color_remap.store(mode as u32, Ordering::SeqCst);
    }

    pub fn get_color_remap_mode(&self) -> u32 {
        self.color_remap.load(Ordering::SeqCst)
    }

    pub fn set_gamma_table(&self, table: &[u8]) -> Result<()> {
        if table.len() < GAMMA_TABLE_SIZE {
            return Err(Error::IncompatibleSurface);
        }
        self.gamma
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .copy_from_slice(&table[..GAMMA_TABLE_SIZE]);
        Ok(())
    }

    pub fn get_gamma_table(&self, table: &mut [u8]) -> Result<()> {
        if table.len() < GAMMA_TABLE_SIZE {
            return Err(Error::IncompatibleSurface);
        }
        table[..GAMMA_TABLE_SIZE].copy_from_slice(
            &self.gamma.lock().unwrap_or_else(|e| e.into_inner()),
        );
        Ok(())
    }

    pub fn set_droppable(&self, droppable: bool) {
        self.droppable.store(droppable, Ordering::SeqCst);
    }

    pub fn ready_for_swap(&self) -> Result<()> {
        if self.is_main {
            Ok(())
        } else {
            Err(Error::Iomfb(-536870206))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn swap_is_local() {
        let u = Userland::new(640, 480, true).unwrap();
        let token = u.swap_begin();
        assert!(token > 0);
        u.swap_set_layer(0, core::ptr::null_mut()).unwrap();
        u.swap_end().unwrap();
        u.swap_wait(token, crate::Wait::UntilDisplayed).unwrap();
        assert_eq!(u.swap_get_current(), token as u32);
    }

    #[test]
    fn zero_size_is_rejected() {
        assert!(Userland::new(0, 480, true).is_err());
    }

    #[test]
    fn present_callback_fires() {
        use std::sync::atomic::AtomicI32;
        static HITS: AtomicI32 = AtomicI32::new(0);
        unsafe extern "C" fn cb(
            _ctx: *mut core::ffi::c_void,
            surface: *mut core::ffi::c_void,
            width: u32,
            height: u32,
            token: i32,
            layer: i32,
        ) {
            assert_eq!((width, height, layer), (64, 48, 1));
            assert!(!surface.is_null());
            assert!(token > 0);
            HITS.fetch_add(1, Ordering::SeqCst);
        }
        let u = Userland::new(64, 48, true).unwrap();
        u.set_present(Some(cb), core::ptr::null_mut());
        let token = u.swap_begin();
        u.swap_set_layer(1, 0x1 as *mut core::ffi::c_void).unwrap();
        u.swap_end().unwrap();
        u.swap_wait(token, crate::Wait::UntilDisplayed).unwrap();
        assert_eq!(HITS.load(Ordering::SeqCst), 1);
        assert_eq!(u.last_surface() as usize, 1);
    }
}
