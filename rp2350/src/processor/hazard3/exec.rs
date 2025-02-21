use super::instruction_format::*;
use super::registers::{Register, RegisterValue};
use super::PrivilegeMode;
use super::*;
use crate::bus::{Bus, BusAccessContext, LoadStatus, StoreStatus};
use crate::common::*;
use crate::utils::{extract_bits, sign_extend};
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

const OPCODE_JAL: u32 = 0b1101111;
const OPCODE_JALR: u32 = 0b1100111;
const OPCODE_LUI: u32 = 0b0110111;
const OPCODE_AUIPC: u32 = 0b0010111;

// compressed instructions always have opcode and func3 part of the instruction
const OPCODE_COMPRESSED_MASK: u16 = 0b00 | 0b111 << 13;

const fn func3(code: u32) -> u32 {
    code >> 12 & 0b111
}

const fn func7(code: u32) -> u32 {
    code >> 25 & 0b1111111
}

pub(super) struct ExecContext<'a> {
    pub(super) cycles: u8,
    pub(super) register_read: Option<Register>,
    pub(super) register_write: Option<(Register, u32)>,
    pub(super) next_pc_offset: i32,
    pub(super) exception: Option<Exception>,
    pub(super) load: Option<(Register, Rc<RefCell<LoadStatus>>)>,
    pub(super) store: Option<Rc<RefCell<StoreStatus>>>,
    pub(super) bus: &'a mut Bus,
    pub(super) core: &'a mut Hazard3,
    pub(super) instruction_name: &'static str,
}

