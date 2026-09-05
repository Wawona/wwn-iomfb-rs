//! Color / brightness family. Confirmed on guest 26.1.
//!
//! Gamma tables are `GAMMA_TABLE_SIZE` (0xc0c) bytes. Not a Desktop
//! present prerequisite.

use crate::{map_return, Display, Error, Result};
use iomfb_abi::GAMMA_TABLE_SIZE;

impl Display {
    /// Selector `0x13`.
    pub fn set_white_on_black(&self, on: bool) -> Result<()> {
        let f = self
            .symbols
            .set_white_on_black
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, i32::from(on)) })
    }

    /// Selector `0x33`.
    pub fn set_color_remap_mode(&self, mode: i32) -> Result<()> {
        let f = self
            .symbols
            .set_color_remap_mode
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, mode) })
    }

    /// Selector `0x39`.
    pub fn get_color_remap_mode(&self) -> Result<u32> {
        let f = self
            .symbols
            .get_color_remap_mode
            .ok_or(Error::MissingSymbol)?;
        let mut mode = 0u32;
        map_return(unsafe { f(self.raw, &mut mode) })?;
        Ok(mode)
    }

    /// Selector `0x11`. `table` must be `GAMMA_TABLE_SIZE` bytes.
    pub fn set_gamma_table(&self, table: &[u8]) -> Result<()> {
        if table.len() < GAMMA_TABLE_SIZE {
            return Err(Error::IncompatibleSurface);
        }
        let f = self.symbols.set_gamma_table.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, table.as_ptr().cast()) })
    }

    /// Selector `0x1b`. Writes `GAMMA_TABLE_SIZE` bytes.
    pub fn get_gamma_table(&self, table: &mut [u8]) -> Result<()> {
        if table.len() < GAMMA_TABLE_SIZE {
            return Err(Error::IncompatibleSurface);
        }
        let f = self.symbols.get_gamma_table.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, table.as_mut_ptr().cast()) })
    }

    /// Selector `0x32`.
    pub fn set_brightness_correction(&self, value: i32) -> Result<()> {
        let f = self
            .symbols
            .set_brightness_correction
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, value) })
    }

    /// Confirmed 3-GPR hold helper. Not an exclusive grab.
    pub fn ready_for_swap(&self) -> Result<()> {
        let f = self.symbols.ready_for_swap.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, core::ptr::null_mut(), 0) })
    }

    /// Notify type 5 / sel `0x48`. Null callback is a recorded enable.
    pub fn enable_vsync_notifications(&self) -> Result<()> {
        let f = self
            .symbols
            .enable_vsync_notifications
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, core::ptr::null_mut(), core::ptr::null_mut()) })
    }

    pub fn disable_vsync_notifications(&self) -> Result<()> {
        let f = self
            .symbols
            .disable_vsync_notifications
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw) })
    }

    pub fn wait_surface(&self) -> Result<()> {
        let f = self.symbols.wait_surface.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw) })
    }

    pub fn set_droppable(&self, droppable: bool) -> Result<()> {
        let f = self.symbols.set_droppable.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(self.raw, i32::from(droppable)) })
    }

    /// Selector `0x5c`.
    pub fn swap_cancel_all_get_current(&self) -> Result<u32> {
        let f = self
            .symbols
            .swap_cancel_all_get_current
            .ok_or(Error::MissingSymbol)?;
        let mut token = 0u32;
        map_return(unsafe { f(self.raw, &mut token) })?;
        Ok(token)
    }
}
