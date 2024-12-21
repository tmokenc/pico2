use super::{Register, Registers};

pub(super) trait OpValue {
    type Value: Default;

    fn get(&self, registers: &Registers) -> Self::Value {
        Default::default()
    }
}

impl OpValue for RType {
    type Value = (u32, u32);

    fn get(&self, registers: &Registers) -> Self::Value {
        (registers.read(self.rs1), registers.read(self.rs2))
    }
}

impl OpValue for IType {
    type Value = (u32, u32);

    fn get(&self, registers: &Registers) -> Self::Value {
        (registers.read(self.rs1), self.imm)
    }
}

impl OpValue for SType {
    type Value = (u32, u32, u32);

    fn get(&self, registers: &Registers) -> Self::Value {
        (registers.read(self.rs1), registers.read(self.rs2), self.imm)
    }
}

impl OpValue for BType {
    type Value = (u32, u32, u32);

    fn get(&self, registers: &Registers) -> Self::Value {
        (registers.read(self.rs1), registers.read(self.rs2), self.imm)
    }
}

impl OpValue for UType {
    type Value = u32;

    fn get(&self, registers: &Registers) -> Self::Value {
        self.imm
    }
}

impl OpValue for JType {
    type Value = u32;

    fn get(&self, registers: &Registers) -> Self::Value {
        self.imm
    }
}

impl OpValue for ZbbType {
    type Value = u32;

    fn get(&self, registers: &Registers) -> Self::Value {
        registers.read(self.rs1)
    }
}

#[derive(Default)]
pub(super) struct RType {
    pub(super) rs2: Register,
    pub(super) rs1: Register,
    pub(super) rd: Register,
}

#[derive(Default)]
pub(super) struct IType {
    pub(super) imm: u32,
    pub(super) rs1: Register,
    pub(super) rd: Register,
}

#[derive(Default)]
pub(super) struct SType {
    pub(super) imm: u32,
    pub(super) rs2: Register,
    pub(super) rs1: Register,
}

#[derive(Default)]
pub(super) struct BType {
    pub(super) imm: u32,
    pub(super) rs2: Register,
    pub(super) rs1: Register,
}

#[derive(Default)]
pub(super) struct UType {
    pub(super) imm: u32,
    pub(super) rd: Register,
}

#[derive(Default)]
pub(super) struct JType {
    pub(super) imm: u32,
    pub(super) rd: Register,
}

#[derive(Default)]
pub(super) struct ZbbType {
    pub(super) rd: Register,
    pub(super) rs1: Register,
}

impl From<u32> for RType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7, 11),
            rs1: extract_bits(inst, 15, 19),
            rs2: extract_bits(inst, 20, 24),
        }
    }
}

impl From<u32> for IType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7, 11),
            rs1: extract_bits(inst, 15, 19),
            imm: load_immediate(inst, &[(20, 30, 0)], 31),
        }
    }
}

impl From<u32> for SType {
    fn from(inst: u32) -> Self {
        Self {
            rs1: extract_bits(inst, 15, 19),
            rs2: extract_bits(inst, 20, 24),
            imm: load_immediate(inst, &[(7, 11, 0), (25, 30, 5)], 31),
        }
    }
}

impl From<u32> for BType {
    fn from(inst: u32) -> Self {
        Self {
            rs1: extract_bits(inst, 15, 19),
            rs2: extract_bits(inst, 20, 24),
            imm: load_immediate(inst, &[(7, 7, 11), (8, 11, 1), (25, 30, 5)], 31),
        }
    }
}

impl From<u32> for UType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7, 11),
            imm: load_immediate(inst, &[(12, 30, 12)], 31),
        }
    }
}

impl From<u32> for JType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7, 11),
            imm: load_immediate(inst, &[(12, 19, 12), (20, 20, 11), (21, 30, 1)], 31),
        }
    }
}

impl From<u32> for ZbbType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7, 11),
            rs1: extract_bits(inst, 15, 19),
        }
    }
}

pub enum Instruction {
    // RV32I
    Ebreak,
    Ecall,
    Fence,

