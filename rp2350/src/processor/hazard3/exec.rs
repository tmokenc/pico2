/**
 * @file /processor/hazard/exec.rs
 * @author Nguyen Le Duy
 * @date 02/01/2025
 * @brief Hazard3 processor execution unit
 */
use super::instruction_format::*;
use super::registers::{Register, RegisterValue};
use super::PrivilegeMode;
use super::*;
use crate::bus::{Bus, BusAccessContext, LoadStatus, StoreStatus};
use crate::common::*;
use crate::utils::{extract_bit, extract_bits, sign_extend, Fifo};
use num_traits::AsPrimitive;
use std::cell::RefCell;
use std::rc::Rc;

const OPCODE_MASK: u32 = 0b1111111;
const OPCODE_SYSTEM: u32 = 0b1110011;
const OPCODE_LOAD: u32 = 0b0000011;
const OPCODE_STORE: u32 = 0b0100011;
const OPCODE_ARITHMETIC_IMM: u32 = 0b0010011;
const OPCODE_AIRTHMETIC_REG: u32 = 0b0110011;
const OPCODE_BRANCH: u32 = 0b1100011;
const OPCODE_ATOMIC: u32 = 0b0101111;
const OPCODE_CUSTOM0: u32 = 0b0001011;

const OPCODE_JAL: u32 = 0b1101111;
const OPCODE_JALR: u32 = 0b1100111;
const OPCODE_LUI: u32 = 0b0110111;
const OPCODE_AUIPC: u32 = 0b0010111;

// compressed instructions always have opcode and func3 part of the instruction
const OPCODE_COMPRESSED_MASK: u16 = 0b11 | 0b111 << 13;

const fn func3(code: u32) -> u32 {
    code >> 12 & 0b111
}

const fn func7(code: u32) -> u32 {
    code >> 25 & 0b1111111
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AtomicOp {
    Add,
    And,
    Or,
    Xor,
    Swap,
    Min,
    Max,
    MinU,
    MaxU,
}

pub(super) enum MemoryAccess {
    Load(Register, Rc<RefCell<LoadStatus>>),
    Store(Rc<RefCell<StoreStatus>>),
    Atomic {
        address: u32,
        rd: Register,
        bus_state: Rc<RefCell<LoadStatus>>,
        value: u32,
        op: AtomicOp,
    },
    None,
}

pub(super) type InstructionSequence = Fifo<ZcmpAction, 32>;

#[derive(Default, Debug, Clone, Copy)]
pub(super) enum ZcmpAction {
    RegisterUpdate(Register, u32),
    // From, To
    RegisterMove(Register, Register),
    Store {
        address: u32,
        from_register: Register,
    },
    Load {
        address: u32,
        to_register: Register,
    },
    #[default]
    Return,
}

pub(super) struct ExecContext<'a> {
    xx_bypassed: bool,
    pub(super) cycles: u8,
    pub(super) next_pc: u32,
    pub(super) register_write: Option<(Register, u32)>,
    pub(super) exception: Option<Exception>,
    pub(super) memory_access: MemoryAccess,
    pub(super) bus: &'a mut Bus,
    pub(super) core: &'a mut Hazard3,
    pub(super) instruction_name: &'static str,
    pub(super) wake_opposite_core: bool,
    pub(super) zcmp_actions: InstructionSequence,
}

impl ExecContext<'_> {
    pub fn new<'a>(core: &'a mut Hazard3, bus: &'a mut Bus) -> ExecContext<'a> {
        ExecContext {
            cycles: 1,
            xx_bypassed: false,
            register_write: None,
            exception: None,
            instruction_name: "Unknown",
            next_pc: 0,
            memory_access: MemoryAccess::None,
            zcmp_actions: Fifo::default(),
            wake_opposite_core: false,
            core,
            bus,
        }
    }

    pub fn finalize(&mut self) {
        if self.xx_bypassed {
            self.cycles += 1;
        }
    }

    fn privilege_mode(&self) -> PrivilegeMode {
        self.core.csrs.privilege_mode
    }

    fn read_register(&mut self, reg: u8) -> u32 {
        if let Some(bypassing_value) = self.core.xx_bypass {
            if bypassing_value.0 == reg {
                self.xx_bypassed = true;
                return bypassing_value.1;
            }
        }

        self.core.registers.read(reg)
    }

    fn write_register(&mut self, reg: Register, value: impl RegisterValue) {
        self.register_write = Some((reg, value.as_u32()));
    }

    fn read_csr(&mut self, csr: u32) -> Result<u32, Exception> {
        self.core.csrs.read(csr as u16)
    }

    fn write_csr(
        &mut self,
        csr: impl AsPrimitive<u16>,
        value: impl AsPrimitive<u32>,
    ) -> Result<(), Exception> {
        self.core.csrs.write(csr.as_(), value.as_())
    }

    fn atomic_access(
        &mut self,
        rd: Register,
        address: impl AsPrimitive<u32>,
        value: u32,
        op: AtomicOp,
    ) {
        let address: u32 = address.as_();
        if !is_address_aligned(address, DataSize::Word) {
            self.raise_exception(Exception::LoadAlignment);
            return;
        }

        let bus_ctx = BusAccessContext {
            size: DataSize::Word,
            signed: false,
            exclusive: true,
            secure: self.privilege_mode() == PrivilegeMode::Machine,
            architecture: ArchitectureType::Hazard3,
            requestor: match self.core.csrs.core_id {
                0 => Requestor::Proc0,
                1 => Requestor::Proc1,
                _ => unreachable!(),
            },
        };

        match self.bus.load(address, bus_ctx) {
            Ok(status) => {
                self.memory_access = MemoryAccess::Atomic {
                    address,
                    rd,
                    bus_state: status,
                    value,
                    op,
                }
            }
            Err(_e) => {
                self.raise_exception(Exception::LoadFault);
            }
        }
    }

    fn _load(
        &mut self,
        rd: Register,
        address: impl AsPrimitive<u32>,
        size: DataSize,
        signed: bool,
        exclusive: bool,
    ) {
        if !is_address_aligned(address.as_(), size) {
            self.raise_exception(Exception::LoadAlignment);
            return;
        }

        let bus_ctx = BusAccessContext {
            size,
            signed,
            exclusive,
            secure: self.privilege_mode() == PrivilegeMode::Machine,
            architecture: ArchitectureType::Hazard3,
            requestor: match self.core.csrs.core_id {
                0 => Requestor::Proc0,
                1 => Requestor::Proc1,
                _ => unreachable!(),
            },
        };

        match self.bus.load(address.as_(), bus_ctx) {
            Ok(status) => self.memory_access = MemoryAccess::Load(rd, status),
            Err(_e) => {
                self.raise_exception(Exception::LoadFault);
            }
        }
    }

    fn load(&mut self, rd: Register, address: impl AsPrimitive<u32>, size: DataSize, signed: bool) {
        self._load(rd, address, size, signed, false);
    }

    fn load_exclusive(&mut self, rd: Register, address: impl AsPrimitive<u32>) {
        self._load(rd, address, DataSize::Word, false, true);
    }

    fn _store(&mut self, address: u32, value: u32, size: DataSize, exclusive: bool) {
        if !is_address_aligned(address, size) {
            self.raise_exception(Exception::StoreAlignment);
            return;
        }

        let bus_ctx = BusAccessContext {
            size,
            signed: false,
            exclusive,
            secure: self.privilege_mode() == PrivilegeMode::Machine,
            architecture: ArchitectureType::Hazard3,
            requestor: match self.core.csrs.core_id {
                0 => Requestor::Proc0,
                1 => Requestor::Proc1,
                _ => unreachable!(),
            },
        };

        match self.bus.store(address, value, bus_ctx) {
            Ok(status) => self.memory_access = MemoryAccess::Store(status),
            Err(_e) => self.raise_exception(Exception::StoreFault),
        }
    }

    fn store(&mut self, address: u32, value: u32, size: DataSize) {
        self._store(address, value, size, false);
    }

    fn store_exclusive(&mut self, address: u32, value: u32) {
        self._store(address, value, DataSize::Word, true);
    }

    fn get_pc(&self) -> u32 {
        self.core.pc
    }

    fn set_next_pc_offset(&mut self, offset: impl AsPrimitive<i32>) {
        self.next_pc = (self.core.pc as i32).wrapping_add(offset.as_()).as_();
    }

    fn set_absolute_pc_value(&mut self, value: u32) {
        self.next_pc = value;
    }

    fn raise_exception(&mut self, exception: Exception) {
        self.exception = Some(exception);
    }

    fn set_cycles(&mut self, cycles: u8) {
        self.cycles = cycles;
    }

    fn branch(&mut self, taken: bool, label: impl AsPrimitive<i32>) {
        if self
            .core
            .branch_predictor
            .miss_predicted(self.core.pc, taken)
        {
            // cost of misprediction
            self.cycles += 1;
        }

        if taken {
            self.set_next_pc_offset(label);
        }
    }

    fn inst_name(&mut self, name: &'static str) {
        self.instruction_name = name;
    }

    fn wfi(&mut self) {
        self.core.state = State::Wfi;
    }

    fn add_zcmp_action(&mut self, action: ZcmpAction) {
        self.zcmp_actions.push(action);
    }

    fn zcmp_stack_push(&mut self, code: u16) {
        let mut addr = self.read_register(2);
        let sp = addr - zcmp_stack_adj(code);
        let reg_mask = zcmp_reg_mask(code);

        if reg_mask == 0 {
            // reserved
            self.raise_exception(Exception::IllegalInstruction);
            return;
        }

        for i in (1..32).rev() {
            if (reg_mask & (1 << i)) != 0 {
                addr -= 4;
                self.add_zcmp_action(ZcmpAction::Store {
                    address: addr,
                    from_register: i,
                });
            }
        }

        self.add_zcmp_action(ZcmpAction::RegisterUpdate(2, sp));
    }

    fn zcmp_stack_pop(&mut self, code: u16, ret: bool, clear_a0: bool) {
        // CM.POP/CM.POPRET/CM.POPRETZ
        let mut addr = self.read_register(2) + zcmp_stack_adj(code);
        let sp = addr;
        let reg_mask = zcmp_reg_mask(code);

        if reg_mask == 0 {
            // reserved
            self.raise_exception(Exception::IllegalInstruction);
            return;
        }

        for i in (1..32).rev() {
            if (reg_mask & (1 << i)) != 0 {
                addr -= 4;
                self.add_zcmp_action(ZcmpAction::Load {
                    address: addr,
                    to_register: i,
                });
            }
        }

        if clear_a0 {
            // clear A0 register
            self.add_zcmp_action(ZcmpAction::RegisterUpdate(10, 0));
        }

        if ret {
            self.add_zcmp_action(ZcmpAction::Return);
        }

        // update stack pointer
        self.add_zcmp_action(ZcmpAction::RegisterUpdate(2, sp));
    }
}

