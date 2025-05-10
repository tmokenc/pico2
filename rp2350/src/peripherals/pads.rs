/**
 * @file peripherals/pads.rs
 * @author Nguyen Le Duy
 * @date 03/05/2025
 * @brief Pads peripheral for the RP2350
 */
use crate::utils::extract_bit;

use super::*;

const VOLTAGE_SELECT: u16 = 0x00;
const GPIO_START: u16 = 0x04;
const GPIO_END: u16 = 0xc0;
const SWCLK: u16 = 0xc4;
const SWD: u16 = 0xc8;

const GPIO_STEP: u16 = 0x04;

#[derive(Default, Clone, Copy)]
pub enum Voltage {
    #[default]
    _3V3 = 0,
    _1V8 = 1,
}

#[derive(Default)]
pub struct PadsBank0 {
    voltage: Voltage,
    swclk: u32, // TODO
    swd: u32,   // TODO
}

impl Peripheral for PadsBank0 {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        log::info!("PadsBank0::read: address=0x{:04x}", address,);
        let value = match address {
            GPIO_START..=GPIO_END => {
                let index = (address - GPIO_START) / GPIO_STEP;
                let gpio = ctx.gpio.borrow();
                gpio.get_pin(index as _)
                    .map(|pin| pin.pad)
                    .unwrap_or_default()
            }
            VOLTAGE_SELECT => self.voltage as u32,
            SWCLK => self.swclk,
            SWD => self.swd,
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
        match address {
            GPIO_START..=GPIO_END => {
                let index = (address - GPIO_START) / GPIO_STEP;
                ctx.gpio.borrow_mut().update_pin_pads(index as _, value);
            }
            VOLTAGE_SELECT => {
                self.voltage = match extract_bit(value, 0) {
                    0 => Voltage::_3V3,
                    1 => Voltage::_1V8,
                    _ => unreachable!(),
                };
            }
            SWCLK => self.swclk = value,
            SWD => self.swd = value,
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}
