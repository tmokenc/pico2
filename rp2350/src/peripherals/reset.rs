use super::*;

pub const FRCE_ON: u16 = 0x0; // Force block out of reset (i.e. power it on)
pub const FRCE_OFF: u16 = 0x4; // Force into reset (i.e. power it off)
pub const WDSEL: u16 = 0x8; // Set to 1 if the Watchdog should reset this
pub const DONE: u16 = 0xC; // Is the subsystem ready?

pub struct Reset {
    frce_on: u32,
    frce_off: u32,
    wdsel: u32,
}

impl Default for Reset {
    fn default() -> Self {
        Self {
            frce_on: 0,
            frce_off: 0,
            wdsel: 0,
        }
    }
}

impl Peripheral for Reset {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let value = match address {
            FRCE_ON => self.frce_on,
            FRCE_OFF => self.frce_off,
            WDSEL => self.wdsel,
            DONE => 0x1fff_ffff, // In our simulator, this is always ready
            _ => return Err(PeripheralError::OutOfBounds),
        };

        Ok(value)
    }

    fn write_raw(
        &mut self,
        address: u16,
        value: u32,
        _ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        match address {
            FRCE_ON => self.frce_on = value & 0x1fff_ffff,
            FRCE_OFF => self.frce_off = value & 0x1fff_ffff,
            WDSEL => self.wdsel = value & 0x1fff_ffff,
            DONE => { /* Read only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}
