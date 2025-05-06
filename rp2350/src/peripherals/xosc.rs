/**
 * @file peripherals/xosc.rs
 * @author Nguyen Le Duy
 * @date 06/05/2025
 * @brief XOSC peripheral implementation
 * @todo actually implement the XOSC peripheral, this is just a hotfix to get the simulator running
 */
use super::*;
use crate::utils::extract_bit;

pub const CTRL: u16 = 0x00; // Crystal Oscillator Control
pub const STATUS: u16 = 0x04; // Crystal Oscillator STATUS
pub const DORMANT: u16 = 0x08; // Crystal Oscillator pause control
pub const STARTUP: u16 = 0x0C; // Controls the startup delay
pub const COUNT: u16 = 0x10; // A down counter running at the XOSC frequency which counts to zero and stops

const DORMANT_VAL: u32 = 0x636f6d61;
const WAKE: u32 = 0x77616b65;

const CTRL_ENABLE: u32 = 0xfab << 12;
const _CTRL_DISABLE: u32 = 0xd1e << 12;

pub struct Xosc {
    ctrl: u32,
    startup: u32,
    dormant: u32,
    counter: u16,
}

impl Default for Xosc {
    fn default() -> Self {
        Self {
            ctrl: CTRL_ENABLE,
            startup: 0x00c4,
            dormant: WAKE,
            counter: 0,
        }
    }
}

impl Peripheral for Xosc {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let value = match address {
            CTRL => self.ctrl,
            STATUS => 1 << 31 | 1 << 12, // XOSC is always ready in our simulator
            DORMANT => WAKE,
            STARTUP => self.startup,
            COUNT if self.counter == 0 => 0,
            COUNT => 1,
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
            CTRL => self.ctrl = value,
            STATUS => {
                let _clear_bad_write = extract_bit(value, 24);
                // TODO
            }
            DORMANT if value == DORMANT_VAL => self.dormant = DORMANT_VAL,
            DORMANT => self.dormant = WAKE,
            STARTUP => self.startup = value,
            COUNT => {
                let mut counter = value as u16;
                if counter < 4 {
                    counter = 4;
                }

                self.counter = counter;
                // TODO: start the counter
            }
            _ => return Err(PeripheralError::OutOfBounds),
        };

        Ok(())
    }
}