impl ExecContext<'_> {
    pub fn new<'a>(core: &'a mut Hazard3, bus: &'a mut Bus) -> ExecContext<'a> {
        ExecContext {
            cycles: 1,
            register_read: None,
            register_write: None,
            exception: None,
            instruction_name: "",
            next_pc_offset: 0,
            load: None,
            store: None,
            core,
            bus,
        }
    }

    fn read_register(&mut self, reg: u8) -> u32 {
        self.register_read = Some(reg);
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
        // TODO write error
        self.core.csrs.write(csr.as_(), value.as_())
    }

    fn load(&mut self, rd: Register, address: impl AsPrimitive<u32>, size: DataSize) {
        if !is_address_aligned(address.as_(), size) {
            self.raise_exception(Exception::LoadAlignment);
            return;
        }

        let bus_ctx = BusAccessContext {
            size,
            secure: self.core.privilege_mode == PrivilegeMode::Machine,
            architecture: ArchitectureType::Hazard3,
            requestor: match self.core.csrs.core_id {
                0 => Requestor::Proc0,
                1 => Requestor::Proc1,
                _ => unreachable!(),
            },
        };

        match self.bus.load(address.as_(), bus_ctx) {
            Ok(status) => {
                self.load = Some((rd, status));
            }
            Err(_e) => {
                self.raise_exception(Exception::LoadFault);
            }
        }
    }

    fn store(&mut self, address: u32, value: u32, size: DataSize) {
        if !is_address_aligned(address, size) {
            self.raise_exception(Exception::StoreAlignment);
            return;
        }

        let bus_ctx = BusAccessContext {
            size,
            secure: self.core.privilege_mode == PrivilegeMode::Machine,
            architecture: ArchitectureType::Hazard3,
            requestor: match self.core.csrs.core_id {
                0 => Requestor::Proc0,
                1 => Requestor::Proc1,
                _ => unreachable!(),
            },
        };

        match self.bus.store(address, value, bus_ctx) {
            Ok(status) => self.store = Some(status),
            Err(_e) => self.raise_exception(Exception::StoreFault),
        }
    }

    fn get_pc(&self) -> u32 {
        self.core.pc
    }

    fn set_next_pc_offset(&mut self, offset: impl AsPrimitive<i32>) {
        self.next_pc_offset = offset.as_();
    }

    fn raise_exception(&mut self, exception: Exception) {
        self.exception = Some(exception);
    }

    fn set_cycles(&mut self, cycles: u8) {
        self.cycles = cycles;
    }

    fn branch(&mut self, taken: bool, label: impl AsPrimitive<i32>) {
        // TODO branch prediction
        if taken {
            self.next_pc_offset = label.as_();
        }
    }

    fn inst_name(&mut self, name: &'static str) {
        self.instruction_name = name;
    }

    fn wfi(&mut self) {
        self.core.state = State::Wfi;
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

            if ctx.core.privilege_mode == PrivilegeMode::Machine {
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

            if ctx.core.privilege_mode != PrivilegeMode::Machine {
                // TODO: Raise exception
            }

            todo!()
        }
        0b00010000010100000000000001110011 => {
            ctx.inst_name("WFI");
            if ctx.core.privilege_mode != PrivilegeMode::Machine {
                // TODO check for MSTATUS.TV is clear or not (must be cleared)
            }
            ctx.wfi();
            // TODO
        }
        _ => {
            let IType { rd, rs1, imm } = code.into();

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
            ctx.load(rd, address, DataSize::Byte);
        }
        0b001 => {
            ctx.inst_name("LH");
            ctx.load(rd, address, DataSize::HalfWord);
        }
        0b010 => {
            ctx.inst_name("LW");
            ctx.load(rd, address, DataSize::Word);
        }
        0b100 => {
            ctx.inst_name("LBU");
            ctx.load(rd, address, DataSize::Byte); // TODO
        }
        0b101 => {
            ctx.inst_name("LHU");
            ctx.load(rd, address, DataSize::HalfWord); // TODO
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
            ctx.write_register(rd, (a >> shamt) | (a << (32 - shamt)));
        }

        // Zbkb
        (0b101, 0b0000100) if rs2 == 0b00111 => {
            ctx.inst_name("BREV8");
            let mut bytes = a.to_le_bytes();

            for i in 0..4 {
                bytes[i] = bytes[i].reverse_bits();
            }

            ctx.write_register(rd, u32::from_le_bytes(bytes));
        }
        (0b101, 0b0010100) if rs2 == 0b01111 => {
            ctx.inst_name("UNZIP");
            let mut result = 0;

            for i in 0..16 {
                result |= (a >> (2 * i) & 1) << i;
                result |= (a >> (2 * i + 1) & 1) << (i + 16);
            }

            ctx.write_register(rd, result);
        }
        (0b001, 0b0100100) if rs2 == 0b01111 => {
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
            ctx.write_register(rd, (a << shamt) | (a >> (32 - shamt)));
        }
        (0b101, 0b0110000) => {
            ctx.inst_name("ROR");
            let shamt = b & 0b11111;
            ctx.write_register(rd, (a >> shamt) | (a << (32 - shamt)));
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
    if func3(code) != 0b010 {
        ctx.raise_exception(Exception::IllegalInstruction);
    }

    let RType { rd, rs1, rs2 } = code.into();
    let func5 = code >> 27;
    let ordering = code >> 25 & 0b11;

    match func5 {
        0b00010 if rs2 == 0 => {
            ctx.inst_name("LR.W");
            // TODO
            return;
        }
        0b00011 => {
            ctx.inst_name("SC.W");
            // TODO
            return;
        }
        _ => {}
    }

    let a = ctx.read_register(rs1);
    let b = ctx.read_register(rs2);

    match func5 {
        0b00001 => {
            ctx.inst_name("AMOSWAP.W");
            // TODO
        }
        0b00000 => {
            ctx.inst_name("AMOADD.W");
            // TODO
        }
        0b00100 => {
            ctx.inst_name("AMOXOR.W");
            // TODO
        }
        0b01100 => {
            ctx.inst_name("AMOAND.W");
            // TODO
        }
        0b01000 => {
            ctx.inst_name("AMOOR.W");
            // TODO
        }
        0b10000 => {
            ctx.inst_name("AMOMIN.W");
            // TODO
        }
        0b10100 => {
            ctx.inst_name("AMOMAX.W");
            // TODO
        }
        0b11000 => {
            ctx.inst_name("AMOMINU.W");
            // TODO
        }
        0b11100 => {
            ctx.inst_name("AMOMAXU.W");
            // TODO
        }
        _ => {
            ctx.raise_exception(Exception::IllegalInstruction);
            return;
        }
    };
}

#[inline]
// Execute a compressed instruction
// maybe split this into multiple functions for better readability
fn exec_compressed_instruction(code: u16, ctx: &mut ExecContext) {
    match code & OPCODE_COMPRESSED_MASK {
        0b0000000000000000 => {
            ctx.inst_name("C.ADDI4SPN");
            // illegal if imm 0
            // TODO
        }
        0b0100000000000000 => {
            ctx.inst_name("C.LW");
            // TODO
        }
        0b1100000000000000 => {
            ctx.inst_name("C.SW");
            // TODO
        }
        0b0000000000000001 => {
            ctx.inst_name("C.ADDI");
            // TODO
        }
        0b0010000000000001 => {
            ctx.inst_name("C.JAL");
            // TODO
        }
        0b1010000000000001 => {
            ctx.inst_name("C.J");
            // TODO
        }
        0b0100000000000001 => {
            ctx.inst_name("C.LI");
            // TODO
        }
        0b0110000000000001 => {
            ctx.inst_name("C.LUI");
            // reserved if imm 0
            // TODO
        }
        0b1000000000000001 => match (code >> 10) & 0b111 {
            0b000 => {
                ctx.inst_name("C.SRLI");
                // imm[5] (instr[12]) must be 0, else reserved NSE
                // TODO
            }
            0b001 => {
                ctx.inst_name("C.SRAI");
                // imm[5] (instr[12]) must be 0, else reserved NSE
                // TODO
            }
            0b010 | 0b110 => {
                ctx.inst_name("C.ANDI");
                // TODO
            }
            0b011 => match (code >> 5) & 0b11 {
                0b00 => {
                    ctx.inst_name("C.SUB");
                    // TODO
                }
                0b01 => {
                    ctx.inst_name("C.XOR");
                    // TODO
                }
                0b10 => {
                    ctx.inst_name("C.OR");
                    // TODO
                }
                0b11 => {
                    ctx.inst_name("C.AND");
                    // TODO
                }
                _ => ctx.raise_exception(Exception::IllegalInstruction),
            },
            _ => ctx.raise_exception(Exception::IllegalInstruction),
        },

        0b1100000000000001 => {
            ctx.inst_name("C.BEQZ");
            // TODO
        }
        0b1110000000000001 => {
            ctx.inst_name("C.BNEZ");
            // TODO
        }

        0b0000000000000010 if code & (1 << 12) == 0 => {
            ctx.inst_name("C.SLLI");
            // imm[5] (instr[12]) must be 0, else reserved NSE
            // TODO
        }

        0b1000000000000010 if code & (1 << 12) == 0 => {
            ctx.inst_name("C.MV");
            // JR if rs2 == 0, if JR and rs1 == 0, reserved
            // TODO
        }
        0b1000000000000010 => {
            // else
            ctx.inst_name("C.MV");
            // JALR if rs2 == 0
            // EBREAK if !instr[11:2]
            // TODO
        }

        0b0100000000000010 => {
            ctx.inst_name("C.LWSP");
            // TODO
        }

        0b1100000000000010 => {
            ctx.inst_name("C.SWSP");
            // TODO
        }

        0b1000000000000000 => {
            match (code >> 12) & 0b111 {
                0b000 => {
                    ctx.inst_name("C.LBU");
                    // TODO
                }
                0b001 if code & (1 << 6) == 0 => {
                    ctx.inst_name("C.LHU");
                    // TODO
                }
                0b001 => {
                    // else
                    ctx.inst_name("C.LH");
                    // TODO
                }
                0b010 => {
                    ctx.inst_name("C.SB");
                    // TODO
                }
                0b011 if code & (1 << 6) == 0 => {
                    ctx.inst_name("C.SH");
                    // TODO
                }
                _ => ctx.raise_exception(Exception::IllegalInstruction),
            }
        }

        0b1000000000000001 if (code >> 10) & 0b111 == 0b111 => {
            match (code >> 2) & 0b11111 {
                0b11000 => {
                    ctx.inst_name("C.ZEXT.B");
                    // TODO
                }
                0b11001 => {
                    ctx.inst_name("C.SEXT.B");
                    // TODO
                }
                0b11010 => {
                    ctx.inst_name("C.ZEXT.H");
                    // TODO
                }
                0b11011 => {
                    ctx.inst_name("C.SEXT.H");
                    // TODO
                }
                0b11101 => {
                    ctx.inst_name("C.NOT");
                    // TODO
                }
                v if v >= 0b10000 && v < 0b11000 => {
                    ctx.inst_name("C.MUL");
                    // TODO
                }
                _ => ctx.raise_exception(Exception::IllegalInstruction),
            }
        }

        0b1010000000000010 => {
            if code & (1 << 13) == 0 {
                match code & 0b0000110001100000 {
                    0b110000100000 => {
                        ctx.inst_name("CM.MVSA01");
                        // TODO
                    }
                    0b110001100000 => {
                        ctx.inst_name("CM.MVA01S");
                        // TODO
                    }
                    _ => ctx.raise_exception(Exception::IllegalInstruction),
                }
            } else {
                match code & 0b0000111100000000 {
                    0b100000000000 => {
                        ctx.inst_name("CM.PUSH");
                        // TODO
                    }
                    0b101000000000 => {
                        ctx.inst_name("CM.POP");
                        // TODO
                    }
                    0b110000000000 => {
                        ctx.inst_name("CM.POPRETZ");
                        // TODO
                    }
                    0b111000000000 => {
                        ctx.inst_name("CM.POPRET");
                        // TODO
                    }
                    _ => ctx.raise_exception(Exception::IllegalInstruction),
                }
            }
        }

        _ => ctx.raise_exception(Exception::IllegalInstruction),
    }
}

pub(super) fn exec_instruction(code: u32, ctx: &mut ExecContext<'_>) {
    if code & 0b11 == 0b11 {
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
                let offset = ctx
                    .read_register(op.rs1)
                    .signed()
                    .wrapping_add(op.imm.signed())
                    & !1; // Clear the least significant bit

                ctx.write_register(op.rd, return_address);
                ctx.set_next_pc_offset(offset);
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
            _ if code == 0b00000000000000000001000000001111 => {
                ctx.inst_name("FENCE.I");
                // Do nothing
            }
            _ if extract_bits(code, 0..=19) == 0b00000000000000001111 => {
                ctx.inst_name("FENCE");
                // Do nothing
            }
            _ => ctx.raise_exception(Exception::IllegalInstruction),
        }
    } else {
        ctx.set_next_pc_offset(2);
        exec_compressed_instruction(code as u16, ctx);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bus::Bus;
    use crate::processor::*;

    #[test]
    fn test_lui() {
        let mut core = Hazard3::default();
        let mut bus = Bus::new();
        let mut ctx = ExecContext::new(&mut core, &mut bus);

        exec_instruction(0b00000001100100100101111110110111, &mut ctx);

        assert_eq!(ctx.register_write, Some((31, 6437 << 12)));
    }

    #[test]
    fn test_auipc() {
        let mut core = Hazard3::default();
        core.set_pc(0x1000);
        let mut bus = Bus::new();
        let mut ctx = ExecContext::new(&mut core, &mut bus);

        exec_instruction(0b00000001100100100101111110010111, &mut ctx);

        assert_eq!(ctx.register_write, Some((31, 0x1000 + (6437 << 12))));
    }

    #[test]
    fn test_jal() {
        let mut core = Hazard3::default();
        core.set_pc(0x1000);
        let mut bus = Bus::new();
        let mut ctx = ExecContext::new(&mut core, &mut bus);

        exec_instruction(0b00010010010100000001111111101111, &mut ctx);

        assert_eq!(ctx.register_write, Some((31, 0x1000 + 4)));
        assert_eq!(ctx.next_pc_offset, 6437 & !1);
    }

    #[test]
    fn test_jalr() {
        let mut core = Hazard3::default();
        core.set_pc(0x1000);
        core.registers.write(24, 0x2000);
        let mut bus = Bus::new();
        let mut ctx = ExecContext::new(&mut core, &mut bus);

        exec_instruction(0b11011100110000000000110001100111, &mut ctx);

        assert_eq!(ctx.register_write, Some((24, 0x1000 + 4)));
        assert_eq!(ctx.next_pc_offset, -564 & !1);
    }

    // TODO: Add more tests
}