fn is_address_aligned(address: u32, size: DataSize) -> bool {
    match size {
        DataSize::Byte => true,
        DataSize::HalfWord => address & 1 == 0,
        DataSize::Word => address & 0b11 == 0,
    }
}

fn exec_system_instruction(code: u32, ctx: &mut ExecContext) {
    match code {
        0b00000000000000000000000001110011 => {
            ctx.inst_name("ECALL");
            ctx.set_cycles(3);

            if ctx.privilege_mode() == PrivilegeMode::Machine {
                ctx.raise_exception(Exception::EcallMMode);
            } else {
                ctx.raise_exception(Exception::EcallUMode);
            }
        }
        0b00000000000100000000000001110011 => {
            ctx.inst_name("EBREAK");
            ctx.set_cycles(3);
            ctx.raise_exception(Exception::BreakPoint);
        }
        0b00110000001000000000000001110011 => {
            ctx.inst_name("MRET");

            if ctx.privilege_mode() != PrivilegeMode::Machine {
                ctx.raise_exception(Exception::IllegalInstruction);
                return;
            }

            let next_pc = ctx.core.csrs.trap_mret();
            ctx.set_absolute_pc_value(next_pc);
            ctx.set_cycles(2);
        }
        0b00010000010100000000000001110011 => {
            ctx.inst_name("WFI");
            if ctx.privilege_mode() != PrivilegeMode::Machine {
                // check for MSTATUS.TV is clear or not (must be cleared)
                if ctx.core.csrs.mstatus & super::csrs::MSTATUS_TW != 0 {
                    ctx.raise_exception(Exception::IllegalInstruction);
                    return;
                }
            }

            //If MIP.MEIP is 1, MIE.MEIE is 1, and MSTATUS.MIE is 0, a wfi instruction falls through immediately without pausing.
            if ctx.core.csrs.mip & super::csrs::MIP_MEIP != 0
                && ctx.core.csrs.mie & super::csrs::MIE_MEIE != 0
                && ctx.core.csrs.mstatus & super::csrs::MSTATUS_MIE == 0
            {
                return;
            }

            ctx.wfi();
        }
        _ => {
            let IType { rd, rs1, imm } = code.into();
            let imm = imm & 0xfff; // unsigned

            macro_rules! read_csr {
                ($csr:expr) => {
                    match ctx.read_csr($csr) {
                        Ok(value) => value,
                        Err(e) => {
                            ctx.raise_exception(e);
                            return;
                        }
                    }
                };
            }

            macro_rules! write_csr {
                ($csr:expr, $value:expr) => {
                    if let Err(e) = ctx.write_csr($csr, $value) {
                        ctx.raise_exception(e);
                    }
                };
            }

            match func3(code) {
                0b001 => {
                    ctx.inst_name("CSRRW");
                    if rd != 0 {
                        let csr = read_csr!(imm);
                        ctx.write_register(rd, csr);
                    }

                    let rs1_value = ctx.read_register(rs1);
                    write_csr!(imm, rs1_value);
                }
                0b010 => {
                    ctx.inst_name("CSRRS");
                    let csr = read_csr!(imm);
                    ctx.write_register(rd, csr);
                    if rs1 != 0 {
                        let rs1_value = ctx.read_register(rs1);
                        write_csr!(imm, csr | rs1_value);
                    }
                }
                0b011 => {
                    ctx.inst_name("CSRRC");
                    let csr = read_csr!(imm);
                    ctx.write_register(rd, csr);
                    if rs1 != 0 {
                        let rs1_value = ctx.read_register(rs1);
                        write_csr!(imm, csr & !rs1_value);
                    }
                }
                0b101 => {
                    ctx.inst_name("CSRRWI");

                    if rd != 0 {
                        let csr = read_csr!(imm);
                        ctx.write_register(rd, csr);
                    }

                    write_csr!(imm, rs1);
                }
                0b110 => {
                    ctx.inst_name("CSRRSI");
                    let csr = read_csr!(imm);
                    ctx.write_register(rd, csr);
                    if rs1 != 0 {
                        write_csr!(imm, csr | (rs1 as u32));
                    }
                }
                0b111 => {
                    ctx.inst_name("CSRRCI");
                    let csr = read_csr!(imm);
                    ctx.write_register(rd, csr);

                    if rs1 != 0 {
                        write_csr!(imm, csr & !(rs1 as u32));
                    }
                }
                _ => ctx.raise_exception(Exception::IllegalInstruction),
            }
        }
    }
}

#[inline]
fn exec_load_instruction(code: u32, ctx: &mut ExecContext) {
    let IType { rd, rs1, imm } = code.into();
    let address = ctx.read_register(rs1).wrapping_add(imm);

    match func3(code) {
        0b000 => {
            ctx.inst_name("LB");
            ctx.load(rd, address, DataSize::Byte, true);
        }
        0b001 => {
            ctx.inst_name("LH");
            ctx.load(rd, address, DataSize::HalfWord, true);
        }
        0b010 => {
            ctx.inst_name("LW");
            ctx.load(rd, address, DataSize::Word, true);
        }
        0b100 => {
            ctx.inst_name("LBU");
            ctx.load(rd, address, DataSize::Byte, false);
        }
        0b101 => {
            ctx.inst_name("LHU");
            ctx.load(rd, address, DataSize::HalfWord, false);
        }
        _ => ctx.raise_exception(Exception::IllegalInstruction),
    }
}

#[inline]
fn exec_store_instruction(code: u32, ctx: &mut ExecContext) {
    let SType { rs1, rs2, imm } = code.into();
    let address = ctx.read_register(rs1).wrapping_add(imm);
    let value = ctx.read_register(rs2);

    match func3(code) {
        0b000 => {
            ctx.inst_name("SB");
            ctx.store(address, value, DataSize::Byte);
        }
        0b001 => {
            ctx.inst_name("SH");
            ctx.store(address, value, DataSize::HalfWord);
        }
        0b010 => {
            ctx.inst_name("SW");
            ctx.store(address, value, DataSize::Word);
        }
        _ => ctx.raise_exception(Exception::IllegalInstruction),
    }
}

