/**
 * @file peripherals/otp.rs
 * @author Nguyen Le Duy
 * @date 22/01/2025
 * @brief One Time Programmable (OTP) peripheral implementation
 * @todo need real hardware to implement this
 */
use super::*;

pub const CRIT0: u16 = 0x038; // Page 0 critical boot flags (RBIT-8)
pub const CRIT0_R7: u16 = 0x03f; // CRIT0..CRIT7 are copied
pub const CRIT1: u16 = 0x040; // Page 1 critical boot flags (RBIT-8)
pub const CRIT1_R7: u16 = 0x047; // CRIT1..CRIT7 are copied
pub const BOOT_FLAGS0: u16 = 0x048; // Disable/Enable boot paths/features in the RP2350 mask ROM.
pub const BOOT_FLAGS0_R2: u16 = 0x04a; // Copied
pub const BOOT_FLAGS1: u16 = 0x04b; // Disable/Enable boot paths/features in the RP2350 mask ROM.
pub const BOOT_FLAGS1_R2: u16 = 0x04d; // Copied

pub struct Otp {}

impl Default for Otp {
    fn default() -> Self {
        Otp {}
    }
}

impl Peripheral for Otp {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let value = match address {
            CRIT0..=CRIT0_R7 => 0b01,     // Disable ARM core for now,
            CRIT1..=CRIT1_R7 => 0b001000, // Bootarch RISC-V
            BOOT_FLAGS0..=BOOT_FLAGS0_R2 => 0,
            BOOT_FLAGS1..=BOOT_FLAGS1_R2 => 0,

            _ => {
                log::warn!("Unimplemented OTP read at address {:#X}", address);
                0
            }
        };

        Ok(value)
    }

    fn write_raw(
        &mut self,
        _address: u16,
        _value: u32,
        _ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        // One Time Programable, so everything here are read only
        Ok(())
    }
}
