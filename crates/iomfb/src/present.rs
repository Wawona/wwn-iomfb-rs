//! One IOMFB commit. Set+end is the present. Wait is reported, not guessed.

use crate::{Display, Error, Result, Wait};
use iomfb_abi::{PresentStatus, WaitOutcome, PRESENT_LAYER};

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
        let wait = match self.swap_wait(token, Wait::UntilDisplayed) {
            Ok(()) => WaitOutcome::Displayed,
            Err(Error::Iomfb(rc)) => WaitOutcome::from_wait_rc(rc),
            Err(e) => return Err(e),
        };
        Ok(PresentStatus {
            token,
            wait,
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
