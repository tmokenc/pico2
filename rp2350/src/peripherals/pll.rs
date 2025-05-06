use crate::utils::extract_bit;

/**
 * @file peripherals/pll.rs
 * @author Nguyen Le Duy
 * @date 06/05/2025
 * @brief PLL peripheral implementation
 * @todo actually implement the PLL peripheral, this is just a hotfix to get the simulator running
 * @todo this actually generates interrupts, but we don't have a way to handle them yet
 */
use super::*;

pub const CS: u16 = 0x00;
pub const PWR: u16 = 0x04;
pub const FBDIV_INT: u16 = 0x08;
pub const PRIM: u16 = 0x0C;
pub const INTR: u16 = 0x10;
pub const INTE: u16 = 0x14;
pub const INTF: u16 = 0x18;
pub const INTS: u16 = 0x1c;

#[derive(Debug)]
// IDX 0 for PLL_SYS, 1 for PLL_USB
pub struct Pll<const IDX: usize> {
    cs: u32,
    pwr: u32,
    fbdiv_int: u32,
    prim: u32,

    interrupt_raw: bool,
    interrupt_enabled: bool,
    interrupt_force: bool,
    // TDOO
}

impl<const IDX: usize> Default for Pll<IDX> {
    fn default() -> Self {
        Self {
            cs: 1 | 1 << 31,
            pwr: 0b101101,
            fbdiv_int: 0,
            prim: (0x7 << 12) | (0x7 << 16),

            interrupt_raw: false,
            interrupt_enabled: false,
            interrupt_force: false,
        }
    }
}

impl<const IDX: usize> Pll<IDX> {
    fn interrupt_status(&self) -> bool {
        (self.interrupt_raw && self.interrupt_enabled) || self.interrupt_force
    }

    fn update_interrupt(&self, interrupts: Rc<RefCell<Interrupts>>) {
        let status = self.interrupt_status();
        let irq = if IDX == 0 {
            Interrupts::PLL_SYS_IRQ
        } else {
            Interrupts::PLL_USB_IRQ
        };
        interrupts.borrow_mut().set_irq(irq, status);
    }
}

impl<const IDX: usize> Peripheral for Pll<IDX> {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let value = match address {
            CS => self.cs,
            PWR => self.pwr,
            FBDIV_INT => self.fbdiv_int,
            PRIM => self.prim,
            INTR => self.interrupt_raw as u32,
            INTE => self.interrupt_enabled as u32,
            INTF => self.interrupt_force as u32,
            INTS => self.interrupt_status() as u32,
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
            CS => {
                if extract_bit(value, 30) == 1 {
                    // unlock the pll if locked, maybe in future usages???
                }

                let value = value & !((1 << 31) | (1 << 30));
                self.cs = value | (1 << 31); // lock
            }
            PWR => self.pwr = value & 0b101101,
            FBDIV_INT => self.pwr = value & 0xFFF,
            PRIM => self.prim = value & ((0b111 << 16) | (0b111 << 12)),
            INTR => {
                if extract_bit(value, 0) == 1 {
                    self.interrupt_raw = false;
                    self.update_interrupt(ctx.interrupts.clone());
                }
            }
            INTE => {
                self.interrupt_enabled = extract_bit(value, 0) == 1;
                self.update_interrupt(ctx.interrupts.clone());
            }
            INTF => {
                self.interrupt_force = extract_bit(value, 0) == 1;
                self.update_interrupt(ctx.interrupts.clone());
            }
            INTS => {
                // read only
            }
            _ => return Err(PeripheralError::OutOfBounds),
        };
        Ok(())
    }
}
