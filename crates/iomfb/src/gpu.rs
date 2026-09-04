//! GPU swapchain: Metal-renderable IOSurfaces presented without a copy.
//!
//! ```text
//! Metal render target  ==  IOSurface  ==  IOMFB SwapSetLayer
//! ```
//!
//! vphone has IOMFB and no Metal. Acquire still yields IOSurfaces.
//! `metal_texture` is null there. Real devices wrap the same surface.

use crate::surface::{IoSurface, MetalDevice, MetalTexture};
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
    /// Kept alive so textures stay valid.
    _metal: Option<MetalDevice>,
}

/// One acquired back buffer. Render into `metal_texture` when present.
pub struct GpuFrame {
    pub index: usize,
    pub surface: *mut core::ffi::c_void,
    pub metal_texture: *mut core::ffi::c_void,
    pub iosurface_id: u32,
    pub width: u32,
    pub height: u32,
}

impl GpuSwapchain {
    /// Open the main display, power on, allocate BGRA IOSurfaces.
    /// Metal wrap is best-effort (null on vphone).
    pub fn main() -> Result<Self> {
        let display = Display::main().or_else(|_| Display::secondary())?;
        Self::attach(display)
    }

    pub fn attach(display: Display) -> Result<Self> {
        let _ = display.request_power_on();
        let (width, height) = display.size()?;
        if width == 0 || height == 0 {
            return Err(Error::IncompatibleSurface);
        }
        let metal = MetalDevice::system();
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
            _metal: metal,
        })
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn has_metal(&self) -> bool {
        self.slots.iter().any(|s| s.texture.is_some())
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
        })
    }

    /// Present the last acquired slot. Zero-copy.
    pub fn present(&mut self) -> Result<PresentStatus> {
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
}
