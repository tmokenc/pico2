mod exec;
// pub mod instruction;
pub(crate) mod csrs;
pub(crate) mod instruction_format;
pub(crate) mod registers;
pub(crate) mod trap;

use super::CpuArchitecture;
use super::Stats;
use crate::bus::Bus;
use csrs::Csrs;
use exec::{exec_instruction, ExecContext};

use instruction_format::*;
pub use registers::*;
use std::collections::HashMap;
use trap::*;

type RegisterWrite = (u8, u32);

#[derive(Default, PartialEq, Eq)]
pub(crate) enum PrivilegeMode {
    Machine,
    #[default]
    User,
}

#[derive(Default)]
pub(crate) enum State {
    Wfi,
    Stall(u32, RegisterWrite),
    BusWait,
    #[default]
    Normal,
}

#[derive(Default)]
pub struct Hazard3 {
    pub(self) pc: u32,
    pub(self) state: State,
    pub(self) registers: Registers,
    pub(self) privilege_mode: PrivilegeMode,
    pub(self) monitor: bool,
    pub(self) csrs: Csrs,
    pub count_instructions: Option<HashMap<&'static str, u32>>,
}

impl CpuArchitecture for Hazard3 {
    fn set_core_id(&mut self, core_id: u8) {
        self.csrs.core_id = core_id;
    }

    fn get_pc(&self) -> u32 {
        self.pc
    }

    fn set_pc(&mut self, value: u32) {
        self.pc = value;
    }

    fn tick(&mut self, bus: &mut Bus) {
        let inst_code = match bus.fetch(self.pc) {
            Ok(inst_code) => inst_code,
            Err(e) => {
                todo!()
            }
        };

        let mut ctx = ExecContext::new(self, bus);

        exec_instruction(inst_code, &mut ctx);
        todo!();
        self.csrs.tick()
    }

    fn stats(&self) -> &Stats {
        todo!()
    }
}

impl Hazard3 {
    /// RP2350 specification section 3.8.4
    /// Hardware performs the following steps automatically and atomically when entering a trap:
    ///    1. Save the address of the interrupted or excepting instruction to MEPC
    ///    2. Set the MSB of MCAUSE to indicate the cause is an interrupt, or clear it to indicate an exception
    ///    3. Write the detailed trap cause to the LSBs of the MCAUSE register
    ///    4. Save the current privilege level to MSTATUS.MPP
    ///    5. Set the privilege to M-mode (note Hazard3 does not implement S-mode)
    ///    6. Save the current value of MSTATUS.MIE to MSTATUS.MPIE
    ///    7. Disable interrupts by clearing MSTATUS.MIE
    ///    8. Jump to the correct offset from MTVEC depending on the trap cause
    fn trap_handle(&mut self, trap: Trap) {
        todo!()
    }

    pub(self) fn instruction_log(&mut self, inst_code: u32, name: &'static str) {
        if let Some(ref mut count) = self.count_instructions {
            *count.entry(name).or_insert(0) += 1;
        }

        log::info!("0x{:08x}: 0x{:08x}: {}", self.pc, inst_code, name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SRAM: u32 = 0x2000_0000;

    #[test]
    fn test_lui_and_delayed_M_stage() {
        let mut cpu = Hazard3::default();
        let mut bus = Bus::default();
        cpu.set_pc(SRAM);
        bus.sram.write_u32(SRAM, 0x0ffff0b7); // lui x1, 65535
        cpu.registers.x[1] = 0x1234;
        cpu.tick(&mut bus);
        assert_eq!(cpu.registers.x[1], 0x1234);
        cpu.tick(&mut bus);
        assert_eq!(cpu.registers.read(0), 0xffff0000);
    }

    #[test]
    fn test_write_x0() {
        let mut cpu = Hazard3::default();
        let mut bus = Bus::default();
        cpu.set_pc(SRAM);
        bus.sram.write_u32(SRAM, 0x0ffff037); // lui x0, 65535
        cpu.tick(&mut bus);
        cpu.tick(&mut bus);
        assert_eq!(cpu.registers.x[0], 0);
    }
}
