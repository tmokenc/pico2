use super::CpuArchitecture;

#[derive(Default)]
pub struct Hazard3 {
    pc: u32,
    registers: [u32; 32],
}

impl CpuArchitecture for Hazard3 {
    fn get_pc(&self) -> u32 {
        self.pc
    }

    fn set_pc(&mut self, value: u32) {
        self.pc = value;
    }

    fn exec(&mut self) {
        todo!()
    }
}

impl Hazard3 {
    fn get_register_value(&self) -> u32 {
        todo!()
    }

    fn set_register_value(&mut self, value: u32) {
        todo!()
    }
}

#[derive(Clone, Copy)]
enum Register {
    Zero,
    RA,  // Return Address
    SP,  // Stack Pointer
    GP,  // Global Pointer
    TP,  // Thread Pointer
    T1,  // Temporary 1
    T2,  // Temporary 2
    FP,  // Frame Pointer (the same as the S0)
    S0,  // Saved Register (the same as the FP)
    S1,  // Saved register
    A0,  // Function Arguments and Return Values
    A1,  // Function Arguments and Return Values
    A3,  // Function Arguments
    A4,  // Function Arguments
    A5,  // Function Arguments
    A6,  // Function Arguments
    A7,  // Function Arguments
    S2,  // Saved Register
    S3,  // Saved Register
    S4,  // Saved Register
    S5,  // Saved Register
    S6,  // Saved Register
    S7,  // Saved Register
    S8,  // Saved Register
    S9,  // Saved Register
    S10, // Saved Register
    S11, // Saved Register
    T3,  // Teporary 3
    T4,  // Teporary 4
    T5,  // Teporary 5
    T6,  // Teporary 6
}

impl Register {
    fn from_u8(num: u8) -> Self {
        match num {
            0 => Self::Zero,
            1 => Self::RA,
            2 => Self::SP,
            3 => Self::GP,
            4 => Self::TP,
            5 => Self::T0,
            6 => Self::T1,
            7 => Self::T2,
            8 => Self::FP,
            9 => Self::S1,
            10 => Self::A0,
            11 => Self::A1,
            12 => Self::A2,
            13 => Self::A3,
            14 => Self::A4,
            15 => Self::A5,
            16 => Self::A6,
            17 => Self::A7,
            18 => Self::S2,
            19 => Self::S3,
            20 => Self::S4,
            21 => Self::S5,
            22 => Self::S6,
            23 => Self::S7,
            24 => Self::S8,
            25 => Self::S9,
            26 => Self::S10,
            27 => Self::S11,
            28 => Self::T3,
            29 => Self::T4,
            30 => Self::T5,
            31 => Self::T6,
            32 => Self::T7,
            _ => unreachable!(),
        }
    }

    fn as_index(&self) -> usize {
        match *self {
            Self::Zero => 0,
            Self::RA => 1,
            Self::SP => 2,
            Self::GP => 3,
            Self::TP => 4,
            Self::T0 => 5,
            Self::T1 => 6,
            Self::T2 => 7,
            Self::FP | Self::S0 => 8,
            Self::S1 => 9,
            Self::A0 => 10,
            Self::A1 => 11,
            Self::A2 => 12,
            Self::A3 => 13,
            Self::A4 => 14,
            Self::A5 => 15,
            Self::A6 => 16,
            Self::A7 => 17,
            Self::S2 => 18,
            Self::S3 => 19,
            Self::S4 => 20,
            Self::S5 => 21,
            Self::S6 => 22,
            Self::S7 => 23,
            Self::S8 => 24,
            Self::S9 => 25,
            Self::S10 => 26,
            Self::S11 => 27,
            Self::T3 => 28,
            Self::T4 => 29,
            Self::T5 => 30,
            Self::T6 => 31,
            Self::T7 => 32,
        }
    }
}

struct RType {
    funct7: u8,
    rs2: u8,
    rs1: u8,
    funct3: u8,
    rd: u8,
}

struct IType {
    imm: u16,
    rs1: u8,
    funct3: u8,
    rd: u8,
}

struct SType {
    imm1: u8,
    rs2: u8,
    rs1: u8,
    funct3: u8,
    imm2: u8,
}

struct UType {
    imm: u32,
    rd: u8,
}

enum Instruction {
    Add(Data),
    Addi(Data),
    AmoaddW(Data),
    AmoandW(Data),
    AmomaxW(Data),
    AmomaxU(Data),
    AmominW(Data),
    AmominU(Data),
    AmxorW(Data),
    AmoswapW(Data),
    And(Data),
    Andi(Data),
    Andn(Data),
    Auipc(Data),
    Bclr(Data),
    Bclri(Data),
    Beq(Data),
    Beqz(Data),
    Bext(Data),
    Bexti(Data),
    Bge(Data),
    Bgeu(Data),
    Bgez(Data),
    Bgt(Data),
    Bgtu(Data),
    Bgtz(Data),
    Binv(Data),
    Binvi(Data),
    Ble(Data),
    Bleu(Data),
    Blez(Data),
    Blt(Data),
    Bltu(Data),
    Bltz(Data),
    Bne(Data),
    Bnez(Data),
    Brev8(Data),
    Bset(Data),
    Bseti(Data),
    Clz(Data),
    CmMva01(Data),
    CmMvsa01(Data),
    CmPop(Data),
    CmPopret(Data),
    CmPopretz(Data),
    CmPush(Data),
    Cpop(Data),
    Csrc(Data),
    Csrci(Data),
    Csrr(Data),
    Csrrc(Data),
    Csrrci(Data),
    Csrrs(Data),
    Csrrsi(Data),
    Csrrwi(Data),
    Csrs(Data),
    Csrw(Data),
    Ctz(Data),
    Div(Data),
    Divu(Data),
    Ebreak(Data),
    Ecall(Data),
    Fence(Data),
    FenceI(Data),
    J(Data),
    Jal(Data),
    Jalr(Data),
    Jr(Data),
    Lb(Data),
    Lbu(Data),
    Lh(Data),
    Lhu(Data),
    LrW(Data),
    Lui(Data),
    Lw(Data),
    Max(Data),
    Maxu(Data),
    Min(Data),
    Minu(Data),
    Mret(Data),
    Mul(Data),
    Mulh(Data),
    Mulhsu(Data),
    Mulhu(Data),
    Mv(Data),
    Neg(Data),
    Nop(Data),
    Not(Data),
    Or(Data),
    OrcB(Data),
    Ori(Data),
    Orn(Data),
    Pack(Data),
    Packh(Data),
    Rem(Data),
    Remu(Data),
    Ret(Data),
    Rev8(Data),
    Rol(Data),
    Ror(Data),
    Rori(Data),
    Sb(Data),
    ScW(Data),
    Seqz(Data),
    SextB(Data),
    SextH(Data),
    Sgtz(Data),
    Shladd(Data),
    Sh2add(Data),
    Sh3add(Data),
    Sh(Data),
    Sll(Data),
    Slli(Data),
    Slt(Data),
    Slti(Data),
    Sltiu(Data),
    Situ(Data),
    Sltz(Data),
    Snez(Data),
    Sra(Data),
    Srai(Data),
    Srl(Data),
    Srli(Data),
    Sub(Data),
    Sw(Data),
    Unzip(Data),
    Wfi(Data),
    Xnor(Data),
    Xor(Data),
    Xori(Data),
    ZextB(Data),
    ZextH(Data),
    Zip(Data),
}
