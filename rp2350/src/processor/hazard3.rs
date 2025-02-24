mod exec;
// pub mod instruction;
pub(crate) mod csrs;
pub(crate) mod instruction_format;
pub(crate) mod registers;
pub(crate) mod trap;

use super::{CpuArchitecture, ProcessorContext, Stats};
use crate::bus::{LoadStatus, StoreStatus};
use core::mem;
use csrs::Csrs;
pub use csrs::PrivilegeMode;
use exec::{exec_instruction, ExecContext, MemoryAccess};
use instruction_format::*;
pub use registers::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use trap::*;

type RegisterWrite = (Register, u32);

#[derive(Debug, Default)]
pub(crate) enum State {
    Wfi,
    Stall(u8, RegisterWrite),
    BusWaitLoad(Register, Rc<RefCell<LoadStatus>>),
    BusWaitStore(Rc<RefCell<StoreStatus>>),
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
    pub(self) xx_bypass: Option<RegisterWrite>,
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

    fn tick(&mut self, ctx: &mut ProcessorContext) {
        // Value which was in X-X bypass is now written to register
        // since that instruction has done the M (memory) stage
        // which was 1 cycle behind the current instruction
        if let Some((rd, value)) = self.xx_bypass.take() {
            self.registers.write(rd, value);
        }

        match mem::take(&mut self.state) {
            State::Stall(cycles, reg_write) => {
                if cycles == 1 {
                    self.xx_bypass = Some(reg_write);
                } else {
                    self.state = State::Stall(cycles - 1, reg_write);
                }
            }
            State::BusWaitLoad(rd, load_status) => match *load_status.clone().borrow() {
                LoadStatus::Waiting => self.state = State::BusWaitLoad(rd, load_status),
                LoadStatus::Done(value) => {
                    self.registers.write(rd, value);
                }

                LoadStatus::Error(e) => {
                    log::warn!("Load error: {:?}", e);
                    self.trap_handle(Exception::LoadFault);
                }
            },
            State::BusWaitStore(store_status) => match *store_status.clone().borrow() {
                StoreStatus::Waiting => self.state = State::BusWaitStore(store_status),
                StoreStatus::Done => (),
                StoreStatus::Error(e) => {
                    log::warn!("Store error: {:?}", e);
                    self.trap_handle(Exception::StoreFault);
                }
            },
            State::Normal => {
                let Ok(inst_code) = ctx.bus.fetch(self.pc) else {
                    self.trap_handle(Exception::InstructionFetchFault);
                    return;
                };

                let mut exec_ctx = ExecContext::new(self, ctx.bus);
                exec_instruction(inst_code, &mut exec_ctx);

                let ExecContext {
                    exception,
                    register_write,
                    memory_access,
                    next_pc_offset,
                    cycles,
                    instruction_name,
                    ..
                } = exec_ctx;

                if self.monitor {
                    self.instruction_log(inst_code, instruction_name);
                }

                self.csrs.tick();

                if let Some(exception) = exception {
                    self.trap_handle(exception);
                } else {
                    self.pc = (self.pc as i32).wrapping_add(next_pc_offset) as u32;
                }

                if let Some(write) = register_write {
                    self.state = State::Stall(cycles, write);
                }

                match memory_access {
                    MemoryAccess::Load(reg, status) => {
                        self.state = State::BusWaitLoad(reg, status);
                    }
                    MemoryAccess::Store(status) => {
                        self.state = State::BusWaitStore(status);
                    }
                    MemoryAccess::None => (),
                }

                // due to the way CSRs are handled differently here
                return;
            }
            State::Wfi => {
                // TODO
                self.state = State::Wfi;
            }
        };

        self.csrs.tick();
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
    fn trap_handle(&mut self, trap: impl Into<Trap>) {
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
    use crate::bus::Bus;
    use crate::interrupts::Interrupts;

    const SRAM: u32 = 0x2000_0000;

    macro_rules! setup {
        ($cpu:tt, $ctx:tt) => {
            let mut $cpu = Hazard3::default();
            let mut bus = Bus::default();
            let mut interrupts = Interrupts::default();

            $cpu.set_pc(SRAM);

            let mut $ctx = ProcessorContext {
                bus: &mut bus,
                interrupts: &mut interrupts,
            };
        };
    }

    #[test]
    fn test_lui_and_delayed_M_stage() {
        setup!(cpu, ctx);
        ctx.bus.sram.write_u32(0, 0x0ffff0b7); // lui x1, 65535
        cpu.registers.x[1] = 0x1234;
        cpu.tick(&mut ctx);
        assert_eq!(cpu.pc, SRAM + 4);
        assert_eq!(cpu.registers.x[1], 0x1234);
        assert!(cpu.xx_bypass.is_none());
        cpu.tick(&mut ctx);
        assert_eq!(cpu.registers.x[1], 0x1234);
        assert!(cpu.xx_bypass.is_some());
        cpu.tick(&mut ctx);
        assert!(cpu.xx_bypass.is_none());
        assert_eq!(cpu.registers.x[1], 0xffff << 12);
    }

    #[test]
    fn test_write_x0() {
        setup!(cpu, ctx);
        ctx.bus.sram.write_u32(0, 0x0ffff037); // lui x0, 65535
        cpu.tick(&mut ctx);
        assert_eq!(cpu.registers.x[0], 0);
        assert!(cpu.xx_bypass.is_none());
        cpu.tick(&mut ctx);
        assert_eq!(cpu.registers.x[0], 0);
        assert!(cpu.xx_bypass.is_some());
        cpu.tick(&mut ctx);
        assert!(cpu.xx_bypass.is_none());
        assert_eq!(cpu.registers.x[0], 0);
    }
}
