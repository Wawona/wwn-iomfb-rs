//! IOSurface + Metal glue. Public SDK only. No IOMFB header.
//!
//! On non-Apple targets these helpers return null / 0 so the crate
//! still tests. On Apple they call `ffi/iomfb_surface.m`.

use iomfb_abi::{PixelFormat, PIXEL_FORMAT_BGRA};

#[cfg(all(target_vendor = "apple"))]
extern "C" {
    fn iomfb_glue_surface_create(width: u32, height: u32, fourcc: u32) -> *mut core::ffi::c_void;
    fn iomfb_glue_surface_retain(surface: *mut core::ffi::c_void);
    fn iomfb_glue_surface_release(surface: *mut core::ffi::c_void);
    fn iomfb_glue_surface_id(surface: *mut core::ffi::c_void) -> u32;
    fn iomfb_glue_surface_width(surface: *mut core::ffi::c_void) -> u32;
    fn iomfb_glue_surface_height(surface: *mut core::ffi::c_void) -> u32;
    fn iomfb_glue_surface_bytes_per_row(surface: *mut core::ffi::c_void) -> u32;
    fn iomfb_glue_surface_fourcc(surface: *mut core::ffi::c_void) -> u32;
    fn iomfb_glue_surface_matches(
        surface: *mut core::ffi::c_void,
        width: u32,
        height: u32,
        fourcc: u32,
    ) -> i32;
    fn iomfb_glue_metal_device() -> *mut core::ffi::c_void;
    fn iomfb_glue_metal_release(object: *mut core::ffi::c_void);
    fn iomfb_glue_metal_texture_wrap(
        device: *mut core::ffi::c_void,
        surface: *mut core::ffi::c_void,
        render_target: i32,
    ) -> *mut core::ffi::c_void;
    fn iomfb_glue_metal_queue(device: *mut core::ffi::c_void) -> *mut core::ffi::c_void;
    fn iomfb_glue_metal_wait(queue: *mut core::ffi::c_void) -> i32;
    fn iomfb_glue_metal_clear(
        queue: *mut core::ffi::c_void,
        texture: *mut core::ffi::c_void,
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    ) -> i32;
}

#[cfg(not(target_vendor = "apple"))]
mod stub {
    pub unsafe fn iomfb_glue_surface_create(_: u32, _: u32, _: u32) -> *mut core::ffi::c_void {
        core::ptr::null_mut()
    }
    pub unsafe fn iomfb_glue_surface_retain(_: *mut core::ffi::c_void) {}
    pub unsafe fn iomfb_glue_surface_release(_: *mut core::ffi::c_void) {}
    pub unsafe fn iomfb_glue_surface_id(_: *mut core::ffi::c_void) -> u32 {
        0
    }
    pub unsafe fn iomfb_glue_surface_width(_: *mut core::ffi::c_void) -> u32 {
        0
    }
    pub unsafe fn iomfb_glue_surface_height(_: *mut core::ffi::c_void) -> u32 {
        0
    }
    pub unsafe fn iomfb_glue_surface_bytes_per_row(_: *mut core::ffi::c_void) -> u32 {
        0
    }
    pub unsafe fn iomfb_glue_surface_fourcc(_: *mut core::ffi::c_void) -> u32 {
        0
    }
    pub unsafe fn iomfb_glue_surface_matches(
        _: *mut core::ffi::c_void,
        _: u32,
        _: u32,
        _: u32,
    ) -> i32 {
        0
    }
    pub unsafe fn iomfb_glue_metal_device() -> *mut core::ffi::c_void {
        core::ptr::null_mut()
    }
    pub unsafe fn iomfb_glue_metal_release(_: *mut core::ffi::c_void) {}
    pub unsafe fn iomfb_glue_metal_texture_wrap(
        _: *mut core::ffi::c_void,
        _: *mut core::ffi::c_void,
        _: i32,
    ) -> *mut core::ffi::c_void {
        core::ptr::null_mut()
    }
    pub unsafe fn iomfb_glue_metal_queue(_: *mut core::ffi::c_void) -> *mut core::ffi::c_void {
        core::ptr::null_mut()
    }
    pub unsafe fn iomfb_glue_metal_wait(_: *mut core::ffi::c_void) -> i32 {
        -1
    }
    pub unsafe fn iomfb_glue_metal_clear(
        _: *mut core::ffi::c_void,
        _: *mut core::ffi::c_void,
        _: f32,
        _: f32,
        _: f32,
        _: f32,
    ) -> i32 {
        -1
    }
}

#[cfg(not(target_vendor = "apple"))]
use stub::*;

/// Owned IOSurface. Released on drop.
pub struct IoSurface {
    raw: *mut core::ffi::c_void,
}

