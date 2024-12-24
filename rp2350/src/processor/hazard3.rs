mod exec;
pub mod instruction;
pub(crate) mod instruction_format;
pub(crate) mod registers;
pub(crate) mod trap;

use super::CpuArchitecture;
use super::Stats;
pub use instruction::*;
pub use instruction_format::*;
pub use registers::*;
pub use trap::*;

#[derive(Default)]
pub(self) enum PrivilegeMode {
    Machine,
    #[default]
    User,
}

#[derive(Default)]
pub struct Hazard3 {
    pc: u32,
    registers: Registers,
    privilege_mode: PrivilegeMode,
    monitor: bool,
}

impl CpuArchitecture for Hazard3 {
    fn get_pc(&self) -> u32 {
        self.pc
    }

    fn set_pc(&mut self, value: u32) {
        self.pc = value;
    }

    fn tick(&mut self) {
        todo!()
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
}