#[inline]
fn exec_arit_imm_instruction(code: u32, ctx: &mut ExecContext) {
    let IType { rd, rs1, imm } = code.into();
    let a = ctx.read_register(rs1);
    // let imm_signed = sign_extend(imm, 11) as i32;
    let rs2 = imm & 0b11111;
    let shamt = rs2;

    match (func3(code), func7(code)) {
        (0b000, _) => {
            ctx.inst_name("ADDI");
            ctx.write_register(rd, a.wrapping_add(imm));
        }
        (0b010, _) => {
            ctx.inst_name("SLTI");
            ctx.write_register(rd, a.signed() < imm.signed());
        }
        (0b011, _) => {
            ctx.inst_name("SLTIU");
            ctx.write_register(rd, a < imm);
        }
        (0b100, _) => {
            ctx.inst_name("XORI");
            ctx.write_register(rd, a ^ imm);
        }
        (0b110, _) => {
            ctx.inst_name("ORI");
            ctx.write_register(rd, a | imm);
        }
        (0b111, _) => {
            ctx.inst_name("ANDI");
            ctx.write_register(rd, a & imm);
        }
        // Zbs
        (0b001, 0b0100100) => {
            ctx.inst_name("BCLRI");
            ctx.write_register(rd, a & !(1 << shamt));
        }
        (0b101, 0b0100100) => {
            ctx.inst_name("BEXTI");
            ctx.write_register(rd, (a >> shamt) & 1);
        }
        (0b001, 0b0110100) => {
            ctx.inst_name("BINVI");
            ctx.write_register(rd, a ^ (1 << shamt));
        }
        (0b001, 0b0010100) => {
            ctx.inst_name("BSETI");
            ctx.write_register(rd, a | (1 << shamt));
        }

        (0b001, 0b0000000) => {
            ctx.inst_name("SLLI");
            ctx.write_register(rd, a << shamt);
        }
        // Zbb extension
        (0b001, 0b0110000) => match rs2 {
            0b00000 => {
                ctx.inst_name("CLZ");
                ctx.write_register(rd, a.leading_zeros());
            }
            0b00010 => {
                ctx.inst_name("CPOP");
                ctx.write_register(rd, a.count_ones());
            }
            0b00001 => {
                ctx.inst_name("CTZ");
                ctx.write_register(rd, a.trailing_zeros());
            }
            0b00100 => {
                ctx.inst_name("SEXT.B");
                ctx.write_register(rd, sign_extend(a, 7));
            }
            0b00101 => {
                ctx.inst_name("SEXT.H");
                ctx.write_register(rd, sign_extend(a, 15));
            }
            _ => ctx.raise_exception(Exception::IllegalInstruction),
        },
        (0b101, 0b0000000) => {
            ctx.inst_name("SRLI");
            ctx.write_register(rd, a >> shamt);
        }
        (0b101, 0b0100000) => {
            ctx.inst_name("SRAI");
            ctx.write_register(rd, a.signed() >> shamt);
        }
        (0b101, 0b0010100) if rs2 == 0b00111 => {
            ctx.inst_name("ORC.B");
            let bytes = a.to_le_bytes().map(|byte| if byte != 0 { 0xff } else { 0 });
            ctx.write_register(rd, u32::from_le_bytes(bytes));
        }
        (0b101, 0b0110100) if rs2 == 0b11000 => {
            ctx.inst_name("REV8");
            ctx.write_register(rd, a.swap_bytes());
        }
        (0b101, 0b0110000) => {
            ctx.inst_name("RORI");
            ctx.write_register(rd, a.rotate_right(shamt));
            // ctx.write_register(rd, (a >> shamt) | (a << (32 - shamt)));
        }

        // Zbkb
        (0b101, 0b0110100) if rs2 == 0b00111 => {
            ctx.inst_name("BREV8");
            let mut bytes = a.to_le_bytes();

            for i in 0..4 {
                bytes[i] = bytes[i].reverse_bits();
            }

            ctx.write_register(rd, u32::from_le_bytes(bytes));
        }
        (0b101, 0b0000100) if rs2 == 0b01111 => {
            ctx.inst_name("UNZIP");
            let mut result = 0;

            for i in 0..16 {
                result |= (a >> (2 * i) & 1) << i;
                result |= (a >> (2 * i + 1) & 1) << (i + 16);
            }

            ctx.write_register(rd, result);
        }
        (0b001, 0b0000100) if rs2 == 0b01111 => {
            ctx.inst_name("ZIP");
            let mut result = 0;

            for i in 0..16 {
                result |= (a >> i & 1) << (2 * i);
                result |= (a >> (i + 16) & 1) << (2 * i + 1);
            }

            ctx.write_register(rd, result);
        }
        _ => ctx.raise_exception(Exception::IllegalInstruction),
    }
}

#[inline]
fn exec_arit_reg_instruction(code: u32, ctx: &mut ExecContext) {
    let RType { rd, rs1, rs2 } = code.into();
    let a = ctx.read_register(rs1);
    let b = ctx.read_register(rs2);

    match (func3(code), func7(code)) {
        (0b000, 0b0000000) => {
            ctx.inst_name("ADD");
            ctx.write_register(rd, a.wrapping_add(b));
        }
        (0b000, 0b0100000) => {
            ctx.inst_name("SUB");
            ctx.write_register(rd, a.wrapping_sub(b));
        }
        (0b001, 0b0000000) => {
            ctx.inst_name("SLL");
            ctx.write_register(rd, a << (b & 0b11111));
        }
        (0b010, 0b0000000) => {
            ctx.inst_name("SLT");
            ctx.write_register(rd, a.signed() < b.signed());
        }
        (0b011, 0b0000000) => {
            ctx.inst_name("SLTU");
            ctx.write_register(rd, a < b);
        }
        (0b100, 0b0000000) => {
            ctx.inst_name("XOR");
            ctx.write_register(rd, a ^ b);
        }
        (0b101, 0b0000000) => {
            ctx.inst_name("SRL");
            ctx.write_register(rd, a >> (b & 0b11111));
        }
        (0b101, 0b0100000) => {
            ctx.inst_name("SRA");
            ctx.write_register(rd, a.signed() >> (b & 0b11111).signed());
        }
        (0b110, 0b0000000) => {
            ctx.inst_name("OR");
            ctx.write_register(rd, a | b);
        }
        (0b111, 0b0000000) => {
            ctx.inst_name("AND");
            ctx.write_register(rd, a & b);
        }
        // M standard extension
        (0b000, 0b0000001) => {
            ctx.inst_name("MUL");
            ctx.write_register(rd, a.wrapping_mul(b));
        }
        (0b001, 0b0000001) => {
            ctx.inst_name("MULH");
            ctx.write_register(rd, {
                ((a.signed() as i64).wrapping_mul(b.signed() as i64) >> 32) as u32
            });
        }
        (0b010, 0b0000001) => {
            ctx.inst_name("MULHSU");
            ctx.write_register(rd, {
                ((a.signed() as i64).wrapping_mul(b as u64 as i64) >> 32) as u32
            });
        }
        (0b011, 0b0000001) => {
            ctx.inst_name("MULHU");
            ctx.write_register(rd, ((a as u64).wrapping_mul(b as u64) >> 32) as u32);
        }
        (0b100, 0b0000001) => {
            ctx.inst_name("DIV");

            let result = if b == 0 {
                -1
            } else if (a == 0x80000000) && (b.signed() == -1) {
                0x80000000u32 as i32
            } else {
                a.signed().wrapping_div(b.signed())
            };
            ctx.write_register(rd, result);
            ctx.set_cycles(if result < 0 { 19 } else { 18 });
        }
        (0b101, 0b0000001) => {
            ctx.inst_name("DIVU");
            let result = if b == 0 {
                0xffffffff
            } else {
                a.wrapping_div(b)
            };

            ctx.write_register(rd, result);
            ctx.set_cycles(18);
        }
        (0b110, 0b0000001) => {
            ctx.inst_name("REM");
            let result = if b == 0 {
                a.signed()
            } else {
                a.signed().wrapping_rem(b.signed())
            };

            ctx.write_register(rd, result);
        }
        (0b111, 0b0000001) => {
            ctx.inst_name("REMU");
            let result = if b == 0 { a } else { a.wrapping_rem(b) };
            ctx.write_register(rd, result);
        }

        // Zba extension
        (0b010, 0b0010000) => {
            ctx.inst_name("SH1ADD");
            ctx.write_register(rd, (a << 1).wrapping_add(b));
        }
        (0b100, 0b0010000) => {
            ctx.inst_name("SH2ADD");
            ctx.write_register(rd, (a << 2).wrapping_add(b));
        }
        (0b110, 0b0010000) => {
            ctx.inst_name("SH3ADD");
            ctx.write_register(rd, (a << 3).wrapping_add(b));
        }

        // Zbs extension
        (0b001, 0b0100100) => {
            ctx.inst_name("BCLR");
            ctx.write_register(rd, a & !(1 << (b & 0b11111)));
        }
        (0b101, 0b0100100) => {
            ctx.inst_name("BEXT");
            ctx.write_register(rd, (a >> (b & 0b11111)) & 1);
        }
        (0b001, 0b0110100) => {
            ctx.inst_name("BINV");
            ctx.write_register(rd, a ^ (1 << (b & 0b11111)));
        }
        (0b001, 0b0010100) => {
            ctx.inst_name("BSET");
            ctx.write_register(rd, a | (1 << (b & 0b11111)));
        }

        // Zbb extension
        (0b111, 0b0100000) => {
            ctx.inst_name("ANDN");
            ctx.write_register(rd, a & !b);
        }
        (0b110, 0b0000101) => {
            ctx.inst_name("MAX");
            ctx.write_register(rd, if a.signed() < b.signed() { b } else { a });
        }
        (0b111, 0b0000101) => {
            ctx.inst_name("MAXU");
            ctx.write_register(rd, if a < b { b } else { a });
        }
        (0b100, 0b0000101) => {
            ctx.inst_name("MIN");
            ctx.write_register(rd, if a.signed() < b.signed() { a } else { b });
        }
        (0b101, 0b0000101) => {
            ctx.inst_name("MINU");
            ctx.write_register(rd, if a < b { a } else { b });
        }
        (0b110, 0b0100000) => {
            ctx.inst_name("ORN");
            ctx.write_register(rd, a | !b);
        }
        (0b001, 0b0110000) => {
            ctx.inst_name("ROL");
            let shamt = b & 0b11111;
            ctx.write_register(rd, a.rotate_left(shamt));
            // ctx.write_register(rd, (a << shamt) | (a >> (32 - shamt)));
        }
        (0b101, 0b0110000) => {
            ctx.inst_name("ROR");
            let shamt = b & 0b11111;
            ctx.write_register(rd, a.rotate_right(shamt));
            // ctx.write_register(rd, (a >> shamt) | (a << (32 - shamt)));
        }
        (0b100, 0b0100000) => {
            ctx.inst_name("XNOR");
            ctx.write_register(rd, a ^ !b);
        }
        (0b100, 0b0000100) if rs2 == 0 => {
            ctx.inst_name("ZEXT.H");
            ctx.write_register(rd, a & 0xffff);
        }

        // Zbkb
        (0b100, 0b0000100) => {
            ctx.inst_name("PACK");
            ctx.write_register(rd, (b & 0xffff) << 16 | (a & 0xffff));
        }
        (0b111, 0b0000100) => {
            ctx.inst_name("PACKH");
            ctx.write_register(rd, (b & 0xff) << 8 | (a & 0xff));
        }

        _ => ctx.raise_exception(Exception::IllegalInstruction),
    }
}