unsafe impl Send for IoSurface {}

impl IoSurface {
    pub fn create(width: u32, height: u32, format: PixelFormat) -> Option<Self> {
        let raw = unsafe { iomfb_glue_surface_create(width, height, format.fourcc()) };
        if raw.is_null() {
            None
        } else {
            Some(Self { raw })
        }
    }

    /// Borrow an existing IOSurface without taking ownership.
    pub fn from_raw_borrowed(raw: *mut core::ffi::c_void) -> Option<Self> {
        if raw.is_null() {
            return None;
        }
        unsafe { iomfb_glue_surface_retain(raw) };
        Some(Self { raw })
    }

    pub fn as_ptr(&self) -> *mut core::ffi::c_void {
        self.raw
    }

    pub fn id(&self) -> u32 {
        unsafe { iomfb_glue_surface_id(self.raw) }
    }

    pub fn width(&self) -> u32 {
        unsafe { iomfb_glue_surface_width(self.raw) }
    }

    pub fn height(&self) -> u32 {
        unsafe { iomfb_glue_surface_height(self.raw) }
    }

    pub fn bytes_per_row(&self) -> u32 {
        unsafe { iomfb_glue_surface_bytes_per_row(self.raw) }
    }

    pub fn fourcc(&self) -> u32 {
        unsafe { iomfb_glue_surface_fourcc(self.raw) }
    }

    pub fn matches(&self, width: u32, height: u32, format: PixelFormat) -> bool {
        unsafe { iomfb_glue_surface_matches(self.raw, width, height, format.fourcc()) != 0 }
    }

    pub fn is_bgra(&self) -> bool {
        self.fourcc() == PIXEL_FORMAT_BGRA
    }
}

impl Drop for IoSurface {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { iomfb_glue_surface_release(self.raw) };
            self.raw = core::ptr::null_mut();
        }
    }
}

/// System Metal device. Null only when the guest has no MTL device.
pub struct MetalDevice {
    raw: *mut core::ffi::c_void,
}

impl MetalDevice {
    pub fn system() -> Option<Self> {
        let raw = unsafe { iomfb_glue_metal_device() };
        if raw.is_null() {
            None
        } else {
            Some(Self { raw })
        }
    }

    pub fn as_ptr(&self) -> *mut core::ffi::c_void {
        self.raw
    }

    pub fn new_queue(&self) -> Option<MetalQueue> {
        let raw = unsafe { iomfb_glue_metal_queue(self.raw) };
        if raw.is_null() {
            None
        } else {
            Some(MetalQueue { raw })
        }
    }

    /// Wrap `surface` as a Metal texture on this device. Same backing.
    pub fn wrap_texture(&self, surface: &IoSurface, render_target: bool) -> Option<MetalTexture> {
        let raw = unsafe {
            iomfb_glue_metal_texture_wrap(self.raw, surface.as_ptr(), i32::from(render_target))
        };
        if raw.is_null() {
            None
        } else {
            Some(MetalTexture { raw })
        }
    }
}

impl Drop for MetalDevice {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { iomfb_glue_metal_release(self.raw) };
            self.raw = core::ptr::null_mut();
        }
    }
}

/// Command queue for encode + wait before IOMFB swap.
pub struct MetalQueue {
    raw: *mut core::ffi::c_void,
}

impl MetalQueue {
    pub fn as_ptr(&self) -> *mut core::ffi::c_void {
        self.raw
    }

    pub fn wait(&self) -> crate::Result<()> {
        if unsafe { iomfb_glue_metal_wait(self.raw) } == 0 {
            Ok(())
        } else {
            Err(crate::Error::SurfaceCreateFailed)
        }
    }

    pub fn clear(&self, texture: &MetalTexture, rgba: [f32; 4]) -> crate::Result<()> {
        let rc = unsafe {
            iomfb_glue_metal_clear(self.raw, texture.as_ptr(), rgba[0], rgba[1], rgba[2], rgba[3])
        };
        if rc == 0 {
            Ok(())
        } else {
            Err(crate::Error::SurfaceCreateFailed)
        }
    }
}

impl Drop for MetalQueue {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { iomfb_glue_metal_release(self.raw) };
            self.raw = core::ptr::null_mut();
        }
    }
}

/// Metal texture that aliases an IOSurface. Do not blit from this.
pub struct MetalTexture {
    raw: *mut core::ffi::c_void,
}

impl MetalTexture {
    pub fn as_ptr(&self) -> *mut core::ffi::c_void {
        self.raw
    }
}

impl Drop for MetalTexture {
    fn drop(&mut self) {
        if !self.raw.is_null() {
            unsafe { iomfb_glue_metal_release(self.raw) };
            self.raw = core::ptr::null_mut();
        }
    }
}