    Lui(UType),
    Auipc(UType),
    Jal(JType),
    Jalr(IType),

    Beq(BType),
    Bne(BType),
    Blt(BType),
    Bge(BType),
    Bltu(BType),
    Bgeu(BType),

    Lb(IType),
    Lh(IType),
    Lw(IType),
    Lbu(IType),
    Lhu(IType),

    Sb(SType),
    Sh(SType),
    Sw(SType),

    Addi(IType),
    Slti(IType),
    Sltiu(IType),
    Xori(IType),
    Ori(IType),
    Andi(IType),

    Slli(RType),
    Srli(RType),
    Srai(RType),

    Add(RType),
    Sub(RType),
    Sll(RType),
    Slt(RType),
    Sltu(RType),
    Xor(RType),
    Srl(RType),
    Sra(RType),
    Or(RType),
    And(RType),

    // M standard extension
    Mul(RType),
    Mulh(RType),
    Mulhsu(RType),
    Mulhu(RType),
    Div(RType),
    Divu(RType),
    Rem(RType),
    Remu(RType),
    // Zba extension
    Sh1add(RType),
    Sh2add(RType),
    Sh3add(RType),

    // A standard extension
    LrW(RType),
    ScW(RType),
    AmoaddW(RType),
    AmoandW(RType),
    AmomaxW(RType),
    AmomaxuW(RType),
    AmominW(RType),
    AmominuW(RType),
    AmoxorW(RType),
    AmoorW(RType),
    AmoswapW(RType),

    // Zbb extension
    Andn(RType),
    Clz(ZbbType),
    Cpop(ZbbType),
    Ctz(ZbbType),
    Max(RType),
    Maxu(RType),
    Min(RType),
    Minu(RType),
    OrcB(ZbbType),
    Orn(RType),
    Rev8(ZbbType),
    Rol(RType),
    Ror(RType),
    Rori(RType),
    SextB(ZbbType),
    SextH(ZbbType),
    Xnor(RType),
    ZextH(ZbbType),
    // TODO Zbs and more from the rp2350 spec
    // Bclr(Data),
    // Bclri(Data),
    // Beqz(Data),
    // Bext(Data),
    // Bexti(Data),
    // Bgez(Data),
    // Bgt(Data),
    // Bgtu(Data),
    // Bgtz(Data),
    // Binv(Data),
    // Binvi(Data),
    // Ble(Data),
    // Bleu(Data),
    // Blez(Data),
    // Bltz(Data),
    // Bnez(Data),
    // Brev8(Data),
    // Bset(Data),
    // Bseti(Data),
    // CmMva01(Data),
    // CmMvsa01(Data),
    // CmPop(Data),
    // CmPopret(Data),
    // CmPopretz(Data),
    // CmPush(Data),
    // Csrc(Data),
    // Csrci(Data),
    // Csrr(Data),
    // Csrrc(Data),
    // Csrrci(Data),
    // Csrrs(Data),
    // Csrrsi(Data),
    // Csrrwi(Data),
    // Csrs(Data),
    // Csrw(Data),
    // FenceI(Data),
    // J(Data),
    // Jr(Data),
    // Mret(Data),
    // Mv(Data),
    // Neg(Data),
    // Nop(Data),
    // Not(Data),
    // Pack(Data),
    // Packh(Data),
    // Ret(Data),
    // Seqz(Data),
    // Sgtz(Data),
    // Sltz(Data),
    // Snez(Data),
    // Unzip(Data),
    // Wfi(Data),
    // ZextB(Data), // From ZCB
    // Zip(Data),
}

const fn extract_bits(num: u32, from: u32, to: u32) -> u32 {
    (num >> from) & ((1 << (to - from + 1)) - 1)
}

fn load_immediate(raw: u32, positions: &[(u32, u32, u32)], sign_bit_position: usize) -> u32 {
    let mut value = 0;

    for &(start, end, placement) in positions {
        let bits = extract_bits(raw, start, end);
        value |= bits << placement;
    }

    // Handle sign extension if necessary
    if sign_bit_position < 32 && ((value >> sign_bit_position) & 1) == 1 {
        // Extend the sign bit to the left
        let sign_extension_mask = !((1u32 << sign_bit_position) - 1);
        value |= sign_extension_mask;
    }

    value
}