#[inline]
fn exec_branch_instruction(code: u32, ctx: &mut ExecContext) {
    let BType { rs1, rs2, imm } = code.into();
    let a = ctx.read_register(rs1);
    let b = ctx.read_register(rs2);

    let taken = match func3(code) {
        0b000 => {
            ctx.inst_name("BEQ");
            a == b
        }
        0b001 => {
            ctx.inst_name("BNE");
            a != b
        }
        0b100 => {
            ctx.inst_name("BLT");
            a.signed() < b.signed()
        }
        0b110 => {
            ctx.inst_name("BLTU");
            a < b
        }
        0b101 => {
            ctx.inst_name("BGE");
            a.signed() >= b.signed()
        }
        0b111 => {
            ctx.inst_name("BGEU");
            a >= b
        }
        _ => {
            ctx.raise_exception(Exception::IllegalInstruction);
            return;
        }
    };

    ctx.branch(taken, imm);
}

#[inline]
fn exec_atomic_instruction(code: u32, ctx: &mut ExecContext) {
    let func3 = func3(code);

    if func3 != 0b010 {
        ctx.raise_exception(Exception::IllegalInstruction);
        return;
    }

    let RType { rd, rs1, rs2 } = code.into();
    let func5 = code >> 27;
    let _ordering = code >> 25 & 0b11; // TODO correctly handle memory ordering

    match func5 {
        0b00010 if rs2 == 0 => {
            ctx.inst_name("LR.W");
            let address = ctx.read_register(rs1);
            ctx.load_exclusive(rd, address);
            return;
        }
        0b00011 => {
            ctx.inst_name("SC.W");
            let address = ctx.read_register(rs1);
            let value = ctx.read_register(rs2);
            ctx.store_exclusive(address, value);
            return;
        }
        _ => {}
    }

    let address = ctx.read_register(rs1);
    let value = ctx.read_register(rs2);

    let op = match func5 {
        0b00001 => {
            ctx.inst_name("AMOSWAP.W");
            AtomicOp::Swap
        }
        0b00000 => {
            ctx.inst_name("AMOADD.W");
            AtomicOp::Add
        }
        0b00100 => {
            ctx.inst_name("AMOXOR.W");
            AtomicOp::Xor
        }
        0b01100 => {
            ctx.inst_name("AMOAND.W");
            AtomicOp::And
        }
        0b01000 => {
            ctx.inst_name("AMOOR.W");
            AtomicOp::Or
        }
        0b10000 => {
            ctx.inst_name("AMOMIN.W");
            AtomicOp::Min
        }
        0b10100 => {
            ctx.inst_name("AMOMAX.W");
            AtomicOp::Max
        }
        0b11000 => {
            ctx.inst_name("AMOMINU.W");
            AtomicOp::MinU
        }
        0b11100 => {
            ctx.inst_name("AMOMAXU.W");
            AtomicOp::MaxU
        }
        _ => {
            ctx.raise_exception(Exception::IllegalInstruction);
            return;
        }
    };

    ctx.atomic_access(rd, address, value, op);
}

