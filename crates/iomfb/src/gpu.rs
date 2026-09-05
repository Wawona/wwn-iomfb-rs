//! GPU swapchain: Metal-renderable IOSurfaces presented without a copy.
//!
//! ```text
//! Metal render target  ==  IOSurface  ==  host present callback
//! ```
//!
//! vphone `wawona-jb` is the proof device. Metal.framework is present.
//! `has_metal()` is true when `MTLCreateSystemDefaultDevice` succeeds.

use crate::surface::{IoSurface, MetalDevice, MetalQueue, MetalTexture};
use crate::{Display, Error, Result};
use iomfb_abi::{PixelFormat, PresentStatus, SWAPCHAIN_BUFFERS};

struct Slot {
    surface: IoSurface,
    texture: Option<MetalTexture>,
}

/// Triple-buffered IOMFB present path for tipas, tweaks, and Wawona.
pub struct GpuSwapchain {
    display: Display,
    slots: Vec<Slot>,
    next: usize,
    width: u32,
    height: u32,
    metal: Option<MetalDevice>,
    queue: Option<MetalQueue>,
}

/// One acquired back buffer. Render into `metal_texture` when present.
pub struct GpuFrame {
    pub index: usize,
    pub surface: *mut core::ffi::c_void,
    pub metal_texture: *mut core::ffi::c_void,
    pub iosurface_id: u32,
    pub width: u32,
    pub height: u32,
    pub bytes_per_row: u32,
}

impl GpuSwapchain {
    /// Open the main display, power on, allocate BGRA IOSurfaces.
    /// Metal wrap + queue when the guest has a system MTL device.
    pub fn main() -> Result<Self> {
        let display = Display::main()?;
        Self::attach(display)
    }

    pub fn attach(display: Display) -> Result<Self> {
        let _ = display.request_power_on();
        let _ = display.set_video_power_savings(false);
        let (width, height) = display.size()?;
        if width == 0 || height == 0 {
            return Err(Error::IncompatibleSurface);
        }
        let metal = MetalDevice::system();
        let queue = metal.as_ref().and_then(MetalDevice::new_queue);
        let mut slots = Vec::with_capacity(SWAPCHAIN_BUFFERS);
        for _ in 0..SWAPCHAIN_BUFFERS {
            let surface = IoSurface::create(width, height, PixelFormat::Bgra8888)
                .ok_or(Error::SurfaceCreateFailed)?;
            if !surface.matches(width, height, PixelFormat::Bgra8888) {
                return Err(Error::IncompatibleSurface);
            }
            let texture = metal
                .as_ref()
                .and_then(|dev| dev.wrap_texture(&surface, true));
            slots.push(Slot { surface, texture });
        }
        Ok(Self {
            display,
            slots,
            next: 0,
            width,
            height,
            metal,
            queue,
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn has_metal(&self) -> bool {
        self.metal.is_some() && self.queue.is_some() && self.slots.iter().any(|s| s.texture.is_some())
    }

    /// GPU clear into the current back buffer. Same IOSurface IOMFB will swap.
    pub fn clear(&self, rgba: [f32; 4]) -> Result<()> {
        let slot = self.slots.get(self.next).ok_or(Error::IncompatibleSurface)?;
        let queue = self.queue.as_ref().ok_or(Error::SurfaceCreateFailed)?;
        let texture = slot.texture.as_ref().ok_or(Error::SurfaceCreateFailed)?;
        queue.clear(texture, rgba)
    }

    /// Next back buffer. GPU apps draw to `metal_texture` then [`Self::present`].
    pub fn acquire(&mut self) -> Result<GpuFrame> {
        let index = self.next;
        let slot = self.slots.get(index).ok_or(Error::IncompatibleSurface)?;
        Ok(GpuFrame {
            index,
            surface: slot.surface.as_ptr(),
            metal_texture: slot
                .texture
                .as_ref()
                .map(MetalTexture::as_ptr)
                .unwrap_or(core::ptr::null_mut()),
            iosurface_id: slot.surface.id(),
            width: self.width,
            height: self.height,
            bytes_per_row: slot.surface.bytes_per_row(),
        })
    }

    pub fn current_surface(&self) -> *mut core::ffi::c_void {
        self.slots
            .get(self.next)
            .map(|slot| slot.surface.as_ptr())
            .unwrap_or(core::ptr::null_mut())
    }

    /// Present the last acquired slot. Waits GPU, then userspace present. Zero-copy.
    pub fn present(&mut self) -> Result<PresentStatus> {
        if let Some(queue) = self.queue.as_ref() {
            queue.wait()?;
        }
        let index = self.next;
        let surface = self
            .slots
            .get(index)
            .ok_or(Error::IncompatibleSurface)?
            .surface
            .as_ptr();
        let status = self.display.present_iosurface(surface)?;
        self.next = (self.next + 1) % self.slots.len();
        Ok(status)
    }

    /// Wawona / compositor path: present a producer IOSurface as-is.
    /// Same IOSurfaceID must reach IOMFB. No blit in this crate.
    pub fn present_external(&self, surface: *mut core::ffi::c_void) -> Result<PresentStatus> {
        if surface.is_null() {
            return Err(Error::NullSurface);
        }
        self.display.present_iosurface(surface)
    }

    pub fn restore(&self) -> Result<PresentStatus> {
        self.display.restore_default_surface()?;
        Ok(PresentStatus {
            token: 0,
            wait: iomfb_abi::WaitOutcome::Displayed,
            zero_copy: true,
        })
    }

    pub fn display(&self) -> &Display {
        &self.display
    }

    /// Pixel grid of this swapchain. Same size IOMFB `GetDisplaySize` used.
    pub fn touch_map(&self) -> crate::TouchMap {
        crate::TouchMap::new(self.width, self.height)
    }

    /// Seat on this swapchain. HID steal stays the caller.
    pub fn touch_seat(&self) -> crate::TouchSeat {
        crate::TouchSeat::new(self.touch_map())
    }
}

impl Drop for GpuSwapchain {
    fn drop(&mut self) {
        let _ = self.display.restore_default_surface();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn main_fails_closed_off_device() {
        assert!(GpuSwapchain::main().is_err());
    }

    #[test]
    fn attach_userland_does_not_need_apple() {
        let d = Display::userland(32, 32).unwrap();
        match GpuSwapchain::attach(d) {
            Ok(sw) => {
                assert!(sw.display().is_userland());
                assert_eq!(sw.size(), (32, 32));
            }
            Err(Error::SurfaceCreateFailed) => {}
            Err(e) => panic!("unexpected attach error: {e:?}"),
        }
    }
}