const fn sign_extend(num: u32, from: u32, to: u32) -> u32 {
    let shift = 32 - to;
    (num << shift) as i32 as u32 >> shift
}

pub(super) fn is_16_bit_inst(inst: u32) -> bool {
    inst & 0b11 == 0b11
}

impl Instruction {
    pub(crate) fn decode(inst: u32) -> Self {
        if is_16_bit_inst(inst) {
            Self::from(inst as u16)
        } else {
            Self::from(inst)
        }
    }
}

impl From<u16> for Instruction {
    fn from(inst: u16) -> Self {
        todo!()
    }
}

impl From<u32> for Instruction {
    fn from(inst: u32) -> Self {
        let opcode = || inst & 0b1111111;
        let func3 = || extract_bits(inst, 12, 14);
        let func7 = || extract_bits(inst, 25, 31);

        match inst {
            0b00000000000000000000000001110011 => Self::Ecall, // ECALL
            0b00000000000100000000000001110011 => Self::Ebreak, // Ebreak
            _ => match opcode() {
                0b0001111 if func3() == 0 => Self::Fence,

                0b0110111 => Self::Lui(inst.into()),   // LUI
                0b0010111 => Self::Auipc(inst.into()), // AUIPC
                0b1101111 => Self::Jal(inst.into()),   // JAL
                0b1100111 => Self::Jalr(inst.into()),  // JALR

                // Branch Instructions (B-Type)
                0b1100011 => match func3() {
                    0b000 => Self::Beq(inst.into()),  // BEQ
                    0b001 => Self::Bne(inst.into()),  // BNE
                    0b100 => Self::Blt(inst.into()),  // BLT
                    0b101 => Self::Bge(inst.into()),  // BGE
                    0b110 => Self::Bltu(inst.into()), // BLTU
                    0b111 => Self::Bgeu(inst.into()), // BGEU
                    _ => todo!(),
                },

                // Load Instructions (I-Type)
                0b0000011 => match func3() {
                    0b000 => Self::Lb(inst.into()),  // LB
                    0b001 => Self::Lh(inst.into()),  // LH
                    0b010 => Self::Lw(inst.into()),  // LW
                    0b100 => Self::Lbu(inst.into()), // LBU
                    0b101 => Self::Lhu(inst.into()), // LHU
                    _ => todo!(),
                },

                // Store Instructions (S-Type)
                0b0100011 => match func3() {
                    0b000 => Self::Sb(inst.into()), // SB
                    0b001 => Self::Sh(inst.into()), // SH
                    0b010 => Self::Sw(inst.into()), // SW
                    _ => todo!(),
                },

                // Arithmetic Immediate (I-Type)
                0b0010011 => match func3() {
                    0b000 => Self::Addi(inst.into()),  // ADDI
                    0b010 => Self::Slti(inst.into()),  // SLTI
                    0b011 => Self::Sltiu(inst.into()), // SLTIU
                    0b100 => Self::Xori(inst.into()),  // XORI
                    0b110 => Self::Ori(inst.into()),   // ORI
                    0b111 => Self::Andi(inst.into()),  // ANDI
                    0b001 => match func7() {
                        0b0000000 => Self::Slli(inst.into()), // SLLI
                        // Zbb extension
                        0b0110000 => match extract_bits(inst, 20, 24) {
                            0b00000 => Self::Clz(inst.into()),
                            0b00010 => Self::Cpop(inst.into()),
                            0b00001 => Self::Ctz(inst.into()),
                            0b00100 => Self::SextB(inst.into()),
                            0b00101 => Self::SextH(inst.into()),
                            _ => unreachable!(),
                        },
                        _ => unreachable!(),
                    },
                    0b101 => match func7() {
                        0b0000000 => Self::Srli(inst.into()), // SRLI
                        0b0100000 => Self::Srai(inst.into()), // SRAI
                        0b0010100 if extract_bits(inst, 20, 24) == 0b00111 => {
                            Self::OrcB(inst.into())
                        }
                        0b0110100 if extract_bits(inst, 20, 24) == 0b11000 => {
                            Self::Rev8(inst.into())
                        }
                        0b0110000 => Self::Rori(inst.into()),
                        _ => todo!(),
                    },
                    _ => todo!(),
                },

                // Register-Register Arithmetic (R-Type)
                0b0110011 => match (func3(), func7()) {
                    (0b000, 0b0000000) => Self::Add(inst.into()),  // ADD
                    (0b000, 0b0100000) => Self::Sub(inst.into()),  // SUB
                    (0b001, 0b0000000) => Self::Sll(inst.into()),  // SLL
                    (0b010, 0b0000000) => Self::Slt(inst.into()),  // SLT
                    (0b011, 0b0000000) => Self::Sltu(inst.into()), // SLTU
                    (0b100, 0b0000000) => Self::Xor(inst.into()),  // XOR
                    (0b101, 0b0000000) => Self::Srl(inst.into()),  // SRL
                    (0b101, 0b0100000) => Self::Sra(inst.into()),  // SRA
                    (0b110, 0b0000000) => Self::Or(inst.into()),   // OR
                    (0b111, 0b0000000) => Self::And(inst.into()),  // AND
                    // M standard extension
                    (0b000, 0b0000001) => Self::Mul(inst.into()), // MUL
                    (0b001, 0b0000001) => Self::Mulh(inst.into()), // MULH
                    (0b010, 0b0000001) => Self::Mulhsu(inst.into()), // MULHSU
                    (0b011, 0b0000001) => Self::Mulhu(inst.into()), // MULHU
                    (0b100, 0b0000001) => Self::Div(inst.into()), // DIV
                    (0b101, 0b0000001) => Self::Divu(inst.into()), // DIVU
                    (0b110, 0b0000001) => Self::Rem(inst.into()), // REM
                    (0b111, 0b0000001) => Self::Remu(inst.into()), // REMU

                    // Zba extension
                    (0b010, 0b0010000) => Self::Sh1add(inst.into()),
                    (0b100, 0b0010000) => Self::Sh2add(inst.into()),
                    (0b110, 0b0010000) => Self::Sh3add(inst.into()),

                    // Zbb extension
                    (0b111, 0b0100000) => Self::Andn(inst.into()),
                    (0b110, 0b0000101) => Self::Max(inst.into()),
                    (0b111, 0b0000101) => Self::Maxu(inst.into()),
                    (0b100, 0b0000101) => Self::Min(inst.into()),
                    (0b101, 0b0000101) => Self::Minu(inst.into()),
                    (0b110, 0b0100000) => Self::Orn(inst.into()),
                    (0b001, 0b0110000) => Self::Rol(inst.into()),
                    (0b101, 0b0110000) => Self::Ror(inst.into()),
                    (0b100, 0b0100000) => Self::Xnor(inst.into()),
                    (0b100, 0b0000100) if extract_bits(inst, 20, 24) == 0 => {
                        Self::ZextH(inst.into())
                    }

                    _ => todo!(),
                },

                // A Standard Extension
                0b010111 if func3() == 0b010 => match extract_bits(inst, 27, 31) {
                    0b00010 if extract_bits(inst, 20, 24) == 0 => Self::LrW(inst.into()), // LR.W
                    0b00011 => Self::ScW(inst.into()),                                    // SC.W
                    0b00001 => Self::AmoswapW(inst.into()), // AMOSWAP.W
                    0b00000 => Self::AmoaddW(inst.into()),
                    0b00100 => Self::AmoxorW(inst.into()),
                    0b01100 => Self::AmoandW(inst.into()),
                    0b01000 => Self::AmoorW(inst.into()),
                    0b10000 => Self::AmominW(inst.into()),
                    0b10100 => Self::AmomaxW(inst.into()),
                    0b11000 => Self::AmominuW(inst.into()),
                    0b11100 => Self::AmomaxuW(inst.into()),
                    _ => todo!(),
                },

                // Zba extension (bit manipulation)

                // Default case for unimplemented opcodes
                _ => todo!(),
            },
        }
    }
}
