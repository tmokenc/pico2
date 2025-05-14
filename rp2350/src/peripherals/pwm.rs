/**
 * @file peripherals/pwm.rs
 * @author Nguyen Le Duy
 * @date 22/04/2025
 * @brief PWM peripheral implementation
 */
use crate::utils::extract_bit;

use self::channel::{channel_as_function_select, DivMode};

use super::*;
use std::cell::RefCell;
use std::rc::Rc;

pub mod channel;
mod schedule;

pub use channel::PwmChannel;
pub use schedule::channel_b_update;
use schedule::{start_channel, stop_channel};

pub const CHN_CSR: u16 = 0x000; // Control and status register
pub const CHN_DIV: u16 = 0x004; // INT and FRAC form a fixed-point fractional number
pub const CHN_CTR: u16 = 0x008; // Direct access to the PWM counter
pub const CHN_CC: u16 = 0x00C; // Counter compare values
pub const CHN_TOP: u16 = 0x010; // Counter wrap value

pub const EN: u16 = 0x0F0; // This register aliases the CSR_EN bits for all channels
pub const INTR: u16 = 0x0F4; // Raw Interrupts
pub const IRQ0_INTE: u16 = 0x0F8; // Interrupt Enable for irq0
pub const IRQ0_INTF: u16 = 0x0FC; // Interrupt Force for irq0
pub const IRQ0_INTS: u16 = 0x100; // Interrupt status after masking & forcing for irq0
pub const IRQ1_INTE: u16 = 0x104; // Interrupt Enable for irq1
pub const IRQ1_INTF: u16 = 0x108; // Interrupt Force for irq1
pub const IRQ1_INTS: u16 = 0x10C; // Interrupt status after masking & forcing for irq1

const CHANNEL_OFFSET: u16 = 0x014; // Offset to the next channel
pub const NOF_CHANNEL: usize = 12;

#[derive(Default)]
pub struct Pwm {
    pub interrupts_mask: [u16; 2],
    pub interrupts_forced: [u16; 2],
    pub channels: [PwmChannel; NOF_CHANNEL],
}

impl Pwm {
    fn interrupt_raw(&self) -> u16 {
        let mut result = 0;
        for (i, channel) in self.channels.iter().enumerate() {
            if channel.is_interrupting() {
                result |= 1 << i;
            }
        }
        result
    }

    fn enable_status(&self) -> u32 {
        let mut result = 0;
        for (i, channel) in self.channels.iter().enumerate() {
            if channel.is_enabled() {
                result |= 1 << i;
            }
        }
        result
    }

    pub(self) fn update_gpio(&mut self, gpio: Rc<RefCell<GpioController>>, channel_idx: usize) {
        let channel = &self.channels[channel_idx];
        let mut gpio = gpio.borrow_mut();

        for i in 0..8 {
            // only 8 has gpio out
            let mut a_output_enable = false;
            let mut b_output_enable = false;

            if channel.is_enabled() {
                a_output_enable = true;
                if !channel.divmode().is_channel_b_input() {
                    b_output_enable = true;
                }
            }

            let (func_a, func_b) = channel_as_function_select(i);

            gpio.set_pin_output_enable(func_a, a_output_enable);
            gpio.set_pin_output_enable(func_b, b_output_enable);
            gpio.set_pin_output(func_a, channel.output_a());
            gpio.set_pin_output(func_b, channel.output_b());
        }
    }

    pub(self) fn update_interrupt(&mut self, interrupts: Rc<RefCell<Interrupts>>) {
        let irq0 = (self.interrupt_raw() & self.interrupts_mask[0]) & !self.interrupts_forced[0];
        let irq1 = (self.interrupt_raw() & self.interrupts_mask[1]) & !self.interrupts_forced[1];

        let mut global = interrupts.borrow_mut();
        global.set_irq(Interrupts::PWM_IRQ_WRAP_0, irq0 != 0);
        global.set_irq(Interrupts::PWM_IRQ_WRAP_1, irq1 != 0);
    }
}

impl Peripheral for Rc<RefCell<Pwm>> {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        log::info!("PWM read: {:#x}", address);
        let pwm = self.borrow();

