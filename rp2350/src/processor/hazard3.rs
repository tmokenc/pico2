/**
 * @file processor/hazard3.rs
 * @author Nguyen Le Duy
 * @date 02/01/2025
 * @brief Hazard3 processor implementation.
 */
pub mod branch_predictor;
pub mod csrs;
mod exec;
pub(crate) mod instruction_format;
pub mod registers;
pub mod trap;

use super::{CpuArchitecture, ProcessorContext};
use crate::bus::{BusAccessContext, LoadStatus, StoreStatus};
use crate::{common::*, InspectionEvent};
use branch_predictor::BranchPredictor;
use core::mem;
use csrs::Csrs;
pub use csrs::PrivilegeMode;
use exec::*;
pub use registers::*;
use std::cell::RefCell;
use std::rc::Rc;
use trap::*;

type RegisterWrite = (Register, u32);

#[derive(Default)]
pub enum State {
    Wfi,
    Stall(u8, RegisterWrite),
    BusWaitLoad(Register, Rc<RefCell<LoadStatus>>),
    BusWaitStore(Rc<RefCell<StoreStatus>>),
    Sleep(Box<State>),

    // Atomic instructions
    Atomic {
        rd: Register,
        bus_state: Rc<RefCell<LoadStatus>>,
        address: u32,
        value: u32,
        op: AtomicOp,
    },

    #[default]
    Normal,
}

impl PartialEq for State {
    // Just need to compare the variant, not the contents
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (State::Wfi, State::Wfi) => true,
            (State::Stall(_, _), State::Stall(_, _)) => true,
            (State::BusWaitLoad(_, _), State::BusWaitLoad(_, _)) => true,
            (State::BusWaitStore(_), State::BusWaitStore(_)) => true,
            (State::Atomic { .. }, State::Atomic { .. }) => true,
            (State::Normal, State::Normal) => true,
            (State::Sleep(_), State::Sleep(_)) => true,
            _ => false,
        }
    }
}

impl Eq for State {}

pub struct Hazard3 {
    pub pc: u32,
    pub state: State,
    pub registers: Registers,
    pub csrs: Csrs,
    pub xx_bypass: Option<RegisterWrite>,
    pub branch_predictor: BranchPredictor,

    // for atomic instructions
    // should be clear after any atomic instruction, or SC.W or getting a trap
    pub local_monitor_bit: bool,

    // Zcmp extension
    // Some instructions may expand into a sequence of multiple instructions
    pub(self) inst_seq: InstructionSequence,
}

impl Hazard3 {
    pub fn new() -> Self {
        Self {
            pc: 0x7642, // entry point for the RISC-V bootloader
            state: State::default(),
            registers: Registers::default(),
            csrs: Csrs::default(),
            xx_bypass: None,
            local_monitor_bit: false,
            branch_predictor: BranchPredictor::default(),
            inst_seq: InstructionSequence::default(),
        }
    }
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

    fn set_sp(&mut self, value: u32) {
        self.registers.write(2, value);
    }

