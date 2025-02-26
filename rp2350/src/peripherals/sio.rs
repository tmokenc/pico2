use super::*;

pub mod doorbell;
pub mod gpio;
pub mod interpolator;
pub mod mailboxes;
pub mod spinlock;
pub mod timer;
pub mod tmds;

use doorbell::DoorBell;
use gpio::Gpio;
use interpolator::Interpolator;
use mailboxes::Mailboxes;
use spinlock::SpinLock;
use std::cell::RefCell;
use std::rc::Rc;
use timer::RiscVPlatformTimer;
use tmds::TmdsEncoder;

#[derive(Default)]
pub struct Sio {
    mailboxes: Rc<RefCell<Mailboxes>>,
    spinlock: SpinLock,
    doorbell: DoorBell,
    timer: [RiscVPlatformTimer; 2],
    gpio: Gpio,
    interpolator: [Interpolator; 4],
    tmds: [TmdsEncoder; 2],
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
        let value = match address {
            CPUID => match ctx.requestor {
                Requestor::Proc0 => 0,
                Requestor::Proc1 => 1,
                _ => return Err(PeripheralError::OutOfBounds),
            },

            FIFO_ST => self.mailboxes.borrow_mut().state(ctx.requestor),
            FIFO_RD => self.mailboxes.borrow_mut().read(ctx.requestor),
            FIFO_WR => return Err(PeripheralError::OutOfBounds),
            SPINLOCK_ST => self.spinlock.state(),
            SPINLOCK0..=SPINLOCK31 => {
                let index = (address - SPINLOCK0) / 4;
                self.spinlock.claim(index)
            }

            GPIO_IN // TODO
            | GPIO_HILIN
            | GPIO_OUT
            | GPIO_HILOUT
            | GPIO_OUT_SET
            | GPIO_HILOUT_SET
            | GPIO_OUT_CLR
            | GPIO_HILOUT_CLR
            | GPIO_OUT_XOR
            | GPIO_HLOUT_XOR
            | GPIO_OE
            | GPIO_HI_OE
            | GPIO_OE_SET
            | GPIO_HI_OE_SET
            | GPIO_OE_CLR
            | GPIO_HI_OE_CLR
            | GPIO_OE_XOR
            | GPIO_HI_OE_XOR
            | INTERPO_ACCUM0
            | INTERPO_ACCUM1
            | INTERPO_BASE0
            | INTERPO_BASE1
            | INTERPO_BASE2
            | INTERPO_POP_LANE0
            | INTERPO_POP_LANE1
            | INTERPO_POP_FULL
            | INTERPO_PEEK_LANE0
            | INTERPO_PEEK_LANE1
            | INTERPO_PEEK_FULL
            | INTERPO_CTRL_LANE0
            | INTERPO_CTRL_LANE1
            | INTERPO_ACCUM0_ADD
            | INTERPO_ACCUM1_ADD
            | INTERPO_BASE_1AND0
            | INTERP1_ACCUM0
            | INTERP1_ACCUM1
            | INTERP1_BASE0
            | INTERP1_BASE1
            | INTERP1_BASE2
            | INTERP1_POP_LANE0
            | INTERP1_POP_LANE1
            | INTERP1_POP_FULL
            | INTERP1_PEEK_LANE0
            | INTERP1_PEEK_LANE1
            | INTERP1_PEEK_FULL
            | INTERP1_CTRL_LANE0
            | INTERP1_CTRL_LANE1
            | INTERP1_ACCUM0_ADD
            | INTERP1_ACCUM1_ADD
            | INTERP1_BASE_1AND0
            | PERI_NONSEC
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

            GPIO_IN // TODO
            | GPIO_HILIN
            | GPIO_OUT
            | GPIO_HILOUT
            | GPIO_OUT_SET
            | GPIO_HILOUT_SET
            | GPIO_OUT_CLR
            | GPIO_HILOUT_CLR
            | GPIO_OUT_XOR
            | GPIO_HLOUT_XOR
            | GPIO_OE
            | GPIO_HI_OE
            | GPIO_OE_SET
            | GPIO_HI_OE_SET
            | GPIO_OE_CLR
            | GPIO_HI_OE_CLR
            | GPIO_OE_XOR
            | GPIO_HI_OE_XOR
            | INTERPO_ACCUM0
            | INTERPO_ACCUM1
            | INTERPO_BASE0
            | INTERPO_BASE1
            | INTERPO_BASE2
            | INTERPO_POP_LANE0
            | INTERPO_POP_LANE1
            | INTERPO_POP_FULL
            | INTERPO_PEEK_LANE0
            | INTERPO_PEEK_LANE1
            | INTERPO_PEEK_FULL
            | INTERPO_CTRL_LANE0
            | INTERPO_CTRL_LANE1
            | INTERPO_ACCUM0_ADD
            | INTERPO_ACCUM1_ADD
            | INTERPO_BASE_1AND0
            | INTERP1_ACCUM0
            | INTERP1_ACCUM1
            | INTERP1_BASE0
            | INTERP1_BASE1
            | INTERP1_BASE2
            | INTERP1_POP_LANE0
            | INTERP1_POP_LANE1
            | INTERP1_POP_FULL
            | INTERP1_PEEK_LANE0
            | INTERP1_PEEK_LANE1
            | INTERP1_PEEK_FULL
            | INTERP1_CTRL_LANE0
            | INTERP1_CTRL_LANE1
            | INTERP1_ACCUM0_ADD
            | INTERP1_ACCUM1_ADD
            | INTERP1_BASE_1AND0
            | PERI_NONSEC
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
                                       
            CPUID | SPINLOCK_ST => { /* read-only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}