        let value = match address {
            0..=0x0ec => {
                let index = address / CHANNEL_OFFSET;
                let relative_offset = address % CHANNEL_OFFSET;
                let channel = pwm.channels[index as usize];

                match relative_offset {
                    CHN_CSR => channel.csr as u32,
                    CHN_DIV => channel.div as u32,
                    CHN_CTR => channel.ctr as u32,
                    CHN_CC => channel.cc as u32,
                    CHN_TOP => channel.top as u32,
                    _ => return Err(PeripheralError::OutOfBounds),
                }
            }
            EN => pwm.enable_status(),
            INTR => pwm.interrupt_raw() as u32,
            IRQ0_INTE => pwm.interrupts_mask[0] as u32,
            IRQ0_INTF => pwm.interrupts_forced[0] as u32,
            IRQ0_INTS => {
                ((pwm.interrupt_raw() & pwm.interrupts_mask[0]) | pwm.interrupts_forced[0]) as u32
            }
            IRQ1_INTE => pwm.interrupts_mask[1] as u32,
            IRQ1_INTF => pwm.interrupts_forced[1] as u32,
            IRQ1_INTS => {
                ((pwm.interrupt_raw() & pwm.interrupts_mask[1]) | pwm.interrupts_forced[1]) as u32
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
        let mut pwm = self.borrow_mut();

        match address {
            0..=0x0ec => {
                let index = address / CHANNEL_OFFSET;
                let relative_offset = address % CHANNEL_OFFSET;
                let ref mut channel = pwm.channels[index as usize];

                match relative_offset {
                    CHN_CSR => {
                        channel.update_csr(value as u8);
                        let is_channel_enabled = channel.is_enabled();
                        pwm.update_gpio(ctx.gpio.clone(), index as usize);
                        if is_channel_enabled {
                            drop(pwm);
                            start_channel(
                                self.clone(),
                                index as usize,
                                ctx.clock.clone(),
                                ctx.gpio.clone(),
                                ctx.interrupts.clone(),
                                ctx.inspector.clone(),
                            );
                        } else {
                            stop_channel(index as usize, ctx.clock.clone());
                        }
                    }
                    CHN_DIV => channel.div = value as u16,
                    CHN_CTR => channel.ctr = value as u16,
                    CHN_CC => channel.cc = value as u32,
                    CHN_TOP => channel.top = value as u16,
                    _ => return Err(PeripheralError::OutOfBounds),
                }
            }
            EN => {
                for i in 0..NOF_CHANNEL {
                    if extract_bit(value, i as u32) == 1 {
                        pwm.channels[i].enable();
                        start_channel(
                            self.clone(),
                            i,
                            ctx.clock.clone(),
                            ctx.gpio.clone(),
                            ctx.interrupts.clone(),
                            ctx.inspector.clone(),
                        );
                    } else {
                        pwm.channels[i].disable();
                        stop_channel(i, ctx.clock.clone());
                    }

                    pwm.update_gpio(ctx.gpio.clone(), i);
                }
            }
            INTR => {
                for i in 0..NOF_CHANNEL {
                    if extract_bit(value, i as u32) == 1 {
                        pwm.channels[i].clear_interrupt();
                    }
                }
            }
            IRQ0_INTE => {
                pwm.interrupts_mask[0] = (value as u16) & 0x0FFF;
                pwm.update_interrupt(ctx.interrupts.clone());
            }
            IRQ0_INTF => {
                pwm.interrupts_forced[0] = (value as u16) & 0x0FFF;
                pwm.update_interrupt(ctx.interrupts.clone());
            }
            IRQ1_INTE => {
                pwm.interrupts_mask[1] = (value as u16) & 0x0FFF;
                pwm.update_interrupt(ctx.interrupts.clone());
            }
            IRQ1_INTF => {
                pwm.interrupts_forced[1] = (value as u16) & 0x0FFF;
                pwm.update_interrupt(ctx.interrupts.clone());
            }

            IRQ0_INTS | IRQ1_INTS => { /* Read only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}
