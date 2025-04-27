use super::*;

pub mod doorbell;
pub mod interpolator;
pub mod mailboxes;
pub mod spinlock;
pub mod timer;
pub mod tmds;

use doorbell::DoorBell;
use interpolator::Interpolator;
use mailboxes::Mailboxes;
use spinlock::SpinLock;
use std::cell::RefCell;
use std::rc::Rc;
use timer::RiscVPlatformTimer;
use crate::gpio::GpioController;
use tmds::TmdsEncoder;

#[derive(Default)]
pub struct Sio {
    mailboxes: RefCell<Mailboxes>,
    spinlock: SpinLock,
    doorbell: DoorBell,
    timer: [RiscVPlatformTimer; 2],
    interpolator0: [RefCell<Interpolator<0>>; 2],
    interpolator1: [RefCell<Interpolator<1>>; 2],
    tmds: [TmdsEncoder; 2],

    gpio_value: u32,
    gpio_output_enable: u32
}

impl Sio {
    pub fn new() -> Self {
        Self {
            mailboxes: RefCell::new(Mailboxes::default()),
            spinlock: SpinLock::default(),
            doorbell: DoorBell::default(),
            timer: [RiscVPlatformTimer::default(), RiscVPlatformTimer::default()],
            interpolator0: [Default::default(), Default::default()],
            interpolator1: [Default::default(), Default::default()],
            tmds: [TmdsEncoder::default(), TmdsEncoder::default()],
            gpio_value: 0,
            gpio_output_enable: 0,
        }
    }

    fn update_gpio(&self, gpio: Rc<RefCell<GpioController>>, old_gpio_value: u32, old_gpio_output_enable: u32) {
        let gpio = gpio.as_ref().borrow();

        let updated_pins = (self.gpio_value ^ old_gpio_value)
            | (self.gpio_output_enable ^ old_gpio_output_enable);

        if updated_pins == 0 {
            return
        }

        for (i, pin) in gpio.pins.iter().enumerate() {
            if (updated_pins & (1 << i)) != 0 {
                // TODO
            }
        }
    }
}