#[inline]
// Execute a compressed instruction
// maybe split this into multiple functions for better readability
fn exec_compressed_instruction(code: u16, ctx: &mut ExecContext) {
    match code & OPCODE_COMPRESSED_MASK {
        0b0000000000000000 => {
            ctx.inst_name("C.ADDI4SPN");
            let rd = crs2_(code);
            let nzuimm = (extract_bits(code, 11..=12) << 4)
                + (extract_bits(code, 7..=10) << 6)
                + (extract_bit(code, 6) << 2)
                + (extract_bit(code, 5) << 3);

            if nzuimm == 0 {
                ctx.raise_exception(Exception::IllegalInstruction);
                return;
            }

            let rd_value = ctx.read_register(rd);
            ctx.write_register(rd, rd_value.wrapping_add(nzuimm.into()));
        }
        0b0100000000000000 => {
            ctx.inst_name("C.LW");
            let rd = crs2_(code);
            let rs1 = crs1_(code);
            let addr = ctx.read_register(rs1);
            let offset = (extract_bit(code, 6) << 2)
                + (extract_bits(code, 10..=12) << 3)
                + (extract_bit(code, 5) << 6);

            ctx.load(rd, addr.wrapping_add(offset as _), DataSize::Word, true);
        }
        0b1100000000000000 => {
            ctx.inst_name("C.SW");
            let rd = crs2_(code);
            let rs1 = crs1_(code);
            let addr = ctx.read_register(rs1);
            let offset = (extract_bit(code, 6) << 2)
                + (extract_bits(code, 10..=12) << 3)
                + (extract_bit(code, 5) << 6);

            let value = ctx.read_register(rd);
            ctx.store(addr.wrapping_add(offset as _), value, DataSize::Word);
        }
        0b0000000000000001 => {
            ctx.inst_name("C.ADDI");
            let rd = crs1(code);
            let a = ctx.read_register(rd);
            let imm = imm_ci(code);
            ctx.write_register(rd, a.wrapping_add(imm));
        }
        0b0010000000000001 => {
            ctx.inst_name("C.JAL");
            let pc = ctx.get_pc();
            ctx.set_next_pc_offset(imm_cj(code));
            ctx.write_register(1, pc + 2); // rd is 1
        }
        0b1010000000000001 => {
            ctx.inst_name("C.J");
            let offset = imm_cj(code);
            ctx.write_register(0, ctx.get_pc() + 2);
            ctx.set_next_pc_offset(offset);
        }
        0b0100000000000001 => {
            ctx.inst_name("C.LI");
            let rd = crs1(code);
            let imm = imm_ci(code);
            ctx.write_register(rd, imm);
        }
        0b0110000000000001 => {
            let rd = crs1(code);

            if rd == 0 {
                // reserved if imm 0
                ctx.raise_exception(Exception::IllegalInstruction);
                return;
            }

            if rd == 2 {
                // ADDI16SP
                ctx.inst_name("C.ADDI16SP");
                let imm = (extract_bit(code, 6) << 4)
                    | (extract_bit(code, 5) << 6)
                    | (extract_bits(code, 3..=4) << 7)
                    | (extract_bit(code, 2) << 5)
                    | (extract_bit(code, 12) << 9);
                let imm_signed = sign_extend(imm as u32, 9); // check correctness
                let sp = ctx.read_register(2);
                ctx.write_register(2, sp.wrapping_add(imm_signed));
            } else {
                ctx.inst_name("C.LUI");
                let imm = ((extract_bits(code, 2..=6) as u32) << 12)
                    | (extract_bits(code, 12..=12) as u32) << 17;
                let imm_signed = sign_extend(imm, 17);
                ctx.write_register(rd, imm_signed);
            }
        }
        0b1000000000000001 => {
            let rd = crs1_(code);
            let rd_value = ctx.read_register(rd);

            match extract_bits(code, 10..=12) {
                0b000 => {
                    ctx.inst_name("C.SRLI");
                    // imm[5] (instr[12]) must be 0, else reserved NSE
                    let shamt = extract_bits(code, 2..=6);
                    ctx.write_register(rd, rd_value >> shamt);
                }
                0b001 => {
                    ctx.inst_name("C.SRAI");
                    let shamt = extract_bits(code, 2..=6);
                    ctx.write_register(rd, rd_value.signed() >> shamt);
                }
                0b010 | 0b110 => {
                    ctx.inst_name("C.ANDI");
                    let imm = imm_ci(code);
                    ctx.write_register(rd, rd_value & imm);
                }
                0b011 => {
                    let rs2 = crs2_(code);
                    let a = rd_value;
                    let b = ctx.read_register(rs2);

                    match extract_bits(code, 5..=6) {
                        0b00 => {
                            ctx.inst_name("C.SUB");
                            ctx.write_register(rd, a.wrapping_sub(b));
                        }
                        0b01 => {
                            ctx.inst_name("C.XOR");
                            ctx.write_register(rd, a ^ b);
                        }
                        0b10 => {
                            ctx.inst_name("C.OR");
                            ctx.write_register(rd, a | b);
                        }
                        0b11 => {
                            ctx.inst_name("C.AND");
                            ctx.write_register(rd, a & b);
                        }
                        _ => ctx.raise_exception(Exception::IllegalInstruction),
                    }
                }
                0b111 => match (code >> 2) & 0b11111 {
                    0b11000 => {
                        ctx.inst_name("C.ZEXT.B");
                        ctx.write_register(rd, rd_value & 0xff);
                    }
                    0b11001 => {
                        ctx.inst_name("C.SEXT.B");
                        ctx.write_register(rd, sign_extend(rd_value, 7));
                    }
                    0b11010 => {
                        ctx.inst_name("C.ZEXT.H");
                        ctx.write_register(rd, rd_value & 0xffff);
                    }
                    0b11011 => {
                        ctx.inst_name("C.SEXT.H");
                        ctx.write_register(rd, sign_extend(rd_value, 15));
                    }
                    0b11101 => {
                        ctx.inst_name("C.NOT");
                        ctx.write_register(rd, !rd_value);
                    }
                    v if v >= 0b10000 && v < 0b11000 => {
                        ctx.inst_name("C.MUL");
                        let rs2 = crs2_(code);
                        let a = rd as u32;
                        let b = ctx.read_register(rs2);
                        ctx.write_register(rd, a.wrapping_mul(b));
                    }
                    _ => ctx.raise_exception(Exception::IllegalInstruction),
                },
                _ => ctx.raise_exception(Exception::IllegalInstruction),
            }
        }

        0b1100000000000001 => {
            ctx.inst_name("C.BEQZ");
            let rs1 = crs1_(code);
            let taken = ctx.read_register(rs1) == 0;
            let offset = imm_cb(code);
            ctx.branch(taken, offset);
        }
        0b1110000000000001 => {
            ctx.inst_name("C.BNEZ");
            let rs1 = crs1_(code);
            let taken = ctx.read_register(rs1) != 0;
            let offset = imm_cb(code);
            ctx.branch(taken, offset);
        }

        0b0000000000000010 if extract_bit(code, 12) == 0 => {
            // imm[5] (instr[12]) must be 0, else reserved NSE
            ctx.inst_name("C.SLLI");
            let rd = crs1_(code);
            let shamt = extract_bits(code, 2..=6);
            let value = ctx.read_register(rd);
            ctx.write_register(rd, value << shamt);
        }

        0b1000000000000010 if extract_bit(code, 12) == 0 => {
            let rs2 = crs2(code);
            let rs1 = crs1(code);

            // JR if rs2 == 0, if JR and rs1 == 0, reserved
            if rs1 == 0 {
                ctx.raise_exception(Exception::IllegalInstruction);
                return;
            }

            if rs2 == 0 {
                ctx.inst_name("C.JR");
                let new_pc = ctx.read_register(rs1) & !1; // clear LSB
                println!("new_pc: {:#x}", new_pc);
                ctx.set_absolute_pc_value(new_pc as u32);
                ctx.write_register(0, ctx.get_pc() + 2);
            } else {
                ctx.inst_name("C.MV");
                let value = ctx.read_register(rs2);
                ctx.write_register(rs1, value);
            }
        }
        0b1000000000000010 if extract_bit(code, 12) == 1 => {
            // ADD
            // JALR if !rs2
            // EBREAK if !rs1 && !rs2
            let rs1 = crs1(code); // rd
            let rs2 = crs2(code);

            if rs2 == 0 {
                if rs1 == 0 {
                    ctx.inst_name("C.EBREAK");
                    ctx.set_cycles(3);
                    ctx.raise_exception(Exception::BreakPoint);
                } else {
                    ctx.inst_name("C.JALR");
                    let new_pc = ctx.read_register(rs1) & !1; // clear LSB
                    ctx.write_register(1, ctx.get_pc() + 2);
                    ctx.set_absolute_pc_value(new_pc as u32);
                }
            } else {
                ctx.inst_name("C.ADD");
                let a = ctx.read_register(rs1);
                let b = ctx.read_register(rs2);
                ctx.write_register(rs1, a.wrapping_add(b));
            }
        }

        0b0100000000000010 => {
            ctx.inst_name("C.LWSP");
            let rd = crs1(code);
            let address = ctx.read_register(2);
            let offset = (extract_bit(code, 12) << 5)
                + (extract_bits(code, 4..=6) << 2)
                + (extract_bits(code, 2..=3) << 6);

            ctx.load(rd, address.wrapping_add(offset as _), DataSize::Word, true);
        }

        0b1100000000000010 => {
            ctx.inst_name("C.SWSP");
            let rs2 = crs2(code);
            let value = ctx.read_register(rs2);
            let address = ctx.read_register(2);
            let offset = (extract_bits(code, 9..=12) << 2) + (extract_bits(code, 7..=8) << 6);

            ctx.store(address.wrapping_add(offset as _), value, DataSize::Word);
        }

        0b1000000000000000 => {
            // load store
            let rd = crs2_(code);
            let rs1 = crs1_(code);
            let addr = ctx.read_register(rs1);

            match extract_bits(code, 10..=12) {
                0b000 => {
                    ctx.inst_name("C.LBU");
                    let offset = extract_bit(code, 6) + (extract_bit(code, 5) << 1);
                    ctx.load(rd, addr.wrapping_add(offset as u32), DataSize::Byte, false);
                }
                0b001 if code & (1 << 6) == 0 => {
                    ctx.inst_name("C.LHU");
                    let offset = extract_bit(code, 5) << 1;
                    ctx.load(
                        rd,
                        addr.wrapping_add(offset as u32),
                        DataSize::HalfWord,
                        false,
                    );
                }
                0b001 => {
                    // else
                    ctx.inst_name("C.LH");
                    let offset = extract_bit(code, 5) << 1;
                    ctx.load(
                        rd,
                        addr.wrapping_add(offset as u32),
                        DataSize::HalfWord,
                        true,
                    );
                }
                0b010 => {
                    ctx.inst_name("C.SB");
                    let offset = extract_bit(code, 6) + (extract_bit(code, 5) << 1);
                    let value = ctx.read_register(rd);
                    ctx.store(addr.wrapping_add(offset as u32), value, DataSize::Byte);
                }
                0b011 if code & (1 << 6) == 0 => {
                    ctx.inst_name("C.SH");
                    let offset = extract_bit(code, 5) << 1;
                    let value = ctx.read_register(rd);
                    ctx.store(addr.wrapping_add(offset as u32), value, DataSize::HalfWord);
                }
                _ => ctx.raise_exception(Exception::IllegalInstruction),
            }
        }

        0b1010000000000010 => {
            if extract_bit(code, 12) == 0 {
                let r1s = extract_bits(code, 7..=9) as Register;
                let r2s = extract_bits(code, 2..=4) as Register;

                if r1s == r2s {
                    ctx.raise_exception(Exception::IllegalInstruction);
                    return;
                }

                let xreg1 = zcmp_s_mapping(r1s);
                let xreg2 = zcmp_s_mapping(r2s);

                match code & 0b0000110001100000 {
                    0b0110000100000 => {
                        ctx.inst_name("CM.MVSA01");
                        ctx.add_zcmp_action(ZcmpAction::RegisterMove(10, xreg1));
                        ctx.add_zcmp_action(ZcmpAction::RegisterMove(11, xreg2));
                    }
                    0b0110001100000 => {
                        ctx.inst_name("CM.MVA01S");
                        ctx.add_zcmp_action(ZcmpAction::RegisterMove(xreg1, 10));
                        ctx.add_zcmp_action(ZcmpAction::RegisterMove(xreg2, 11));
                    }
                    _ => ctx.raise_exception(Exception::IllegalInstruction),
                }
            } else {
                match extract_bits(code, 8..=11) {
                    0b1000 => {
                        ctx.inst_name("CM.PUSH");
                        ctx.zcmp_stack_push(code);
                    }
                    0b1010 => {
                        ctx.inst_name("CM.POP");
                        ctx.zcmp_stack_pop(code, false, false);
                    }
                    0b1100 => {
                        ctx.inst_name("CM.POPRETZ");
                        ctx.zcmp_stack_pop(code, true, true);
                    }
                    0b1110 => {
                        ctx.inst_name("CM.POPRET");
                        ctx.zcmp_stack_pop(code, true, false);
                    }
                    _ => {
                        ctx.raise_exception(Exception::IllegalInstruction);
                        return;
                    }
                }
            }
        }

        _ => ctx.raise_exception(Exception::IllegalInstruction),
    }
}

