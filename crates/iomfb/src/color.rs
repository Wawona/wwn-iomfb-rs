//! Color / brightness family. Confirmed on guest 26.1.
//!
//! Gamma tables are `GAMMA_TABLE_SIZE` (0xc0c) bytes. Not a Desktop
//! present prerequisite.

use crate::{map_return, Display, Error, Result};
use iomfb_abi::GAMMA_TABLE_SIZE;

impl Display {
    pub fn set_white_on_black(&self, on: bool) -> Result<()> {
        if let Some(u) = self.userland_state() {
            u.set_white_on_black(on);
            return Ok(());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.set_white_on_black.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, i32::from(on)) })
    }

    pub fn set_color_remap_mode(&self, mode: i32) -> Result<()> {
        if let Some(u) = self.userland_state() {
            u.set_color_remap_mode(mode);
            return Ok(());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.set_color_remap_mode.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, mode) })
    }

    pub fn get_color_remap_mode(&self) -> Result<u32> {
        if let Some(u) = self.userland_state() {
            return Ok(u.get_color_remap_mode());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.get_color_remap_mode.ok_or(Error::MissingSymbol)?;
        let mut mode = 0u32;
        map_return(unsafe { f(raw, &mut mode) })?;
        Ok(mode)
    }

    pub fn set_gamma_table(&self, table: &[u8]) -> Result<()> {
        if let Some(u) = self.userland_state() {
            return u.set_gamma_table(table);
        }
        if table.len() < GAMMA_TABLE_SIZE {
            return Err(Error::IncompatibleSurface);
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.set_gamma_table.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, table.as_ptr().cast()) })
    }

    pub fn get_gamma_table(&self, table: &mut [u8]) -> Result<()> {
        if let Some(u) = self.userland_state() {
            return u.get_gamma_table(table);
        }
        if table.len() < GAMMA_TABLE_SIZE {
            return Err(Error::IncompatibleSurface);
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.get_gamma_table.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, table.as_mut_ptr().cast()) })
    }

    pub fn set_brightness_correction(&self, _value: i32) -> Result<()> {
        if self.is_userland() {
            return Ok(());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols
            .set_brightness_correction
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, _value) })
    }

    pub fn ready_for_swap(&self) -> Result<()> {
        if let Some(u) = self.userland_state() {
            return u.ready_for_swap();
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.ready_for_swap.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, core::ptr::null_mut(), 0) })
    }

    pub fn enable_vsync_notifications(&self) -> Result<()> {
        if self.is_userland() {
            return Ok(());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols
            .enable_vsync_notifications
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, core::ptr::null_mut(), core::ptr::null_mut()) })
    }

    pub fn disable_vsync_notifications(&self) -> Result<()> {
        if self.is_userland() {
            return Ok(());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols
            .disable_vsync_notifications
            .ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw) })
    }

    pub fn wait_surface(&self) -> Result<()> {
        if self.is_userland() {
            return Ok(());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.wait_surface.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw) })
    }

    pub fn set_droppable(&self, droppable: bool) -> Result<()> {
        if let Some(u) = self.userland_state() {
            u.set_droppable(droppable);
            return Ok(());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols.set_droppable.ok_or(Error::MissingSymbol)?;
        map_return(unsafe { f(raw, i32::from(droppable)) })
    }

    pub fn swap_cancel_all_get_current(&self) -> Result<u32> {
        if let Some(u) = self.userland_state() {
            return Ok(u.swap_get_current());
        }
        let (raw, symbols) = self.apple_parts()?;
        let f = symbols
            .swap_cancel_all_get_current
            .ok_or(Error::MissingSymbol)?;
        let mut token = 0u32;
        map_return(unsafe { f(raw, &mut token) })?;
        Ok(token)
    }
}
