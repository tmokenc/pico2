/**
 * @file peripherals/xosc.rs
 * @author Nguyen Le Duy
 * @date 06/05/2025
 * @brief XOSC peripheral implementation
 * @todo actually implement the XOSC peripheral, this is just a hotfix to get the simulator running
 */
use super::*;
use crate::utils::extract_bit;

pub const CTRL: u16 = 0x0000; // Watchdog control
pub const LOAD: u16 = 0x0004; // Load the watchdog timer
pub const REASON: u16 = 0x0008; // Logs the reason for the last reset
pub const SCRATCH0: u16 = 0x000c; // Scratch register. Information persists through soft reset of the chip.
pub const SCRATCH1: u16 = 0x0010; // Scratch register. Information persists through soft reset of the chip.
pub const SCRATCH2: u16 = 0x0014; // Scratch register. Information persists through soft reset of the chip.
pub const SCRATCH3: u16 = 0x0018; // Scratch register. Information persists through soft reset of the chip.
pub const SCRATCH4: u16 = 0x001c; // Scratch register. Information persists through soft reset of the chip.
pub const SCRATCH5: u16 = 0x0020; // Scratch register. Information persists through soft reset of the chip.
pub const SCRATCH6: u16 = 0x0024; // Scratch register. Information persists through soft reset of the chip.
pub const SCRATCH7: u16 = 0x0028; // Scratch register. Information persists through soft reset of the chip.

pub struct WatchDog {
    pub pause_dbg1: bool,
    pub pause_dbg0: bool,
    pub pause_jtag: bool,
    pub enable: bool,
    pub timer: u32,
    pub reason_timer: bool,
    pub reason_force: bool,
    pub scratch: [u32; 8],
}

impl Default for WatchDog {
    fn default() -> Self {
        let mut res = Self {
            pause_dbg1: true,
            pause_dbg0: true,
            pause_jtag: true,
            enable: false,
            reason_timer: false,
            reason_force: true,
            scratch: Default::default(),
            timer: 0,
        };

        let entry = 0x1000_0000;

        res.scratch[4] = 0xb007c0d3;
        res.scratch[5] = 0x4ff83f2d ^ entry;
        res.scratch[6] = 0x20000000;
        res.scratch[7] = entry;

        res
    }
}

impl WatchDog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        let Self { scratch, .. } = core::mem::take(self);
        self.scratch = scratch;
    }

    fn reset_trigger(&mut self) {
        log::warn!("Not yet implemented reset trigger");
        todo!()
    }
}

impl Peripheral for WatchDog {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        log::warn!("Watchdog read from {:#x}", address);

        let value = match address {
            CTRL => {
                self.timer
                    | ((self.pause_jtag as u32) << 24)
                    | ((self.pause_dbg0 as u32) << 25)
                    | ((self.pause_dbg1 as u32) << 26)
                    | ((self.enable as u32) << 30)
            }
            LOAD => 0,
            REASON => (self.reason_timer as u32) << 0 | ((self.reason_force as u32) << 1),
            SCRATCH0 | SCRATCH1 | SCRATCH2 | SCRATCH3 | SCRATCH4 | SCRATCH5 | SCRATCH6
            | SCRATCH7 => {
                let index = (address - SCRATCH0) / 4;
                self.scratch[index as usize]
            }

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
            CTRL => {
                if extract_bit(value, 31) != 0 {
                    self.reset_trigger();
                }

                self.enable = extract_bit(value, 30) != 0;
                self.pause_jtag = extract_bit(value, 24) != 0;
                self.pause_dbg0 = extract_bit(value, 25) != 0;
                self.enable = extract_bit(value, 26) != 0;
            }
            LOAD => self.timer = value,
            REASON => { /* read only */ }
            SCRATCH0 | SCRATCH1 | SCRATCH2 | SCRATCH3 | SCRATCH4 | SCRATCH5 | SCRATCH6
            | SCRATCH7 => {
                let index = (address - SCRATCH0) / 4;
                self.scratch[index as usize] = value;
            }

            _ => return Err(PeripheralError::OutOfBounds),
        };
        Ok(())
    }
}