#[inline]
fn exec_custom_instruction(code: u32, ctx: &mut ExecContext) {
    let func3 = func3(code);
    let func7 = func7(code);

    if func3 != 0b000 && func3 != 0b100 {
        ctx.raise_exception(Exception::IllegalInstruction);
        return;
    }

    // func7[0] and func7[6:4] must be 0
    let mask = 0b111 << 4 | 0b1;
    if func7 & mask != 0 {
        ctx.raise_exception(Exception::IllegalInstruction);
        return;
    }

    let RType { rd, rs1, rs2 } = code.into();

    let shamt = match func3 {
        0b000 => {
            ctx.inst_name("H3.BEXTM");
            ctx.read_register(rs2) & 0b11111
        }
        0b100 => {
            ctx.inst_name("H3.BEXTMI");
            rs2 as u32
        }
        _ => {
            ctx.raise_exception(Exception::IllegalInstruction);
            return;
        }
    };

    let size = extract_bits(func7, 1..=3) + 1;
    let a = ctx.read_register(rs1);

    let result = (a >> shamt) & !(((-1i32) as u32) << size);
    ctx.write_register(rd, result);
}

pub(super) fn exec_instruction(code: u32, ctx: &mut ExecContext<'_>) {
    if code & 0b11 == 0b11 {
        log::info!("Normal instruction: {:#x}", code);
        ctx.set_next_pc_offset(4);
        match code & OPCODE_MASK {
            OPCODE_JAL => {
                ctx.inst_name("JAL");

                let JType { rd, imm } = JType::from(code);
                let return_address = ctx.get_pc() + 4;

                ctx.write_register(rd, return_address);
                ctx.set_next_pc_offset(imm);
            }
            OPCODE_JALR if func3(code) == 0b000 => {
                ctx.inst_name("JALR");

                let op = IType::from(code);
                let return_address = ctx.get_pc() + 4;
                let jump_addr = ctx
                    .read_register(op.rs1)
                    .signed()
                    .wrapping_add(op.imm.signed())
                    & !1; // Clear the least significant bit

                ctx.write_register(op.rd, return_address);
                ctx.set_absolute_pc_value(jump_addr as u32);
            }

            OPCODE_LUI => {
                ctx.inst_name("LUI");
                let UType { rd, imm } = UType::from(code);
                ctx.write_register(rd, imm);
            }

            OPCODE_AUIPC => {
                ctx.inst_name("AUIPC");
                let op = UType::from(code);

                let inst_address = ctx.get_pc();
                ctx.write_register(op.rd, inst_address.wrapping_add(op.imm));
            }

            OPCODE_SYSTEM => exec_system_instruction(code, ctx),
            OPCODE_LOAD => exec_load_instruction(code, ctx),
            OPCODE_STORE => exec_store_instruction(code, ctx),
            OPCODE_ARITHMETIC_IMM => exec_arit_imm_instruction(code, ctx),
            OPCODE_AIRTHMETIC_REG => exec_arit_reg_instruction(code, ctx),
            OPCODE_BRANCH => exec_branch_instruction(code, ctx),
            OPCODE_ATOMIC => exec_atomic_instruction(code, ctx),
            OPCODE_CUSTOM0 => exec_custom_instruction(code, ctx),
            _ if code == 0b00000000000000000001000000001111 => {
                ctx.inst_name("FENCE.I");
                // Do nothing
            }
            _ if code == 0b00000000000000000010000000110011 => {
                ctx.inst_name("H3.BLOCK");
                ctx.core.sleep();
            }
            _ if code == 0b00000000000100000010000000110011 => {
                ctx.inst_name("H3.UNBLOCK");
                ctx.wake_opposite_core = true;
            }
            _ if extract_bits(code, 0..=19) == 0b00000000000000001111 => {
                ctx.inst_name("FENCE");
                // Do nothing
            }
            _ => ctx.raise_exception(Exception::IllegalInstruction),
        }
    } else {
        log::info!("Compressed instruction: {:#x}", code as u16);
        ctx.set_next_pc_offset(2);
        exec_compressed_instruction(code as u16, ctx);
    }
}

#[cfg(test)]
mod tests {
    #![allow(unused_variables)]

    use super::*;
    use crate::bus::Bus;
    use crate::clock::Clock;
    use crate::gpio::GpioController;
    use crate::processor::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    const PC: u32 = 0x1000;

    macro_rules! setup {
        ($core:ident, $bus:ident) => {
            let interrupts = Rc::new(RefCell::new(Interrupts::default()));
            let gpio = Rc::new(RefCell::new(GpioController::default()));
            let clock = Rc::new(Clock::default());

            let mut $core = Hazard3::new();
            $core.set_pc(PC);
            let mut $bus = Bus::new(gpio, interrupts, clock, Default::default());
        };
    }

    macro_rules! instruction_test {
        ($name:ident, $instr:expr, $ctx:ident, { $($assertion:tt)* }) => {
            #[test]
            fn $name() {
                setup!(core, bus);
                let mut $ctx = ExecContext::new(&mut core, &mut bus);
                exec_instruction($instr, &mut $ctx);
                $($assertion)*
            }
        };
    }

    macro_rules! aritmetic_test {
        ($name:ident, $instr_mask:expr, [$($a:expr, $b:expr => $expected:expr),* $(,)?]) => {
            #[test]
            fn $name() {
                setup!(core, bus);
                core.set_pc(PC);
                let rs1: u32 = 1;
                let rs2: u32 = 2;
                let rd: u32 = 3;

                {
                    $(
                        core.registers.write(rs1 as u8, $a);
                        core.registers.write(rs2 as u8, $b);

                        let instr = $instr_mask | (rd << 7) | (rs1 << 15) | (rs2 << 20);
                        let mut ctx = ExecContext::new(&mut core, &mut bus);
                        exec_instruction(instr, &mut ctx);

                        assert_eq!(ctx.register_write, Some((rd as u8, $expected)));
                    )*
                }
            }
        };

        ($name:ident, $name_imm:ident, $instr_mask:expr, [$($a:expr, $b:expr => $expected:expr),* $(,)?]) => {
            aritmetic_test!($name, $instr_mask, [$($a, $b => $expected),*]);

            #[test]
            fn $name_imm() {
                setup!(core, bus);
                core.set_pc(PC);
                let rs1: u32 = 1;
                let rs2: u32 = 2;
                let rd: u32 = 3;

                {
                    $(
                        core.registers.write(rs1 as u8, $a);

                        let instr = ($instr_mask & !0b100000) | (rd << 7) | (rs1 << 15) | (($b as u32) << 20);
                        let mut ctx = ExecContext::new(&mut core, &mut bus);
                        exec_instruction(instr, &mut ctx);

                        assert_eq!(ctx.register_write, Some((rd as u8, $expected)));
                    )*
                }
            }
        };

        // For shift instructions with immediate
        ($name:ident,, $name_imm:ident, $instr_mask:expr, [$($a:expr, $b:expr => $expected:expr),* $(,)?]) => {
            aritmetic_test!($name, $instr_mask, [$($a, $b => $expected),*]);

            #[test]
            fn $name_imm() {
                setup!(core, bus);
                core.set_pc(PC);
                let rs1: u32 = 1;
                let rs2: u32 = 2;
                let rd: u32 = 3;

                {
                    $(
                        core.registers.write(rs1 as u8, $a);

                        let instr = ($instr_mask & !0b100000) | (rd << 7) | (rs1 << 15) | ((($b as u32) & 0b11111) << 20);
                        let mut ctx = ExecContext::new(&mut core, &mut bus);
                        exec_instruction(instr, &mut ctx);

                        assert_eq!(ctx.register_write, Some((rd as u8, $expected)));
                    )*
                }
            }
        };
    }

    // for branch instructions
    macro_rules! branch_test {
        ($name:ident, $instr_mask:expr, [$($a:expr, $b:expr => $expected:expr),* $(,)?]) => {
            #[test]
            fn $name() {
                setup!(core, bus);
                core.set_pc(PC);
                let rs1: u32 = 1;
                let rs2: u32 = 2;
                let rd: u32 = 3;
                let offset = 0x100;
                let offset_mask = 1 << 28;

                {
                    $(
                        core.registers.write(rs1 as u8, $a);
                        core.registers.write(rs2 as u8, $b);

                        let instr = ($instr_mask | offset_mask) | (rs1 << 15) | (rs2 << 20);
                        let mut ctx = ExecContext::new(&mut core, &mut bus);
                        exec_instruction(instr, &mut ctx);

                        if $expected {
                            assert_eq!(ctx.next_pc, PC + offset);
                        } else {
                            assert_eq!(ctx.next_pc, PC + 4);
                        }
                    )*
                }
            }
        };
    }

