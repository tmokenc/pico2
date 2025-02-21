use super::{Register, Registers};
use crate::utils::{extract_bits, sign_extend};
use core::ops::RangeInclusive;

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

pub(self) fn rs1(inst: u32) -> Register {
    extract_bits(inst, 15..=19) as Register
}

pub(self) fn rs2(inst: u32) -> Register {
    extract_bits(inst, 20..=24) as Register
}

pub(self) fn rd(inst: u32) -> Register {
    extract_bits(inst, 7..=11) as Register
}

impl From<u32> for RType {
    fn from(inst: u32) -> Self {
        Self {
            rd: rd(inst),
            rs1: rs1(inst),
            rs2: rs2(inst),
        }
    }
}

impl From<u32> for IType {
    fn from(inst: u32) -> Self {
        Self {
            rd: rd(inst),
            rs1: rs1(inst),
            imm: load_imm(inst, &[20..=31], true),
        }
    }
}

impl From<u32> for SType {
    fn from(inst: u32) -> Self {
        Self {
            rs1: rs1(inst),
            rs2: rs2(inst),
            imm: load_imm(inst, &[25..=31, 7..=11], true),
        }
    }
}

impl From<u32> for BType {
    fn from(inst: u32) -> Self {
        Self {
            rs1: rs1(inst),
            rs2: rs2(inst),
            imm: load_imm(inst, &[31..=31, 7..=7, 25..=30, 8..=11], true) << 1,
        }
    }
}

impl From<u32> for UType {
    fn from(inst: u32) -> Self {
        Self {
            rd: rd(inst),
            imm: inst & 0xfffff000, // load_imm(inst, &[12..=31], true) << 12,
        }
    }
}

impl From<u32> for JType {
    fn from(inst: u32) -> Self {
        Self {
            rd: rd(inst),
            imm: load_imm(inst, &[31..=31, 12..=19, 20..=20, 21..=30], true) << 1,
        }
    }
}

// ====== Compresed Instruction Formats ======
pub(super) struct CRType {
    pub(super) rs2: Register,
    pub(super) rs1: Register, // rd = rs1
}

pub(super) struct CIType {
    pub(super) imm: u32,
    pub(super) rs1: Register, // rd = rs1
}

pub(super) struct CSSType {
    pub(super) imm: u32,
    pub(super) rs2: Register,
}

pub(super) struct CIWType {
    pub(super) imm: u32,
    pub(super) rd: Register,
}

pub(super) struct CLType {
    pub(super) imm: u32,
    pub(super) rd: Register,
    pub(super) rs1: Register,
}

pub(super) struct CSType {
    pub(super) imm: u32,
    pub(super) rs2: Register,
    pub(super) rs1: Register,
}

pub(super) struct CAType {
    pub(super) rs2: Register,
    pub(super) rs1: Register, // rd = rs1
}

pub(super) struct CBType {
    pub(super) offset: u32,
    pub(super) rs1: Register,
}

pub(super) struct CJType {
    pub(super) jump_target: u32,
}

impl From<u16> for CRType {
    fn from(inst: u16) -> Self {
        Self {
            rs2: crs2(inst),
            rs1: crs1(inst),
        }
    }
}

// impl From<u16> for CIType {
//     fn from(inst: u16) -> Self {
//         Self {
//             rd: extract_bits(inst as u32, 7..=11),
//             imm: load_imm(inst as u32, &[12..=12, 5..=6, 10..=11], true),
//         }
//     }
// }

impl From<u16> for CAType {
    fn from(inst: u16) -> Self {
        Self {
            rs2: crs2_(inst),
            rs1: crs1_(inst),
        }
    }
}

// impl From<u16> for CBType {
//     fn from(inst: u16) -> Self {
//         Self {
//             offset: load_imm(inst as u32, &[12..=12, 5..=8, 10..=10, 3..=4, 1..=2], true) << 1,
//             rs1: crs1_(inst),
//         }
//     }
// }
//
// impl From<u16> for CJType {
//     fn from(inst: u16) -> Self {
//         Self {
//             jump_target: load_imm(inst as u32, &[12..=12, 8..=11, 1..=4, 10..=10, 5..=6, 7..=7], true) << 1,
//         }
//     }
// }

