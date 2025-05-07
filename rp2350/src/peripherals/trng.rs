/**
 * @file peripherals/trng.rs
 * @author Nguyen Le Duy
 * @date 06/03/2025
 * @brief TRNG peripheral implementation
 */
use crate::inspector::InspectionEvent;

use super::*;

pub const RNG_IMR: u16 = 0x0100; // RNG interrupt mask register.
pub const RNG_ISR: u16 = 0x0104; // RNG status register. If corresponding RNG_IMR bit is unmasked, an interrupt will be generated.
pub const RNG_ICR: u16 = 0x0108; // Interrupt/status bit clear Register.
pub const TRNG_CONFIG: u16 = 0x010c; // Selecting the inverter-chain length.
pub const TRNG_VALID: u16 = 0x0110; // 192 bit collection indication.
pub const EHR_DATA0: u16 = 0x0114; // RNG collected bits.
pub const EHR_DATA1: u16 = 0x0118; // RNG collected bits.
pub const EHR_DATA2: u16 = 0x011c; // RNG collected bits.
pub const EHR_DATA3: u16 = 0x0120; // RNG collected bits.
pub const EHR_DATA4: u16 = 0x0124; // RNG collected bits.
pub const EHR_DATA5: u16 = 0x0128; // RNG collected bits.
pub const RND_SOURCE_ENABLE: u16 = 0x012c; // Enable signal for the random source.
pub const SAMPLE_CNT1: u16 = 0x0130; // Counts clocks between sampling of random bit.
pub const AUTOCORR_STATISTIC: u16 = 0x0134; // Statistics about autocorrelation test activations.
pub const TRNG_DEBUG_CONTROL: u16 = 0x0138; // Debug register.
pub const TRNG_SW_RESET: u16 = 0x0140; // Generate internal SW reset within the RNG block.
pub const RNG_DEBUG_EN_INPUT: u16 = 0x01b4; // Enable the RNG debug mode.
pub const TRNG_BUSY: u16 = 0x01b8; // RNG Busy indication.
pub const RST_BITS_COUNTER: u16 = 0x01bc; // Reset the counter of collected bits in the RNG.
pub const RNG_VERSION: u16 = 0x01c0; // Displays the version settings of the TRNG.
pub const RNG_BIST_CNTR_0: u16 = 0x01e0; // Collected BIST results.
pub const RNG_BIST_CNTR_1: u16 = 0x01e4; // Collected BIST results.
pub const RNG_BIST_CNTR_2: u16 = 0x01e8; // Collected BIST results.

// Interrupt mask
pub const VN_ERR: u8 = 1 << 3;
pub const CRNGT_ERR: u8 = 1 << 2;
pub const AUTOCORR_ERR: u8 = 1 << 1;
pub const EHR_VALID: u8 = 1 << 0;

pub struct Trng {
    interrupt_mask: u8,
    interrupts: u8,
    config: u8,
    source_enable: bool,
    sample_cnt1: u32,
    is_valid: bool,
    is_busy: bool,
    autocorr_fails: u16,
    autocorr_trys: u16,
    debug_control: u8,
    debug_enable: bool,
    bist_cntr: [u32; 3],
}

impl Default for Trng {
    fn default() -> Self {
        Self {
            source_enable: false,
            sample_cnt1: 0xffff,
            interrupt_mask: 0b1111,
            interrupts: EHR_VALID, // In our implementation, it is available immediately
            config: 0,
            is_valid: false,
            is_busy: false,
            autocorr_fails: 0,
            autocorr_trys: 0,
            debug_control: 0,
            debug_enable: false,
            bist_cntr: [0; 3],
        }
    }
}

impl Peripheral for Trng {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let value = match address {
            RNG_IMR => self.interrupt_mask as u32,
            RNG_ISR => self.interrupts as u32,
            RNG_ICR => 0,
            TRNG_CONFIG => self.config as u32,
            TRNG_VALID => self.is_valid as u32,
            EHR_DATA0..=EHR_DATA5 => {
                // simulating the rng entropy by generate it when needed
                let value = getrandom::u32().unwrap_or_default();
                ctx.inspector.emit(InspectionEvent::TrngGenerated(value));
                value

                // The below is implementation according to the datasheet
                // just don't care about it for now
                // but in timing sensitive environment, it is crucial to use the below
                // let i = (address - EHR_DATA0) / 4;
                // entropy[i] assuming entropy is [u32; 5]
            }
            RND_SOURCE_ENABLE => self.source_enable as u32,
            SAMPLE_CNT1 => self.sample_cnt1,
            AUTOCORR_STATISTIC => self.autocorr_trys as u32 | (self.autocorr_fails as u32) << 14,
            TRNG_DEBUG_CONTROL => self.debug_control as u32,
            TRNG_SW_RESET => 0,
            RNG_DEBUG_EN_INPUT => self.debug_enable as u32,
            TRNG_BUSY => self.is_busy as u32,
            RST_BITS_COUNTER => 0,
            RNG_VERSION => 0,
            RNG_BIST_CNTR_0 => self.bist_cntr[0],
            RNG_BIST_CNTR_1 => self.bist_cntr[1],
            RNG_BIST_CNTR_2 => self.bist_cntr[2],
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
            RNG_IMR => self.interrupt_mask = value as u8,
            RNG_ICR => {
                let value = value as u8;
                if value & VN_ERR != 0 {
                    self.interrupts &= !VN_ERR;
                }

                if value & CRNGT_ERR != 0 {
                    self.interrupts &= !CRNGT_ERR;
                }

                if value & EHR_VALID != 0 {
                    self.interrupts &= !EHR_VALID;
                }

                // AUTOCORR_ERR cannot be cleared without reseting
            }
            TRNG_CONFIG => self.config = value as u8,
            RND_SOURCE_ENABLE => self.source_enable = value & 1 != 0,
            SAMPLE_CNT1 => self.sample_cnt1 = value,
            AUTOCORR_STATISTIC => {
                self.autocorr_fails = (value >> 14) as u16;
                self.autocorr_trys = value as u16;
            }
            TRNG_DEBUG_CONTROL => self.debug_control = value as u8,
            TRNG_SW_RESET => {
                // Should reset the rng, however in our case, we generate it on the fly,
                // so no need to do anything
            }
            RNG_DEBUG_EN_INPUT => self.debug_enable = value & 1 != 0,
            RST_BITS_COUNTER => {
                // Should reset bits counter and rng valid registers
                // however in our case, we generate it on the fly, so no need to do anything
            }

            TRNG_VALID
            | RNG_ISR
            | TRNG_BUSY
            | RNG_VERSION
            | RNG_BIST_CNTR_0
            | RNG_BIST_CNTR_1
            | RNG_BIST_CNTR_2
            | EHR_DATA0..=EHR_DATA5 => { /* Read Only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        }
        Ok(())
    }
}