    #[test]
    // happy path for all instructions
    fn base_instruction_test() {
        let instruction_list = [
            ("BEQ", 0b00000000000000000000000001100011),
            ("BNE", 0b00000000000000000001000001100011),
            ("BLT", 0b00000000000000000100000001100011),
            ("BGE", 0b00000000000000000101000001100011),
            ("BLTU", 0b00000000000000000110000001100011),
            ("BGEU", 0b00000000000000000111000001100011),
            ("JALR", 0b00000000000000000000000001100111),
            ("JAL", 0b00000000000000000000000001101111),
            ("LUI", 0b00000000000000000000000000110111),
            ("AUIPC", 0b00000000000000000000000000010111),
            ("ADDI", 0b00000000000000000000000000010011),
            ("SLLI", 0b00000000000000000001000000010011),
            ("SLTI", 0b00000000000000000010000000010011),
            ("SLTIU", 0b00000000000000000011000000010011),
            ("XORI", 0b00000000000000000100000000010011),
            ("SRLI", 0b00000000000000000101000000010011),
            ("SRAI", 0b01000000000000000101000000010011),
            ("ORI", 0b00000000000000000110000000010011),
            ("ANDI", 0b00000000000000000111000000010011),
            ("ADD", 0b00000000000000000000000000110011),
            ("SUB", 0b01000000000000000000000000110011),
            ("SLL", 0b00000000000000000001000000110011),
            ("SLT", 0b00000000000000000010000000110011),
            ("SLTU", 0b00000000000000000011000000110011),
            ("XOR", 0b00000000000000000100000000110011),
            ("SRL", 0b00000000000000000101000000110011),
            ("SRA", 0b01000000000000000101000000110011),
            ("OR", 0b00000000000000000110000000110011),
            ("AND", 0b00000000000000000111000000110011),
            ("LB", 0b00000000000000000000000000000011),
            ("LH", 0b00000000000000000001000000000011),
            ("LW", 0b00000000000000000010000000000011),
            ("LBU", 0b00000000000000000100000000000011),
            ("LHU", 0b00000000000000000101000000000011),
            ("SB", 0b00000000000000000000000000100011),
            ("SH", 0b00000000000000000001000000100011),
            ("SW", 0b00000000000000000010000000100011),
            ("FENCE", 0b00000000000000000000000000001111),
            ("FENCE.I", 0b00000000000000000001000000001111),
            ("ECALL", 0b00000000000000000000000001110011),
            ("EBREAK", 0b00000000000100000000000001110011),
            ("CSRRW", 0b00000000000000000001000001110011),
            ("CSRRS", 0b00000000000000000010000001110011),
            ("CSRRC", 0b00000000000000000011000001110011),
            ("CSRRWI", 0b00000000000000000101000001110011),
            ("CSRRSI", 0b00000000000000000110000001110011),
            ("CSRRCI", 0b00000000000000000111000001110011),
            ("MRET", 0b00110000001000000000000001110011),
            ("WFI", 0b00010000010100000000000001110011),
            ("MUL", 0b00000010000000000000000000110011),
            ("MULH", 0b00000010000000000001000000110011),
            ("MULHSU", 0b00000010000000000010000000110011),
            ("MULHU", 0b00000010000000000011000000110011),
            ("DIV", 0b00000010000000000100000000110011),
            ("DIVU", 0b00000010000000000101000000110011),
            ("REM", 0b00000010000000000110000000110011),
            ("REMU", 0b00000010000000000111000000110011),
            ("LR.W", 0b00010000000000000010000000101111),
            ("SC.W", 0b00011000000000000010000000101111),
            ("AMOSWAP.W", 0b00001000000000000010000000101111),
            ("AMOADD.W", 0b00000000000000000010000000101111),
            ("AMOXOR.W", 0b00100000000000000010000000101111),
            ("AMOAND.W", 0b01100000000000000010000000101111),
            ("AMOOR.W", 0b01000000000000000010000000101111),
            ("AMOMIN.W", 0b10000000000000000010000000101111),
            ("AMOMAX.W", 0b10100000000000000010000000101111),
            ("AMOMINU.W", 0b11000000000000000010000000101111),
            ("AMOMAXU.W", 0b11100000000000000010000000101111),
            ("SH1ADD", 0b00100000000000000010000000110011),
            ("SH2ADD", 0b00100000000000000100000000110011),
            ("SH3ADD", 0b00100000000000000110000000110011),
            ("ANDN", 0b01000000000000000111000000110011),
            ("CLZ", 0b01100000000000000001000000010011),
            ("CPOP", 0b01100000001000000001000000010011),
            ("CTZ", 0b01100000000100000001000000010011),
            ("MAX", 0b00001010000000000110000000110011),
            ("MAXU", 0b00001010000000000111000000110011),
            ("MIN", 0b00001010000000000100000000110011),
            ("MINU", 0b00001010000000000101000000110011),
            ("ORC.B", 0b00101000011100000101000000010011),
            ("ORN", 0b01000000000000000110000000110011),
            ("REV8", 0b01101001100000000101000000010011),
            ("ROL", 0b01100000000000000001000000110011),
            ("ROR", 0b01100000000000000101000000110011),
            ("RORI", 0b01100000000000000101000000010011),
            ("SEXT.B", 0b01100000010000000001000000010011),
            ("SEXT.H", 0b01100000010100000001000000010011),
            ("XNOR", 0b01000000000000000100000000110011),
            ("ZEXT.H", 0b00001000000000000100000000110011),
            ("BCLR", 0b01001000000000000001000000110011),
            ("BCLRI", 0b01001000000000000001000000010011),
            ("BEXT", 0b01001000000000000101000000110011),
            ("BEXTI", 0b01001000000000000101000000010011),
            ("BINV", 0b01101000000000000001000000110011),
            ("BINVI", 0b01101000000000000001000000010011),
            ("BSET", 0b00101000000000000001000000110011),
            ("BSETI", 0b00101000000000000001000000010011),
            ("PACK", 0b00001000100000000100000000110011),
            ("PACKH", 0b00001000000000000111000000110011),
            ("BREV8", 0b01101000011100000101000000010011),
            ("UNZIP", 0b00001000111100000101000000010011),
            ("ZIP", 0b00001000111100000001000000010011),
            ("H3.BEXTM", 0b00000000000000000000000000001011),
            ("H3.BEXTMI", 0b00000000000000000100000000001011),
            ("C.ADDI4SPN", 0b0000000001000000),
            ("C.LW", 0b0100000000000000),
            ("C.SW", 0b1100000000000000),
            ("C.ADDI", 0b0000000000000001),
            ("C.JAL", 0b0010000000000001),
            ("C.J", 0b1010000000000001),
            ("C.LI", 0b0100000000000001),
            ("C.LUI", 0b0110010000000101),
            ("C.ADDI16SP", 0b0110000101000001),
            ("C.SRLI", 0b1000000000000001),
            ("C.SRAI", 0b1000010000000001),
            ("C.ANDI", 0b1000100000000001),
            ("C.SUB", 0b1000110000000001),
            ("C.XOR", 0b1000110000100001),
            ("C.OR", 0b1000110001000001),
            ("C.AND", 0b1000110001100001),
            ("C.BEQZ", 0b1100000000000001),
            ("C.BNEZ", 0b1110000000000001),
            ("C.SLLI", 0b0000000000000010),
            ("C.MV", 0b1000010000100010),
            ("C.JR", 0b1000010000000010),
            ("C.ADD", 0b1001010001001010),
            ("C.JALR", 0b1001010000000010),
            ("C.EBREAK", 0b1001000000000010),
            ("C.LWSP", 0b0100000000000010),
            ("C.SWSP", 0b1100000000000010),
            ("C.LBU", 0b1000000000000000),
            ("C.LHU", 0b1000010000000000),
            ("C.LH", 0b1000010001000000),
            ("C.SB", 0b1000100000000000),
            ("C.SH", 0b1000110000000000),
            ("C.ZEXT.B", 0b1001110001100001),
            ("C.SEXT.B", 0b1001110001100101),
            ("C.ZEXT.H", 0b1001110001101001),
            ("C.SEXT.H", 0b1001110001101101),
            ("C.NOT", 0b1001110001110101),
            ("C.MUL", 0b1001110001000001),
            ("CM.PUSH", 0b1011100000000010),
            ("CM.POP", 0b1011101000000010),
            ("CM.POPRETZ", 0b1011110000000010),
            ("CM.POPRET", 0b1011111000000010),
            ("CM.MVSA01", 0b1010110100100010),
            ("CM.MVA01S", 0b1010110001101010),
        ];

        setup!(core, bus);

        for (instruction, code) in instruction_list.iter() {
            let mut ctx = ExecContext::new(&mut core, &mut bus);
            exec_instruction(*code, &mut ctx);
            assert_eq!(ctx.instruction_name, *instruction);
        }
    }

