//! Call any confirmed public export by name.
//!
//! Arity comes from [`iomfb_abi::EXPORTS`]. Stubs return [`Error::Absent`].
//! Do not invent a signature the census does not list.

use crate::{table, Display, Error, Result};
use iomfb_abi::{export_meta, IomfbReturn};

impl Display {
    /// Raw `dlsym` pointer for a confirmed public name.
    pub fn export(&self, name: &str) -> Result<*mut core::ffi::c_void> {
        export_ptr(name)
    }

    /// Confirmed 1-GPR wrapper: `(fb) -> IOReturn`.
    pub unsafe fn call_unary(&self, name: &str) -> Result<i32> {
        let meta = export_meta(name).ok_or(Error::Absent)?;
        if meta.stub {
            return Err(Error::Absent);
        }
        if meta.gpr != 1 || meta.fpr != 0 {
            return Err(Error::WrongArity);
        }
        let f: unsafe extern "C" fn(iomfb_abi::DisplayRef) -> IomfbReturn =
            core::mem::transmute(export_ptr(name)?);
        let rc = f(self.raw);
        Ok(rc)
    }

    /// Confirmed 2-GPR int wrapper: `(fb, i32) -> IOReturn`.
    pub unsafe fn call_int(&self, name: &str, value: i32) -> Result<i32> {
        let meta = export_meta(name).ok_or(Error::Absent)?;
        if meta.stub {
            return Err(Error::Absent);
        }
        if meta.gpr != 2 || meta.fpr != 0 {
            return Err(Error::WrongArity);
        }
        let f: unsafe extern "C" fn(iomfb_abi::DisplayRef, i32) -> IomfbReturn =
            core::mem::transmute(export_ptr(name)?);
        Ok(f(self.raw, value))
    }

    /// Confirmed 2-GPR pointer wrapper: `(fb, *mut c_void) -> IOReturn`.
    pub unsafe fn call_ptr(&self, name: &str, ptr: *mut core::ffi::c_void) -> Result<i32> {
        let meta = export_meta(name).ok_or(Error::Absent)?;
        if meta.stub {
            return Err(Error::Absent);
        }
        if meta.gpr != 2 || meta.fpr != 0 {
            return Err(Error::WrongArity);
        }
        let f: unsafe extern "C" fn(
            iomfb_abi::DisplayRef,
            *mut core::ffi::c_void,
        ) -> IomfbReturn = core::mem::transmute(export_ptr(name)?);
        Ok(f(self.raw, ptr))
    }

    pub fn as_raw(&self) -> *mut core::ffi::c_void {
        self.raw.0
    }
}

/// Resolved pointer for a confirmed, non-stub export.
pub fn export_ptr(name: &str) -> Result<*mut core::ffi::c_void> {
    let meta = export_meta(name).ok_or(Error::Absent)?;
    if meta.stub {
        return Err(Error::Absent);
    }
    table()?.raw_named(name).ok_or(Error::MissingSymbol)
}

/// `true` when the public export is a hard stub on guest 26.1.
pub fn export_is_stub(name: &str) -> bool {
    export_meta(name).map(|e| e.stub).unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stubs_are_absent() {
        assert!(export_is_stub("IOMobileFramebufferSwapSignal"));
        assert!(export_is_stub("IOMobileFramebufferSwapSetUISubRegion"));
        assert!(matches!(export_ptr("IOMobileFramebufferSwapSignal"), Err(Error::Absent)));
    }

    #[test]
    fn unknown_name_is_absent() {
        assert!(export_ptr("IOMobileFramebufferDoesNotExist").is_err());
    }
}