pub const CPUID: u16 = 0x000; // Processor core identifier
pub const GPIO_IN: u16 = 0x004; // Input value for GPIO0..31
pub const GPIO_HILIN: u16 = 0x008; // Input value on GPIO32..47, QSPI IOs and USB pins
pub const GPIO_OUT: u16 = 0x010; // GPIO0..31 output value
pub const GPIO_HILOUT: u16 = 0x014; // Output value for GPIO32..47, QSPI IOs and USB pins
pub const GPIO_OUT_SET: u16 = 0x018; // GPIO0..31 output value set
pub const GPIO_HILOUT_SET: u16 = 0x01C; // Output value set for GPIO32..47, QSPI IOs and USB pins
pub const GPIO_OUT_CLR: u16 = 0x020; // GPIO0..31 output value clear
pub const GPIO_HILOUT_CLR: u16 = 0x024; // Output value clear for GPIO32..47, QSPI IOs and USB pins
pub const GPIO_OUT_XOR: u16 = 0x028; // GPIO0..31 output value XOR
pub const GPIO_HLOUT_XOR: u16 = 0x02C; // Output value XOR for GPIO32..47, QSPI IOs and USB pins
pub const GPIO_OE: u16 = 0x030; // GPIO0..31 output enable
pub const GPIO_HI_OE: u16 = 0x034; // Output enable value for GPIO32..47, QSPI IOs and USB pins
pub const GPIO_OE_SET: u16 = 0x038; // GPIO0..31 output enable set
pub const GPIO_HI_OE_SET: u16 = 0x03C; // Output enable set for GPIO32..47, QSPI IOs and USB pins
pub const GPIO_OE_CLR: u16 = 0x040; // GPIO0..31 output enable clear
pub const GPIO_HI_OE_CLR: u16 = 0x044; // Output enable clear for GPIO32..47, QSPI IOs and USB pins
pub const GPIO_OE_XOR: u16 = 0x048; // GPIO0..31 output enable XOR
pub const GPIO_HI_OE_XOR: u16 = 0x04C; // Output enable XOR for GPIO32..47, QSPI IOs and USB pins
pub const FIFO_ST: u16 = 0x050; // Status register for inter-core FIFOs (mailboxes)
pub const FIFO_WR: u16 = 0x054; // Write access to this core's TX FIFO
pub const FIFO_RD: u16 = 0x058; // Read access to this core's RX FIFO
pub const SPINLOCK_ST: u16 = 0x05C; // Spinlock state
pub const INTERPO_ACCUM0: u16 = 0x080; // Read/write access to accumulator 0
pub const INTERPO_ACCUM1: u16 = 0x084; // Read/write access to accumulator 1
pub const INTERPO_BASE0: u16 = 0x088; // Read/write access to BASE0 register
pub const INTERPO_BASE1: u16 = 0x08C; // Read/write access to BASE1 register
pub const INTERPO_BASE2: u16 = 0x090; // Read/write access to BASE2 register
pub const INTERPO_POP_LANE0: u16 = 0x094; // Read LANE0 result, and simultaneously write lane results to both accumulators (POP)
pub const INTERPO_POP_LANE1: u16 = 0x098; // Read LANE1 result, and simultaneously write lane results to both accumulators (POP)
pub const INTERPO_POP_FULL: u16 = 0x09C; // Read FULL result, and simultaneously write lane results to both accumulators (POP)
pub const INTERPO_PEEK_LANE0: u16 = 0x0A0; // Read LANE0 result, without altering any internal state (PEEK)
pub const INTERPO_PEEK_LANE1: u16 = 0x0A4; // Read LANE1 result, without altering any internal state (PEEK)
pub const INTERPO_PEEK_FULL: u16 = 0x0A8; // Read FULL result, without altering any internal state (PEEK)
pub const INTERPO_CTRL_LANE0: u16 = 0x0AC; // Control register for lane 0
pub const INTERPO_CTRL_LANE1: u16 = 0x0B0; // Control register for lane 1
pub const INTERPO_ACCUM0_ADD: u16 = 0x0B4; // Values written here are atomically added to ACCUM0
pub const INTERPO_ACCUM1_ADD: u16 = 0x0B8; // Values written here are atomically added to ACCUM1
pub const INTERPO_BASE_1AND0: u16 = 0x0BC; // On write, the lower 16 bits go to BASE0, upper bits to BASE1 simultaneously
pub const INTERP1_ACCUM0: u16 = 0x0C0; // Read/write access to accumulator 0
pub const INTERP1_ACCUM1: u16 = 0x0C4; // Read/write access to accumulator 1
pub const INTERP1_BASE0: u16 = 0x0C8; // Read/write access to BASE0 register
pub const INTERP1_BASE1: u16 = 0x0CC; // Read/write access to BASE1 register
pub const INTERP1_BASE2: u16 = 0x0D0; // Read/write access to BASE2 register
pub const INTERP1_POP_LANE0: u16 = 0x0D4; // Read LANE0 result, and simultaneously write lane results to both accumulators (POP)
pub const INTERP1_POP_LANE1: u16 = 0x0D8; // Read LANE1 result, and simultaneously write lane results to both accumulators (POP)
pub const INTERP1_POP_FULL: u16 = 0x0DC; // Read FULL result, and simultaneously write lane results to both accumulators (POP)
pub const INTERP1_PEEK_LANE0: u16 = 0x0E0; // Read LANE0 result, without altering any internal state (PEEK)
pub const INTERP1_PEEK_LANE1: u16 = 0x0E4; // Read LANE1 result, without altering any internal state (PEEK)
pub const INTERP1_PEEK_FULL: u16 = 0x0E8; // Read FULL result, without altering any internal state (PEEK)
pub const INTERP1_CTRL_LANE0: u16 = 0x0EC; // Control register for lane 0
pub const INTERP1_CTRL_LANE1: u16 = 0x0F0; // Control register for lane 1
pub const INTERP1_ACCUM0_ADD: u16 = 0x0F4; // Values written here are atomically added to ACCUM0
pub const INTERP1_ACCUM1_ADD: u16 = 0x0F8; // Values written here are atomically added to ACCUM1
pub const INTERP1_BASE_1AND0: u16 = 0x0FC; // On write, the lower 16 bits go to BASE0, upper bits to BASE1 simultaneously
pub const SPINLOCK0: u16 = 0x100;
pub const SPINLOCK31: u16 = 0x17C;
pub const DOORBELL_OUT_SET: u16 = 0x180; // Trigger a doorbell interrupt on the opposite core.
pub const DOORBELL_OUT_CLR: u16 = 0x184; // Clear doorbells which have been posted to the opposite core
pub const DOORBELL_IN_SET: u16 = 0x188; // Write 1s to trigger doorbell interrupts on this core
pub const DOORBELL_IN_CLR: u16 = 0x18C; // Check and acknowledge doorbells posted to this core
pub const PERI_NONSEC: u16 = 0x190; // Detach certain core-local peripherals from Secure SIO
pub const RISCV_SOFTIRQ: u16 = 0x1A0; // Control the assertion of the standard software interrupt (MIP.MSIP) on the RISC-V cores
pub const MTIME_CTRL: u16 = 0x1A4; // Control register for the RISC-V 64-bit Machine-mode timer
pub const MTIME: u16 = 0x1B0; // Read/write access to the high half of RISC-V Machine-mode timer
pub const MTIMEH: u16 = 0x1B4; // Read/write access to the high half of RISC-V Machine-mode timer
pub const MTIMECMP: u16 = 0x1B8; // Low half of RISC-V Machine-mode timer comparator
pub const MTIMECMPH: u16 = 0x1BC; // High half of RISC-V Machine-mode timer comparator
pub const TMDS_CTRL: u16 = 0x1C0; // Control register for TMDS encoder
pub const TMDS_WDATA: u16 = 0x1C4; // Write-only access to the TMDS colour data register
pub const TMDS_PEEK_SINGLE: u16 = 0x1C8; // Get the encoding of one pixel's worth of colour data
pub const TMDS_POP_SINGLE: u16 = 0x1CC; // Get the encoding of one pixel's worth of colour data
pub const TMDS_PEEK_DOUBLE_L0: u16 = 0x1D0; // Get lane 0 of the encoding of two pixels' worth of colour data
pub const TMDS_POP_DOUBLE_L0: u16 = 0x1D4; // Get lane 0 of the encoding of two pixels' worth of colour data
pub const TMDS_PEEK_DOUBLE_L1: u16 = 0x1D8; // Get lane 1 of the encoding of two pixels' worth of colour data
pub const TMDS_POP_DOUBLE_L1: u16 = 0x1DC; // Get lane 1 of the encoding of two pixels' worth of colour data
pub const TMDS_PEEK_DOUBLE_L2: u16 = 0x1E0; // Get lane 2 of the encoding of two pixels' worth of colour data
pub const TMDS_POP_DOUBLE_L2: u16 = 0x1E4; // Get lane 2 of the encoding of two pixels' worth of colour data

