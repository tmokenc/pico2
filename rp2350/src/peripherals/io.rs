/**
 * @file peripherals/io.rs
 * @author Nguyen Le Duy
 * @date 14/04/2025
 * @brief IO Bank0 / QSPI peripherals
 * @todo summary... proc1... and it could be better, a lot of duplicated code here...
 *
 */
use super::*;
use crate::utils::extract_bits;

pub const GPIO_STATUS: u16 = 0x00;
pub const GPIO_CTRL: u16 = 0x04;

pub const INTR0: u16 = 0x230;
pub const INTR5: u16 = 0x244;

pub const PROC0_INTE0: u16 = 0x248;
pub const PROC0_INTE5: u16 = 0x25c;
pub const PROC0_INTF0: u16 = 0x260;
pub const PROC0_INTF5: u16 = 0x274;
pub const PROC0_INTS0: u16 = 0x278;
pub const PROC0_INTS5: u16 = 0x28c;

pub const GPIO_END: u16 = 0x17c;

pub const GPIO_STEP: u16 = 0x08;

#[derive(Default)]
pub struct IoBank0;

impl Peripheral for IoBank0 {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let gpio = ctx.gpio.borrow();
        let value = match address {
            0..=GPIO_END => {
                let index = address / GPIO_STEP;

                match address % GPIO_STEP {
                    GPIO_STATUS => gpio.pin_status(index as _),
                    GPIO_CTRL => gpio.get_pin(index as _).unwrap().ctrl,
                    _ => return Err(PeripheralError::OutOfBounds),
                }
            }

            INTR0..=INTR5 => {
                let mut result = 0;
                let index = (address - INTR0) / 0x4;

                for i in (0..8).rev() {
                    result <<= 4;
                    let pin_index = i + (index * 8);

                    if let Some(pin) = gpio.get_pin(pin_index as _) {
                        result |= pin.interrupt_raw as u32;
                    }
                }

                result
            }

            PROC0_INTE0..=PROC0_INTE5 => {
                let mut result = 0;
                let index = (address - PROC0_INTE0) / 0x4;

                for i in (0..8).rev() {
                    result <<= 4;
                    let pin_index = i + (index * 8);

                    if let Some(pin) = gpio.get_pin(pin_index as _) {
                        result |= pin.interrupt_mask as u32;
                    }
                }

                result
            }

            PROC0_INTF0..=PROC0_INTF5 => {
                let mut result = 0;
                let index = (address - PROC0_INTF0) / 0x4;

                for i in (0..8).rev() {
                    result <<= 4;
                    let pin_index = i + (index * 8);

                    if let Some(pin) = gpio.get_pin(pin_index as _) {
                        result |= pin.interrupt_force as u32;
                    }
                }

                result
            }

            PROC0_INTS0..=PROC0_INTS5 => {
                let mut result = 0;
                let index = (address - PROC0_INTF0) / 0x4;

                for i in (0..8).rev() {
                    result <<= 4;
                    let pin_index = i + (index * 8);

                    if let Some(pin) = gpio.get_pin(pin_index as _) {
                        result |= pin.interrupt_status() as u32;
                    }
                }

                result
            }

            _ => return Err(PeripheralError::OutOfBounds),
        };

        Ok(value)
    }

    fn write_raw(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        let mut gpio = ctx.gpio.borrow_mut();
        match address {
            0..=GPIO_END => {
                let index = address / GPIO_STEP;

                match address % GPIO_STEP {
                    GPIO_STATUS => {} // Read only
                    GPIO_CTRL => gpio.update_pin_ctrl(index as _, value),
                    _ => return Err(PeripheralError::OutOfBounds),
                }
            }

            INTR0..=INTR5 => {
                let index = (address - INTR0) / 0x4;

                for i in 0u32..8 {
                    let pin_index = i as u16 + (index * 8);
                    let value_start = i * 4;
                    let value_end = i * 4 + 3;
                    let value = extract_bits(value, value_start..=value_end);

                    if let Some(pin) = gpio.get_pin_mut(pin_index as _) {
                        pin.interrupt_mask = value as u8;
                    }
                }

                gpio.update_interrupt();
            }

            PROC0_INTE0..=PROC0_INTE5 => {
                let index = (address - PROC0_INTE0) / 0x4;

                for i in 0u32..8 {
                    let pin_index = i as u16 + (index * 8);
                    let value_start = i * 4;
                    let value_end = i * 4 + 3;
                    let value = extract_bits(value, value_start..=value_end);

                    if let Some(pin) = gpio.get_pin_mut(pin_index as _) {
                        pin.update_interrupt(value as u8);
                    }
                }

                gpio.update_interrupt();
            }

            PROC0_INTF0..=PROC0_INTF5 => {
                let index = (address - PROC0_INTF0) / 0x4;

                for i in 0u32..8 {
                    let pin_index = i as u16 + (index * 8);
                    let value_start = i * 4;
                    let value_end = i * 4 + 3;
                    let value = extract_bits(value, value_start..=value_end);

                    if let Some(pin) = gpio.get_pin_mut(pin_index as _) {
                        pin.interrupt_force = value as u8;
                    }
                }

                gpio.update_interrupt();
            }

            PROC0_INTS0..=PROC0_INTS5 => { /* read only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}