    instruction_test!(lui, 0b00000001100100100101111110110111, ctx, {
        assert_eq!(ctx.register_write, Some((31, 6437 << 12)));
    });

    instruction_test!(auipc, 0b00000001100100100101111110010111, ctx, {
        assert_eq!(ctx.register_write, Some((31, PC + (6437 << 12))));
    });

    instruction_test!(jal, 0b00010010010100000001111111101111, ctx, {
        assert_eq!(ctx.register_write, Some((31, PC + 4)));
        assert_eq!(ctx.next_pc, PC + (6437 & !1));
    });

    instruction_test!(jalr, 0b11011100110000000000110001100111, ctx, {
        assert_eq!(ctx.register_write, Some((24, PC + 4)));
        assert_eq!(ctx.next_pc, (-564i32 & !1) as u32);
    });

    branch_test!(beq, 0b00000000000000000000000001100011, [
        10, 10 => true,
        10, -10 => false,
        10, 20 => false,
    ]);

    branch_test!(bne, 0b00000000000000000001000001100011, [
        10, 10 => false,
        10, -10 => true,
        10, 20 => true,
    ]);

    branch_test!(bge, 0b00000000000000000101000001100011, [
        10, 10 => true,
        10, -10 => true,
        -10, 10 => false,
        -10, -10 => true,
        0, 1 => false,
        1, 0 => true,
        -1, 0 => false,
    ]);

    branch_test!(blt, 0b00000000000000000100000001100011, [
        10, 10 => false,
        10, -10 => false,
        -10, 10 => true,
        -10, -10 => false,
        0, 1 => true,
        1, 0 => false,
        -1, 0 => true,
    ]);

    branch_test!(bgeu, 0b00000000000000000111000001100011, [
        10, 10 => true,
        10, 20 => false,
        20, 10 => true,
        0, 1 => false,
        1, 0 => true,
        -1, 0 => true,
    ]);

    branch_test!(bltu, 0b00000000000000000110000001100011, [
        10, 10 => false,
        10, 20 => true,
        20, 10 => false,
        0, 1 => true,
        1, 0 => false,
        -1, 0 => false,
    ]);

    // Load
    // instruction_test!(lb, 0b
    //

    // Store
    //

    // Atomic
    //

    aritmetic_test!(add, addi, 0b00000000000000000000000000110011, [10, 20 => 30]);
    aritmetic_test!(and, andi, 0b00000000000000000111000000110011, [0b1010, 0b1100 => 0b1000]);
    aritmetic_test!(or, ori, 0b00000000000000000110000000110011, [0b1010, 0b1100 => 0b1110]);
    aritmetic_test!(sll,, slli, 0b00000000000000000001000000110011, [
        0b1010, 0b111100 => 0b1010 << 0b011100,
        0b1010, 0b11100 => 0b1010 << 0b011100,
        0b1010, 0 => 0b1010,
    ]);

    aritmetic_test!(slt, slti, 0b00000000000000000010000000110011, [
        10, 20 => 1,
        20, 10 => 0,
        1010, 1010 => 0,
        -10, 10 => 1,
        -10, -20i32 => 0,
        -10, -10i32 => 0,
    ]);

    aritmetic_test!(sltu,, sltiu, 0b00000000000000000011000000110011, [
        10, 20 => 1,
        20, 10 => 0,
        1010, 1010 => 0,
        -1, 1 => 0, /* 0b1111_1111 < 0b0000_0001 */
    ]);

    aritmetic_test!(sra,, srai, 0b01000000000000000101000000110011, [
        0b1010, 0b111100 => 0b1010 >> 0b011100,
        0b1010, 0b11100 => 0b1010 >> 0b011100,
        0b1010, 0 => 0b1010,
        -1, 1 => u32::MAX,
        -1, 10000 => u32::MAX,
        -1 & !0b111, 1 => u32::MAX & !0b11,
    ]);

    aritmetic_test!(srl,, srli, 0b00000000000000000101000000110011, [
        0b1010, 0b111100 => 0b1010 >> 0b011100,
        0b1010, 0b11100 => 0b1010 >> 0b011100,
        0b1010, 0 => 0b1010,
        -1, 1 => u32::MAX >> 1,
        -1, 0b11101 => u32::MAX >> 0b11101,
        -1 & !0b111, 1 => (u32::MAX >> 1) & !0b11,
    ]);

    aritmetic_test!(sub, 0b01000000000000000000000000110011, [
        20, 10 => 10,
        10, 20 => u32::MAX - 9,
        1, 2 => u32::MAX,
    ]);

    aritmetic_test!(xor, xori, 0b00000000000000000100000000110011, [
        0b1010, 0b1100 => 0b0110,
        0b1010, 0b1010 => 0,
    ]);

    aritmetic_test!(binv,, binvi, 0b01101000000000000001000000110011, [
        0b1010, 0b1100 => 0b1000000001010,
        0b1010, 0b111100 => (1 << 28) | 0b1010,
        0b1010, 0b11 => 0b10,
    ]);

    #[test]
    fn test_c_mv() {
        setup!(core, bus);

        core.registers.write(11, 0xdeadbeefu32);
        let inst = 0b1000011000101110;
        let mut ctx = ExecContext::new(&mut core, &mut bus);
        exec_instruction(0b1000011000101110, &mut ctx);
        assert_eq!(ctx.register_write, Some((12, 0xdeadbeef)));
    }

    #[test]
    fn test_c_j() {
        setup!(core, bus);

        let inst1 = 0b1010100000100001; // c.j 24
        let inst2 = 0b1010000000000001; // c.j 0
        let inst3 = 0b1011111111111101; // c.j -2

        let mut ctx = ExecContext::new(&mut core, &mut bus);
        exec_instruction(inst1, &mut ctx);
        assert_eq!(ctx.next_pc, PC + 24);
        assert_eq!(ctx.register_write, Some((0, PC + 2)));

        ctx = ExecContext::new(&mut core, &mut bus);
        exec_instruction(inst2, &mut ctx);
        assert_eq!(ctx.next_pc, PC);
        assert_eq!(ctx.register_write, Some((0, PC + 2)));

        ctx = ExecContext::new(&mut core, &mut bus);
        exec_instruction(inst3, &mut ctx);
        assert_eq!(ctx.next_pc, PC - 2);
        assert_eq!(ctx.register_write, Some((0, PC + 2)));
    }

    #[test]
    fn test_c_jr() {
        setup!(core, bus);

        let inst1 = 0b1000010000000010; // c.jr x8
        let inst2 = 0b1000000010000010; // c.jr x1
        let inst3 = 0b1000111110000010; // c.jr x31

        core.registers.write(8, 0xabc);
        core.registers.write(1, 0xdef);
        core.registers.write(31, 0x123);

        let mut ctx = ExecContext::new(&mut core, &mut bus);
        exec_instruction(inst1, &mut ctx);
        let target = 0xabc & !1;
        assert_eq!(ctx.next_pc, target);
        assert_eq!(ctx.register_write, Some((0, PC + 2)));

        ctx = ExecContext::new(&mut core, &mut bus);
        exec_instruction(inst2, &mut ctx);
        let target = 0xdef & !1;
        assert_eq!(ctx.next_pc, target);
        assert_eq!(ctx.register_write, Some((0, PC + 2)));

        ctx = ExecContext::new(&mut core, &mut bus);
        exec_instruction(inst3, &mut ctx);
        let target = 0x123 & !1;
        assert_eq!(ctx.next_pc, target);
        assert_eq!(ctx.register_write, Some((0, PC + 2)));
    }

    // #[test]
    // fn test_c_jal() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_c_jalr() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_c_bnez() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_c_addi() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_c_addi4spn() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_c_add() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_c_sub() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_c_lui() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_c_andi() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_c_slli() {
    //     setup!(core, bus);
    //     // let inst = 0x1000110a;

    //     // core.registers.write(2, 0b1010);

    //     // let mut ctx = ExecContext::new(&mut core, &mut bus);
    //     // exec_instruction(inst, &mut ctx);
    //     // assert_eq!(ctx.instruction_name, "C.SLLI");
    //     // assert_eq!(ctx.register_write, Some((2, 0b1010 << 4)));
    //     // TODO
    // }

    // #[test]
    // fn test_h3_bextmi() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_h3_bextm() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_h3_block() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_h3_unblock() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_pack() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_packh() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_sh1add() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_sh2add() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_sh3add() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_brev8() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // #[test]
    // fn test_brev() {
    //     setup!(core, bus);
    //     // TODO
    // }

    // TODO
    // CSRRW
    // CSRRS
    // BEXTI
    // LW
    // SW
    // LB
    // C.LI
    // C.MV
    // C.LBU
    // C.SW
    // C.LHU
    // C.SWSP
    // CM.PUSH
}