impl Peripheral for Sio {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let mut interpolator0 = self.interpolator0[ctx.requestor as usize].borrow_mut();
        let mut interpolator1 = self.interpolator1[ctx.requestor as usize].borrow_mut();

        let value = match address {
            CPUID => match ctx.requestor {
                Requestor::Proc0 => 0,
                Requestor::Proc1 => 1,
                _ => return Err(PeripheralError::OutOfBounds),
            }

            GPIO_IN => {
                let gpio = ctx.gpio.as_ref().borrow();
                gpio.pins
                    .iter()
                    .map(|pin| pin.value.is_high() as u32)
                    .rev()
                    .fold(0, |acc, value| (acc << 1) | value)
            }
            GPIO_HILIN => 0, // TODO QSPI USB GPIO32..47
            GPIO_OUT => self.gpio_value,
            GPIO_HILOUT => 0, // TODO QSPI USB GPIO32..47
            GPIO_OE => self.gpio_output_enable,
            GPIO_HI_OE => 0, // TODO QSPI USB GPIO32..47

            FIFO_ST => self.mailboxes.borrow_mut().state(ctx.requestor),
            FIFO_RD => self.mailboxes.borrow_mut().read(ctx.requestor),
            FIFO_WR => return Err(PeripheralError::OutOfBounds),
            SPINLOCK_ST => self.spinlock.state(),
            SPINLOCK0..=SPINLOCK31 => {
                let index = (address - SPINLOCK0) / 4;
                self.spinlock.claim(index)
            }

            INTERPO_ACCUM0 => interpolator0.accum[0],
            INTERPO_ACCUM1 => interpolator0.accum[1],
            INTERPO_BASE0 => interpolator0.base[0],
            INTERPO_BASE1 => interpolator0.base[1],
            INTERPO_BASE2 => interpolator0.base[2],
            INTERPO_POP_LANE0 => {
                let value = interpolator0.result[0];
                interpolator0.writeback();
                value
            }
            INTERPO_POP_LANE1 => {
                let value = interpolator0.result[1];
                interpolator0.writeback();
                value
            }
            INTERPO_POP_FULL => {
                let value = interpolator0.result[2];
                interpolator0.writeback();
                value
            }
            INTERPO_PEEK_LANE0 => interpolator0.result[0],
            INTERPO_PEEK_LANE1 => interpolator0.result[1],
            INTERPO_PEEK_FULL => interpolator0.result[2],
            INTERPO_CTRL_LANE0 => interpolator0.ctrl[0],
            INTERPO_CTRL_LANE1 => interpolator0.ctrl[1],
            INTERPO_ACCUM0_ADD => interpolator0.sm_result[0],
            INTERPO_ACCUM1_ADD => interpolator0.sm_result[1],
            // INTERPO_BASE_1AND0 => 
            INTERP1_ACCUM0 => interpolator1.accum[0],
            INTERP1_ACCUM1 => interpolator1.accum[1],
            INTERP1_BASE0 => interpolator1.base[0],
            INTERP1_BASE1 => interpolator1.base[1],
            INTERP1_BASE2 => interpolator1.base[2],
            INTERP1_POP_LANE0 => {
                let value = interpolator1.result[0];
                interpolator1.writeback();
                value
            }
            INTERP1_POP_LANE1 => {
                let value = interpolator1.result[1];
                interpolator1.writeback();
                value
            }
            INTERP1_POP_FULL => {
                let value = interpolator1.result[2];
                interpolator1.writeback();
                value
            }
            INTERP1_PEEK_LANE0 => interpolator1.result[0],
            INTERP1_PEEK_LANE1 => interpolator1.result[1],
            INTERP1_PEEK_FULL => interpolator1.result[2],
            INTERP1_CTRL_LANE0 => interpolator1.ctrl[0],
            INTERP1_CTRL_LANE1 => interpolator1.ctrl[1],
            INTERP1_ACCUM0_ADD => interpolator1.sm_result[0],
            INTERP1_ACCUM1_ADD => interpolator1.sm_result[1],
            // INTERP1_BASE_1AND0

            | PERI_NONSEC  // TODO
            | RISCV_SOFTIRQ
            | MTIME_CTRL
            | MTIME
            | MTIMEH
            | MTIMECMP
            | MTIMECMPH
            | TMDS_CTRL
            | TMDS_WDATA
            | TMDS_PEEK_SINGLE
            | TMDS_POP_SINGLE
            | TMDS_PEEK_DOUBLE_L0
            | TMDS_POP_DOUBLE_L0
            | TMDS_PEEK_DOUBLE_L1
            | TMDS_POP_DOUBLE_L1
            | TMDS_PEEK_DOUBLE_L2
            | TMDS_POP_DOUBLE_L2 => 0, // TODO

            GPIO_OUT_SET  // Write only
            | GPIO_OUT_CLR
            | GPIO_HILOUT_CLR
            | GPIO_OUT_XOR
            | GPIO_HLOUT_XOR
            | GPIO_OE_SET
            | GPIO_HI_OE_SET
            | GPIO_OE_CLR
            | GPIO_HI_OE_CLR
            | GPIO_OE_XOR
            | GPIO_HI_OE_XOR
            | GPIO_HILOUT_SET => 0,
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
        let mut interpolator0 = self.interpolator0[ctx.requestor as usize].borrow_mut();
        let mut interpolator1 = self.interpolator1[ctx.requestor as usize].borrow_mut();

        let old_gpio_value = self.gpio_value;
        let old_gpio_output_enable = self.gpio_output_enable;

        match address {
            GPIO_OUT => {
                self.gpio_value = value;
            }
            GPIO_HILOUT => {
                // TODO QSPI USB GPIO32..47
            }
            GPIO_OUT_SET => {
                self.gpio_value |= value;
            }
            GPIO_HILOUT_SET => {
                // TODO QSPI USB GPIO32..47
            }
            GPIO_OUT_CLR => {
                self.gpio_value &= !value;
            }
            GPIO_HILOUT_CLR => {
                // TODO QSPI USB GPIO32..47
            }
            GPIO_OUT_XOR => {
                self.gpio_value ^= value;
            }
            GPIO_HLOUT_XOR => {
                // TODO QSPI USB GPIO32..47
            }
            GPIO_OE => {
                self.gpio_output_enable = value;
            }
            GPIO_HI_OE => {
                // TODO QSPI USB GPIO32..47
            }
            GPIO_OE_SET => {
                self.gpio_output_enable |= value;
            }
            GPIO_HI_OE_SET => {
                // TODO QSPI USB GPIO32..47
            }
            GPIO_OE_CLR => {
                self.gpio_output_enable &= !value;
            }
            GPIO_HI_OE_CLR => {
                // TODO QSPI USB GPIO32..47
            }
            GPIO_OE_XOR => {
                self.gpio_output_enable ^= value;
            }
            GPIO_HI_OE_XOR => {
                // TODO QSPI USB GPIO32..47
            }
            FIFO_ST => {
                if value & (1 << 2) != 0 {
                    self.mailboxes.borrow_mut().clear_wof(ctx.requestor);
                }

                if value & (1 << 3) != 0 {
                    self.mailboxes.borrow_mut().clear_roe(ctx.requestor);
                }
            }
            FIFO_WR => self.mailboxes.borrow_mut().write(value, ctx.requestor),
            FIFO_RD => return Err(PeripheralError::OutOfBounds),

            SPINLOCK0..=SPINLOCK31 => {
                let index = (address - SPINLOCK0) / 4;
                self.spinlock.release(index);
            }

            INTERPO_ACCUM0 => {
                interpolator0.accum[0] = value;
                interpolator0.update();
            }
            INTERPO_ACCUM1 => {
                interpolator0.accum[1] = value;
                interpolator0.update();
            }
            INTERPO_BASE0 => {
                interpolator0.base[0] = value;
                interpolator0.update();
            }
            INTERPO_BASE1 => {
                interpolator0.base[1] = value;
                interpolator0.update();
            }
            INTERPO_BASE2 => {
                interpolator0.base[2] = value;
                interpolator0.update();
            }
            INTERPO_CTRL_LANE0 => {
                interpolator0.ctrl[0] = value;
                interpolator0.update();
            }
            INTERPO_CTRL_LANE1 => {
                interpolator0.ctrl[1] = value;
                interpolator0.update();
            }
            INTERPO_ACCUM0_ADD => {
                interpolator0.accum[0] += value;
                interpolator0.update();
            }
            INTERPO_ACCUM1_ADD => {
                interpolator0.accum[1] += value;
                interpolator0.update();
            }
            INTERPO_BASE_1AND0 => {
                interpolator0.set_base01(value);
            }
            INTERP1_ACCUM0 => {
                interpolator1.accum[0] = value;
                interpolator1.update();
            }
            INTERP1_ACCUM1 => {
                interpolator1.accum[1] = value;
                interpolator1.update();
            }
            INTERP1_BASE0 => {
                interpolator1.base[0] = value;
                interpolator1.update();
            }
            INTERP1_BASE1 => {
                interpolator1.base[1] = value;
                interpolator1.update();
            }
            INTERP1_BASE2 => {
                interpolator1.base[2] = value;
                interpolator1.update();
            }
            INTERP1_CTRL_LANE0 => {
                interpolator1.ctrl[0] = value;
                interpolator1.update();
            }
            INTERP1_CTRL_LANE1 => {
                interpolator1.ctrl[1] = value;
                interpolator1.update();
            }
            INTERP1_ACCUM0_ADD => {
                interpolator1.accum[0] += value;
                interpolator1.update();
            }
            INTERP1_ACCUM1_ADD => {
                interpolator1.accum[1] += value;
                interpolator1.update();
            }
            INTERP1_BASE_1AND0 => {
                interpolator1.set_base01(value);
            }

            PERI_NONSEC // TODO
            | RISCV_SOFTIRQ
            | MTIME_CTRL
            | MTIME
            | MTIMEH
            | MTIMECMP
            | MTIMECMPH
            | TMDS_CTRL
            | TMDS_WDATA
            | TMDS_PEEK_SINGLE
            | TMDS_POP_SINGLE
            | TMDS_PEEK_DOUBLE_L0
            | TMDS_POP_DOUBLE_L0
            | TMDS_PEEK_DOUBLE_L1
            | TMDS_POP_DOUBLE_L1
            | TMDS_PEEK_DOUBLE_L2
            | TMDS_POP_DOUBLE_L2 => {}, // TODO
                                       
            CPUID // Read Only
            | GPIO_IN
            | GPIO_HILIN
            | SPINLOCK_ST 
            | INTERPO_POP_LANE0
            | INTERPO_POP_LANE1
            | INTERPO_POP_FULL
            | INTERPO_PEEK_LANE0
            | INTERPO_PEEK_LANE1
            | INTERPO_PEEK_FULL
            | INTERP1_POP_LANE0
            | INTERP1_POP_LANE1
            | INTERP1_POP_FULL
            | INTERP1_PEEK_LANE0
            | INTERP1_PEEK_LANE1
            | INTERP1_PEEK_FULL => { /* read-only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        }

        self.update_gpio(Rc::clone(&ctx.gpio), old_gpio_value, old_gpio_output_enable);

        Ok(())
    }

}
