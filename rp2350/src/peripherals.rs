use crate::common::*;
use crate::memory::*;

pub mod bootram;
pub mod busctrl;
pub mod sio;

pub use bootram::BootRam;
pub use busctrl::BusCtrl;
pub use sio::Sio;

#[derive(Debug, PartialEq)]
pub enum PeripheralError {
    OutOfBounds,
    MissingPermission,
}

pub type PeripheralResult<T> = std::result::Result<T, PeripheralError>;

pub struct PeripheralAccessContext {
    pub secure: bool,
    pub requestor: Requestor,
}

impl Default for PeripheralAccessContext {
    fn default() -> Self {
        Self::new()
    }
}

impl PeripheralAccessContext {
    pub fn new() -> Self {
        Self {
            secure: true,
            requestor: Requestor::Proc0,
        }
    }
}

// Purpose: Define the Peripheral trait and a default implementation for unimplemented peripherals.
pub trait Peripheral {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32>;
    fn write_raw(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()>;

    fn write(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        let address = address & 0x0000_0FFF; // Address is 12 bits

        // Atomic access (SIO does not has this features)
        match dbg!((address >> 12) & 0xF) {
            // Normal
            0x0 => self.write_raw(address, value, ctx),
            // XOR on write
            0x1 => {
                let current_value = self.read(address, ctx)?;
                let value = current_value ^ value;
                self.write_raw(address, value, ctx)
            }
            // bitmask set on write
            0x2 => {
                let current_value = self.read(address, ctx)?;
                let value = current_value | value;
                self.write_raw(address, value, ctx)
            }
            // bitmask clear on write
            0x3 => {
                let current_value = self.read(address, ctx)?;
                let value = current_value & !value;
                self.write_raw(address, value, ctx)
            }
            _ => Err(PeripheralError::OutOfBounds),
        }
    }
}

#[derive(Default)]
pub struct UnimplementedPeripheral;

impl Peripheral for UnimplementedPeripheral {
    fn read(&self, _address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        log::warn!("Unimplemented peripheral read");
        Ok(0)
    }

    fn write_raw(
        &mut self,
        _address: u16,
        _value: u32,
        _ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        log::warn!("Unimplemented peripheral write");
        Ok(())
    }
}
