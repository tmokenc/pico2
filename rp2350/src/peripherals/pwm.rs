use crate::clock::EventType;
use crate::utils::extract_bit;

use super::*;
use std::cell::RefCell;
use std::rc::Rc;

pub mod channel;

pub use channel::PwmChannel;

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
const NOF_CHANNEL: usize = 12;

#[derive(Default)]
pub struct Pwm {
    pub interrupts_mask: [u16; 2],
    pub interrupts_forced: [u16; 2],
    pub channels: [PwmChannel; NOF_CHANNEL],
}

impl Pwm {
    fn interrupt_raw(&self, is_wrap0: bool) -> u16 {
        let mut result = 0;
        for (i, channel) in self.channels.iter().enumerate() {
            if is_wrap0 {
                if channel.irq_wrap_0 {
                    result |= 1 << i;
                }
            } else {
                if channel.irq_wrap_1 {
                    result |= 1 << i;
                }
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

    fn update_interrupt(&mut self, interrupts: Rc<RefCell<Interrupts>>) {
        let irq0 =
            (self.interrupt_raw(false) & self.interrupts_mask[0]) & !self.interrupts_forced[0];
        let irq1 =
            (self.interrupt_raw(true) & self.interrupts_mask[1]) & !self.interrupts_forced[1];

        let mut global = interrupts.borrow_mut();
        global.set_irq(Interrupts::PWM_IRQ_WRAP_0, irq0 != 0);
        global.set_irq(Interrupts::PWM_IRQ_WRAP_1, irq1 != 0);
    }
}

fn start_channel(pwm_ref: Rc<RefCell<Pwm>>, channel: usize, ctx: &PeripheralAccessContext) {
    let pwm = pwm_ref.borrow();
    let is_channel_enabled = pwm.channels[channel].is_enabled();
    drop(pwm);

    let pwm = pwm_ref.clone();
    let clock = ctx.clock.clone();
    let dma = ctx.dma.clone();
    let interrupts = ctx.interrupts.clone();
    let inspector = ctx.inspector.clone();

    if is_channel_enabled {
        ctx.clock.schedule(0, EventType::Pwm(channel), move || {
            channel_update(pwm, channel, clock, dma, interrupts, inspector)
        });
    }
}

fn channel_update(
    pwm_ref: Rc<RefCell<Pwm>>,
    channel: usize,
    clock_ref: Rc<Clock>,
    dma_ref: Rc<RefCell<Dma>>,
    interrupts_ref: Rc<RefCell<Interrupts>>,
    inspector: InspectorRef,
) {
    // TODO interrupt

    let ticks = {
        let mut pwm = pwm_ref.borrow_mut();
        let ref mut channel = pwm.channels[channel];
        channel.advance();
        channel.next_update()
    };

    let pwm = pwm_ref.clone();
    let clock = clock_ref.clone();
    let dma = dma_ref.clone();
    let interrupts = interrupts_ref.clone();
    let inspector = inspector.clone();

    clock_ref.schedule(ticks, EventType::Pwm(channel), move || {
        channel_update(pwm, channel, clock, dma, interrupts, inspector)
    });
}

impl Peripheral for Rc<RefCell<Pwm>> {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
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
            INTR => (pwm.interrupt_raw(false) | pwm.interrupt_raw(true)) as u32,
            IRQ0_INTE => pwm.interrupts_mask[0] as u32,
            IRQ0_INTF => pwm.interrupts_forced[0] as u32,
            IRQ0_INTS => {
                ((pwm.interrupt_raw(false) & pwm.interrupts_mask[0]) | pwm.interrupts_forced[0])
                    as u32
            }
            IRQ1_INTE => pwm.interrupts_mask[1] as u32,
            IRQ1_INTF => pwm.interrupts_forced[1] as u32,
            IRQ1_INTS => {
                ((pwm.interrupt_raw(true) & pwm.interrupts_mask[1]) | pwm.interrupts_forced[1])
                    as u32
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
                let mut channel = pwm.channels[index as usize];

                match relative_offset {
                    CHN_CSR => channel.update_csr(value as u8),
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
                        start_channel(self.clone(), i, ctx);
                    } else {
                        pwm.channels[i].disable();
                        ctx.clock.cancel(EventType::Pwm(i));
                    }
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
            }
            IRQ0_INTF => {
                pwm.interrupts_forced[0] = (value as u16) & 0x0FFF;
            }
            IRQ1_INTE => {
                pwm.interrupts_mask[1] = (value as u16) & 0x0FFF;
            }
            IRQ1_INTF => {
                pwm.interrupts_forced[1] = (value as u16) & 0x0FFF;
            }

            IRQ0_INTS | IRQ1_INTS => { /* Read only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        }

        pwm.update_interrupt(ctx.interrupts.clone());

        Ok(())
    }
}
