/**
 * @file peripherals/ticks.rs
 * @author Nguyen Le Duy
 * @date 06/05/2025
 * @brief TickGen peripheral implementation
 * @todo actually implement the TickGen peripheral, this is just a hotfix to get the simulator
 * running
 */
use super::*;

pub const CTRL: u16 = 0x00;
pub const CYCLES: u16 = 0x04;
pub const COUNT: u16 = 0x08;

const TICK_DEV_OFFSET: u16 = 0x0c;

pub struct TickGen {
    ctrl: u32,
    cycles: u32,
}

impl Default for TickGen {
    fn default() -> Self {
        Self { ctrl: 0, cycles: 0 }
    }
}

#[derive(Default)]
pub struct Ticks {
    proc0: TickGen,
    proc1: TickGen,
    timer0: TickGen,
    timer1: TickGen,
    watchdog: TickGen,
    riscv: TickGen,
}

impl Peripheral for Ticks {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let ticks = match address / TICK_DEV_OFFSET {
            0 => &self.proc0,
            1 => &self.proc1,
            2 => &self.timer0,
            3 => &self.timer1,
            4 => &self.watchdog,
            5 => &self.riscv,
            _ => return Err(PeripheralError::OutOfBounds),
        };

        let value = match address % TICK_DEV_OFFSET {
            CTRL => ticks.ctrl | 0b10,
            CYCLES => ticks.cycles,
            COUNT => 0, // TODO
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
        let ticks = match address / TICK_DEV_OFFSET {
            0 => &mut self.proc0,
            1 => &mut self.proc1,
            2 => &mut self.timer0,
            3 => &mut self.timer1,
            4 => &mut self.watchdog,
            5 => &mut self.riscv,
            _ => return Err(PeripheralError::OutOfBounds),
        };

        match address % TICK_DEV_OFFSET {
            CTRL => ticks.ctrl = value & 1,
            CYCLES => ticks.cycles = value & 0xFF,
            COUNT => { /* read only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        };

        Ok(())
    }
}