#[inline]
/// Compressed instruction rs2
fn crs2(inst: u16) -> Register {
    extract_bits(inst, 2..=6) as Register
}

#[inline]
/// Compressed instruction rd
fn crs1(inst: u16) -> Register {
    extract_bits(inst, 7..=11) as Register
}

#[inline]
/// Compressed instruction rs2'
fn crs2_(inst: u16) -> Register {
    extract_bits(inst, 2..=4) as Register + 8
}

#[inline]
/// Compressed instruction rs1'
fn crs1_(inst: u16) -> Register {
    extract_bits(inst, 7..=9) as Register + 8
}

#[inline]
/// Load an immediate value from an instruction.
/// Inspired from https://github.com/Wren6991/Hazard3/blob/stable/test/sim/rvpy/rvpy
fn load_imm(bits: u32, positions: &[RangeInclusive<u32>], signed: bool) -> u32 {
    let mut accum = 0;
    let mut count = 0;

    for range in positions {
        let value = extract_bits(bits, range.clone());
        let lsb = range.start();
        let msb = range.end();
        accum = (accum << (msb - lsb + 1)) | value;
        count += msb - lsb + 1;
    }

    if signed {
        accum = sign_extend(accum, count - 1);
    }

    accum
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_registers() {
        let inst = 0b00000000000100010000001010110011;
        assert_eq!(rd(inst), 5);
        assert_eq!(rs1(inst), 2);
        assert_eq!(rs2(inst), 1);
    }

    #[test]
    fn parse_registers_max() {
        let inst = 0b00000001111111111000111110110011;
        assert_eq!(rs1(inst), 31);
        assert_eq!(rs2(inst), 31);
        assert_eq!(rd(inst), 31);
    }

    #[test]
    fn parse_registers_min() {
        let inst = 0b00000000000000000000000000110011;
        assert_eq!(rs1(inst), 0);
        assert_eq!(rs2(inst), 0);
        assert_eq!(rd(inst), 0);
    }

    #[test]
    fn R_type() {
        let inst: u32 = 0b0000000_00001_00010_000_00101_0110011; // Example R-type instruction
        let rtype = RType::from(inst);
        assert_eq!(rtype.rd, 5);
        assert_eq!(rtype.rs1, 2);
        assert_eq!(rtype.rs2, 1);
    }

    #[test]
    fn I_type() {
        let inst = 0b10100010000000000000001010010011;
        let itype = IType::from(inst);
        assert_eq!(itype.rd, 5);
        assert_eq!(itype.rs1, 0);
        assert_eq!(itype.imm as i32, -1504);
    }

    #[test]
    fn I_type_max() {
        let inst = 0b11111111111111111111001010010011;
        let itype = IType::from(inst);
        assert_eq!(itype.rd, 5);
        assert_eq!(itype.rs1, 31);
        assert_eq!(itype.imm as i32, -1);
    }

    #[test]
    fn I_type_max_imm() {
        let inst = 0b01111111111111111111001010010011;
        let itype = IType::from(inst);
        assert_eq!(itype.rd, 5);
        assert_eq!(itype.rs1, 31);
        assert_eq!(itype.imm, 2047);
    }

    #[test]
    fn I_type_min_imm() {
        let inst = 0b10000000000011111111001010010011;
        let itype = IType::from(inst);
        assert_eq!(itype.rd, 5);
        assert_eq!(itype.rs1, 31);
        assert_eq!(itype.imm as i32, -2048);
    }

    #[test]
    fn B_type() {
        let inst = 0b0000000_00001_00010_000_1100_0_1100011;
        let btype = BType::from(inst);
        assert_eq!(btype.rs1, 2);
        assert_eq!(btype.rs2, 1);
        assert_eq!(btype.imm, 24);
    }

    #[test]
    fn B_type_max() {
        let inst = 0b11111110000011111000111111100011;
        let btype = BType::from(inst);
        assert_eq!(btype.rs1, 31);
        assert_eq!(btype.rs2, 0);
        assert_eq!(btype.imm as i32, -2);
    }

    #[test]
    fn B_type_max_imm() {
        let inst = 0b01111110000011111000111111100011;
        let btype = BType::from(inst);
        assert_eq!(btype.rs1, 31);
        assert_eq!(btype.rs2, 0);
        assert_eq!(btype.imm, 4094);
    }

    #[test]
    fn B_type_min_imm() {
        let inst = 0b10000001111100000000000001100011;
        let btype = BType::from(inst);
        assert_eq!(btype.rs1, 0);
        assert_eq!(btype.rs2, 31);
        assert_eq!(btype.imm as i32, -4096);
    }

    #[test]
    fn U_type() {
        let inst = 0b00000000001011110110011000110111;
        let utype = UType::from(inst);
        assert_eq!(utype.rd, 12);
        assert_eq!(utype.imm, 758 << 12);
    }

    #[test]
    fn U_type_min_imm() {
        let inst = 0b10000000000000000000011000110111;
        let utype = UType::from(inst);
        assert_eq!(utype.rd, 12);
        assert_eq!(utype.imm as i32, -0x80000 << 12);
    }

    #[test]
    fn U_type_max_imm() {
        let inst = 0b01111111111111111111011000110111;
        let utype = UType::from(inst);
        assert_eq!(utype.rd, 12);
        assert_eq!(utype.imm, 524287 << 12);
    }

    #[test]
    fn U_type_max() {
        let inst = 0b11111111111111111111011000110111;
        let utype = UType::from(inst);
        assert_eq!(utype.rd, 12);
        assert_eq!(utype.imm as i32, -1 << 12);
    }

    #[test]
    fn J_type() {
        let inst = 0b01110011100100000001011001101111;
        let jtype = JType::from(inst);
        assert_eq!(jtype.rd, 12);
        assert_eq!(jtype.imm, 7992);
    }

    #[test]
    fn J_type_min_imm() {
        let inst = 0b10000000000000000000011001101111;
        let jtype = JType::from(inst);
        assert_eq!(jtype.rd, 12);
        assert_eq!(jtype.imm as i32, -0x100000);
    }

    #[test]
    fn J_type_max_imm() {
        let inst = 0b01111111111111111111011001101111;
        let jtype = JType::from(inst);
        assert_eq!(jtype.rd, 12);
        assert_eq!(jtype.imm, 0x0ffffe);
    }

    #[test]
    fn J_type_max() {
        let inst = 0b11111111111111111111011001101111;
        let jtype = JType::from(inst);
        assert_eq!(jtype.rd, 12);
        assert_eq!(jtype.imm as i32, -2);
    }

    #[test]
    fn S_type() {
        let inst = 0b10100000000000000000110110100011;
        let stype = SType::from(inst);
        assert_eq!(stype.rs1, 0);
        assert_eq!(stype.rs2, 0);
        assert_eq!(stype.imm as i32, -1509);
    }

    #[test]
    fn S_type_max_imm() {
        let inst = 0b01111110000000000000111110100011;
        let stype = SType::from(inst);
        assert_eq!(stype.rs1, 0);
        assert_eq!(stype.rs2, 0);
        assert_eq!(stype.imm, 2047);
    }

    #[test]
    fn S_type_min_imm() {
        let inst = 0b10000000000000000000000000100011;
        let stype = SType::from(inst);
        assert_eq!(stype.rs1, 0);
        assert_eq!(stype.rs2, 0);
        assert_eq!(stype.imm as i32, -2048);
    }

    #[test]
    fn S_type_max() {
        let inst = 0b11111110000000000000111110100011;
        let stype = SType::from(inst);
        assert_eq!(stype.rs1, 0);
        assert_eq!(stype.rs2, 0);
        assert_eq!(stype.imm as i32, -1);
    }
}
