use super::instruction_format::*;
use core::ops::RangeInclusive;

pub enum Instruction {
    Ebreak,
    Ecall,
    Fence,
    FenceI,
    Mret,
    Wfi,

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

    CmMva01s(CmmvType),
    CmMvsa01(CmmvType),
    CmPop(CmppType),
    CmPopret(CmppType),
    CmPopretz(CmppType),
    CmPush(CmppType),

    // TODO
    Bclr(RType),
    Bext(RType),
    Bclri(RType),
    Bexti(RType),
    Binv(RType),
    Binvi(RType),
    Bset(RType),
    Bseti(RType),

    // Zbkb crypto
    Pack(RType),
    Packh(RType),
    Brev8(RType),
    Unzip(RType),
    Zip(RType),

    Csrrw(RType),
    Csrrs(RType),
    Csrrc(RType),
    Csrrwi(RType),
    Csrrsi(RType),
    Csrrci(RType),
    // ZextB(Data), // From ZCB, compressible if rd matches rs1 and registers are in x8-x15, pseudo for andi if 32bit instruction
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
        let opcode = inst & 0b11;
        let func3 = || extract_bits(inst, 13..=15);
        let func6 = || extract_bits(inst, 10..=15);
        let rs2 = || extract_bits(inst, 20..=24);

