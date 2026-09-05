//! One IOMFB commit. Set+end is the present. Do not block on SwapWait.

use crate::{Display, Error, Result};
use iomfb_abi::{PresentStatus, WaitOutcome, PRESENT_LAYER, WAIT_INCOMPLETE_GUEST};

impl Display {
    /// Zero-copy present of an existing IOSurface on layer 0.
    ///
    /// The surface must already be the producer's backing (Metal or
    /// compositor). This function does not lock, blit, or copy pixels.
    pub fn present_iosurface(&self, surface: *mut core::ffi::c_void) -> Result<PresentStatus> {
        self.commit_surface(PRESENT_LAYER, surface, false)
    }

    /// Same as [`Self::present_iosurface`] on a confirmed layer `0..3`.
    pub fn present_iosurface_on_layer(
        &self,
        layer: i32,
        surface: *mut core::ffi::c_void,
    ) -> Result<PresentStatus> {
        self.commit_surface(layer, surface, false)
    }

    pub(crate) fn commit_surface(
        &self,
        layer: i32,
        surface: *mut core::ffi::c_void,
        allow_null: bool,
    ) -> Result<PresentStatus> {
        if surface.is_null() && !allow_null {
            return Err(Error::NullSurface);
        }
        let (w, h) = self.size()?;
        if w == 0 || h == 0 {
            return Err(Error::IncompatibleSurface);
        }
        let rect = [0.0, 0.0, f64::from(w), f64::from(h)];
        let token = self.swap_begin()?;
        self.swap_set_layer(layer, surface, rect, rect, 0)?;
        self.swap_end()?;
        // SwapEnd already queued the frame. SwapWait(until-displayed) is
        // lead 0 and blocks. On vphone the paravirt IOMFB never
        // CommandWakes; the 5s gate cancels swaps and wedges the guest
        // (IOMFB swap_wait_gated). Physical still scans out without us
        // waiting.
        Ok(PresentStatus {
            token,
            wait: WaitOutcome::Incomplete(WAIT_INCOMPLETE_GUEST),
            zero_copy: true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_surface_is_rejected() {
        // Off-device open fails before present. The null check is still
        // the first present-path reject when a display exists.
        let err = Display::main();
        assert!(err.is_err());
    }
}