    fn tick(&mut self, ctx: &mut ProcessorContext) {
        if let State::Sleep(_) = self.state {
            return;
        }

        // Value which was in X-X bypass is now written to register
        // since that instruction has done the M (memory) stage
        // which was 1 cycle behind the current instruction
        if let Some((rd, value)) = self.xx_bypass.take() {
            self.registers.write(rd, value);
        }

        self.update_state(ctx);

        if self.state != State::Normal {
            // The processor is in a state where it is waiting for something
            self.csrs.tick();
            return;
        }

        if !self.inst_seq.is_empty() {
            // Execute the next instruction in the sequence
            self.exec_next_instruction_sequence(ctx);
            self.csrs.tick();
            return;
        }

        // IRQ check before executing the next instruction
        if let Some(new_pc) = self.csrs.interrupt_check(self.pc, ctx.interrupts.clone()) {
            self.pc = new_pc;
            self.state = State::Normal;
            self.csrs.tick();
            return;
        }

        // Fetch the next instruction
        let Ok(inst_code) = ctx.bus.fetch(self.pc) else {
            self.trap_handle(Exception::InstructionFetchFault);
            return;
        };

        let mut exec_ctx = ExecContext::new(self, ctx.bus);
        exec_instruction(inst_code, &mut exec_ctx);
        exec_ctx.finalize();

        let ExecContext {
            exception,
            register_write,
            memory_access,
            next_pc,
            cycles,
            zcmp_actions,
            instruction_name,
            wake_opposite_core,
            ..
        } = exec_ctx;

        ctx.inspector.emit(InspectionEvent::ExecutedInstruction {
            core: self.csrs.core_id,
            instruction: inst_code,
            address: self.pc,
            name: instruction_name,
            operands: Vec::new(), // TODO
        });

        self.csrs.tick();
        self.csrs.count_instret();

        if let Some(exception) = exception {
            ctx.inspector.emit(InspectionEvent::Exception {
                core: self.csrs.core_id,
                exception: exception as u32,
            });

            return self.trap_handle(exception);
        } else {
            self.pc = next_pc;
        }

        ctx.wake_opposite_core = wake_opposite_core;

        if !zcmp_actions.is_empty() {
            self.inst_seq = zcmp_actions;
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
            MemoryAccess::Atomic {
                rd,
                bus_state,
                address,
                value,
                op,
            } => {
                self.state = State::Atomic {
                    rd,
                    bus_state,
                    address,
                    value,
                    op,
                };
            }
            MemoryAccess::None => (),
        }
    }

    fn sleep(&mut self) {
        let last_state = mem::take(&mut self.state);
        self.state = State::Sleep(Box::new(last_state));
    }

    fn wake(&mut self) {
        if let State::Sleep(state) = mem::take(&mut self.state) {
            self.state = *state;
        }
    }
}

impl Hazard3 {
    fn trap_handle(&mut self, trap: impl Into<Trap>) {
        self.csrs.trap_handle(trap, self.pc);
    }