        match opcode {
            0b00 => todo!(),
            0b01 => todo!(),
            0b10 => match func3() {
                0b101 => match (
                    extract_bits(inst, 10..=12),
                    extract_bits(inst, 8..=9),
                    extract_bits(inst, 5..=6),
                ) {
                    (0b110, 0b00, _) => Self::CmPush(inst.into()),
                    (0b110, 0b10, _) => Self::CmPop(inst.into()),
                    (0b111, 0b00, _) => Self::CmPopretz(inst.into()),
                    (0b111, 0b10, _) => Self::CmPopret(inst.into()),
                    (0b011, _, 0b01) => Self::CmMvsa01(inst.into()),
                    (0b011, _, 0b11) => Self::CmMva01s(inst.into()),
                    _ => todo!(),
                },

                // SLLI
                // MV
                // ADD
                // LWSP
                // SWSP
                _ => todo!(),
            },
        }
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
            0b00110000001000000000000001110011 => Self::Mret,  // MRET
            0b00010000010100000000000001110011 => Self::Wfi,   // WFI
            0b00000000000000000001000000001111 => Self::FenceI, // FENCE.I
            _ if extract_bits(inst, 0..=19) == 0b00000000000000001111 => Self::Fence, // FENCE
            _ => match opcode() {
                // 0b0001111 if func3() == 0 => Self::Fence,
                0b0110111 => Self::Lui(inst.into()),   // LUI
                0b0010111 => Self::Auipc(inst.into()), // AUIPC
                0b1101111 => Self::Jal(inst.into()),   // JAL
                0b1100111 => match func3() {
                    0b000 => Self::Jalr(inst.into()), // JALR
                    _ => unreachable!(),
                },

                0b1110011 => match func3() {
                    0b001 => Self::Csrrw(inst.into()),
                    0b010 => Self::Csrrs(inst.into()),
                    0b011 => Self::Csrrc(inst.into()),
                    0b101 => Self::Csrrwi(inst.into()),
                    0b110 => Self::Csrrsi(inst.into()),
                    0b111 => Self::Csrrci(inst.into()),
                    _ => unreachable!(),
                },

                // Branch Instructions (B-Type)
                0b1100011 => match func3() {
                    0b000 => Self::Beq(inst.into()),  // BEQ
                    0b001 => Self::Bne(inst.into()),  // BNE
                    0b100 => Self::Blt(inst.into()),  // BLT
                    0b101 => Self::Bge(inst.into()),  // BGE
                    0b110 => Self::Bltu(inst.into()), // BLTU
                    0b111 => Self::Bgeu(inst.into()), // BGEU
                    _ => unreachable!(),
                },

                // Load Instructions (I-Type)
                0b0000011 => match func3() {
                    0b000 => Self::Lb(inst.into()),  // LB
                    0b001 => Self::Lh(inst.into()),  // LH
                    0b010 => Self::Lw(inst.into()),  // LW
                    0b100 => Self::Lbu(inst.into()), // LBU
                    0b101 => Self::Lhu(inst.into()), // LHU
                    _ => unreachable!(),
                },

                // Store Instructions (S-Type)
                0b0100011 => match func3() {
                    0b000 => Self::Sb(inst.into()), // SB
                    0b001 => Self::Sh(inst.into()), // SH
                    0b010 => Self::Sw(inst.into()), // SW
                    _ => unreachable!(),
                },

                // Arithmetic Immediate (I-Type)
                0b0010011 => match (func3(), func7()) {
                    (0b000, _) => Self::Addi(inst.into()),  // ADDI
                    (0b010, _) => Self::Slti(inst.into()),  // SLTI
                    (0b011, _) => Self::Sltiu(inst.into()), // SLTIU
                    (0b100, _) => Self::Xori(inst.into()),  // XORI
                    (0b110, _) => Self::Ori(inst.into()),   // ORI
                    (0b111, _) => Self::Andi(inst.into()),  // ANDI
                    // Zbs
                    (0b001, 0b0100100) => Self::Bclri(inst.into()),
                    (0b101, 0b0100100) => Self::Bexti(inst.into()),
                    (0b001, 0b0110100) => Self::Binvi(inst.into()),
                    (0b001, 0b0010100) => Self::Bseti(inst.into()),

                    (0b001, 0b0000000) => Self::Slli(inst.into()), // SLLI
                    // Zbb extension
                    (0b001, 0b0110000) => match rs2() {
                        0b00000 => Self::Clz(inst.into()),
                        0b00010 => Self::Cpop(inst.into()),
                        0b00001 => Self::Ctz(inst.into()),
                        0b00100 => Self::SextB(inst.into()),
                        0b00101 => Self::SextH(inst.into()),
                        _ => unreachable!(),
                    },
                    (0b101, 0b0000000) => Self::Srli(inst.into()), // SRLI
                    (0b101, 0b0100000) => Self::Srai(inst.into()), // SRAI
                    (0b101, 0b0010100) if rs2() == 0b00111 => Self::OrcB(inst.into()),
                    (0b101, 0b0110100) if rs2() == 0b11000 => Self::Rev8(inst.into()),
                    (0b101, 0b0110000) => Self::Rori(inst.into()),

                    // Zbkb
                    (0b101, 0b0000100) if rs2() == 0b00111 => Self::Brev8(inst.into()),
                    (0b101, 0b0010100) if rs2() == 0b01111 => Self::Unzip(inst.into()),
                    (0b001, 0b0100100) if rs2() == 0b01111 => Self::Zip(inst.into()),
                    _ => unreachable!(),
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
                    (0b010, 0b0010000) => Self::Sh1add(inst.into()), // SH1ADD
                    (0b100, 0b0010000) => Self::Sh2add(inst.into()), // SH2ADD
                    (0b110, 0b0010000) => Self::Sh3add(inst.into()), // SH3ADD

                    // Zbs extension
                    (0b001, 0b0100100) => Self::Bclr(inst.into()), // BCLR
                    (0b101, 0b0100100) => Self::Bext(inst.into()), // BEXT
                    (0b001, 0b0110100) => Self::Binv(inst.into()), // BINV
                    (0b001, 0b0010100) => Self::Bset(inst.into()), // BSET

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
                    (0b100, 0b0000100) if rs2() == 0 => Self::ZextH(inst.into()),

                    // Zbkb
                    (0b100, 0b0000100) => Self::Pack(inst.into()),
                    (0b111, 0b0000100) => Self::Packh(inst.into()),

                    _ => unreachable!(),
                },

                // A Standard Extension
                0b010111 if func3() == 0b010 => match extract_bits(inst, 27..=31) {
                    0b00010 if rs2() == 0 => Self::LrW(inst.into()), // LR.W
                    0b00011 => Self::ScW(inst.into()),               // SC.W
                    0b00001 => Self::AmoswapW(inst.into()),          // AMOSWAP.W
                    0b00000 => Self::AmoaddW(inst.into()),           // AMOADD.W
                    0b00100 => Self::AmoxorW(inst.into()),           // AMOXOR.W
                    0b01100 => Self::AmoandW(inst.into()),           // AMOAND.W
                    0b01000 => Self::AmoorW(inst.into()),            // AMOOR.W
                    0b10000 => Self::AmominW(inst.into()),           // AMOMIN.W
                    0b10100 => Self::AmomaxW(inst.into()),           // AMOMAX.W
                    0b11000 => Self::AmominuW(inst.into()),          // AMOMINU.W
                    0b11100 => Self::AmomaxuW(inst.into()),          // AMOMAXU.W
                    _ => unreachable!(),
                },

                _ => unreachable!(),
            },
        }
    }
}