    fn update_state(&mut self, ctx: &mut ProcessorContext) {
        match mem::take(&mut self.state) {
            State::Sleep(_state) => {}
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

                LoadStatus::ExclusiveDone(value) => {
                    // successful claim the exclusive access to the address
                    self.registers.write(rd, value);
                    self.local_monitor_bit = true;
                }

                LoadStatus::Error(_e) => {
                    self.trap_handle(Exception::LoadFault);
                    return;
                }
            },
            State::BusWaitStore(store_status) => match *store_status.clone().borrow() {
                StoreStatus::Waiting => self.state = State::BusWaitStore(store_status),
                StoreStatus::Done => (),
                StoreStatus::ExclusiveDone => {
                    // Unblock the exclusive access to the address
                    self.local_monitor_bit = false;
                }
                StoreStatus::Error(_e) => {
                    self.trap_handle(Exception::StoreFault);
                    return;
                }
            },
            State::Wfi => {
                match self.csrs.interrupt_check(self.pc, ctx.interrupts.clone()) {
                    Some(new_pc) => {
                        self.pc = new_pc;
                        self.state = State::Normal;
                    }

                    None => {
                        // No interrupt, just return to WFI state
                        self.state = State::Wfi;
                    }
                }
            }

            State::Atomic {
                rd,
                bus_state,
                address,
                value,
                op,
            } => match *bus_state.clone().borrow() {
                LoadStatus::Waiting => {
                    self.state = State::Atomic {
                        rd,
                        bus_state,
                        address,
                        value,
                        op,
                    };
                }
                LoadStatus::ExclusiveDone(read_value) => {
                    let new_value = match op {
                        AtomicOp::Add => read_value.wrapping_add(value),
                        AtomicOp::And => read_value & value,
                        AtomicOp::Or => read_value | value,
                        AtomicOp::Xor => read_value ^ value,
                        AtomicOp::Swap => value,
                        AtomicOp::Min => read_value.signed().min(value.signed()) as u32,
                        AtomicOp::Max => read_value.signed().max(value.signed()) as u32,
                        AtomicOp::MinU => read_value.min(value),
                        AtomicOp::MaxU => read_value.max(value),
                    };

                    let store_status = ctx.bus.store(
                        address,
                        new_value,
                        BusAccessContext {
                            size: DataSize::Word,
                            exclusive: true,
                            signed: false,
                            secure: self.csrs.privilege_mode() == PrivilegeMode::Machine,
                            architecture: ArchitectureType::Hazard3,
                            requestor: match self.csrs.core_id {
                                0 => Requestor::Proc0,
                                1 => Requestor::Proc1,
                                _ => unreachable!(),
                            },
                        },
                    );

                    self.local_monitor_bit = true;

                    match store_status {
                        Ok(status) => {
                            self.registers.write(rd, read_value);
                            self.state = State::BusWaitStore(status);
                        }
                        Err(_e) => {
                            self.trap_handle(Exception::StoreFault);
                            return;
                        }
                    }
                }
                LoadStatus::Error(_e) => {
                    self.trap_handle(Exception::LoadFault);
                    return;
                }
                LoadStatus::Done(_) => unreachable!(),
            },
            State::Normal => {}
        };
    }

    fn exec_next_instruction_sequence(&mut self, ctx: &mut ProcessorContext) {
        // TODO: Is the timing of these actions correct?
        // The sequence is guaranteed to be non-empty
        match self.inst_seq.pop().unwrap() {
            ZcmpAction::RegisterUpdate(rd, val) => {
                self.registers.write(rd, val);
            }
            ZcmpAction::RegisterMove(from, to) => {
                let val = self.registers.read(from);
                self.registers.write(to, val);
            }
            ZcmpAction::Store {
                address,
                from_register,
            } => {
                let value = self.registers.read(from_register);
                let store_status = ctx.bus.store(
                    address,
                    value,
                    BusAccessContext {
                        size: DataSize::Word,
                        exclusive: false,
                        signed: false,
                        secure: self.csrs.privilege_mode() == PrivilegeMode::Machine,
                        architecture: ArchitectureType::Hazard3,
                        requestor: match self.csrs.core_id {
                            0 => Requestor::Proc0,
                            1 => Requestor::Proc1,
                            _ => unreachable!(),
                        },
                    },
                );

                match store_status {
                    Ok(status) => {
                        self.state = State::BusWaitStore(status);
                    }
                    Err(_e) => {
                        self.trap_handle(Exception::StoreFault);
                        return;
                    }
                }
            }

            ZcmpAction::Load {
                address,
                to_register,
            } => {
                let load_status = ctx.bus.load(
                    address,
                    BusAccessContext {
                        size: DataSize::Word,
                        exclusive: false,
                        signed: false,
                        secure: self.csrs.privilege_mode() == PrivilegeMode::Machine,
                        architecture: ArchitectureType::Hazard3,
                        requestor: match self.csrs.core_id {
                            0 => Requestor::Proc0,
                            1 => Requestor::Proc1,
                            _ => unreachable!(),
                        },
                    },
                );

                match load_status {
                    Ok(status) => {
                        self.state = State::BusWaitLoad(to_register, status);
                    }
                    Err(_e) => {
                        self.trap_handle(Exception::LoadFault);
                        return;
                    }
                }
            }

            ZcmpAction::Return => {
                // RET = JALR x0, 0(x1)
                let return_address = self.registers.read(1);
                self.pc = return_address;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::inspector::*;
    use crate::processor::ProcessorContext;

    const SRAM: u32 = 0x2000_0000;

    macro_rules! setup {
        ($cpu:tt, $ctx:tt) => {
            let mut $cpu = Hazard3::new();
            let mut bus = Bus::default();

            $cpu.set_pc(SRAM);

            let mut $ctx = ProcessorContext {
                bus: &mut bus,
                wake_opposite_core: false,
                interrupts: Default::default(),
                inspector: InspectorRef::default(),
            };
        };
    }

    #[test]
    fn test_lui_and_delayed_m_stage() {
        setup!(cpu, ctx);
        ctx.bus.sram.write_u32(0, 0x0ffff0b7).unwrap(); // lui x1, 65535
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
        ctx.bus.sram.write_u32(0, 0x0ffff037).unwrap(); // lui x0, 65535
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
