/**
 * @file /processor/hazard/instruction_format.rs
 * @author Nguyen Le Duy
 * @date 02/01/2025
 * @brief Generic instruction format parser for RISC-V
 */
use super::Register;
use crate::utils::{extract_bit, extract_bits, sign_extend};
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
// These has uniform formats, but the parsing for immediates is vary from insstruction to
// instruction. That's why we define function for basic parsing of RS1, RS2, RD
// There are 2 type of rs, full and compressed.
// The compressed rs are shifted by 8 and the function name is suffixed with _

#[inline]
/// Compressed instruction rs2
pub fn crs2(inst: u16) -> Register {
    extract_bits(inst, 2..=6) as Register
}

#[inline]
/// Compressed instruction rd
pub fn crs1(inst: u16) -> Register {
    extract_bits(inst, 7..=11) as Register
}

#[inline]
/// Compressed instruction rs2' (shifted by 8)
pub fn crs2_(inst: u16) -> Register {
    extract_bits(inst, 2..=4) as Register + 8
}

#[inline]
/// Compressed instruction rs1' (shifted by 8)
pub fn crs1_(inst: u16) -> Register {
    extract_bits(inst, 7..=9) as Register + 8
}

#[inline]
pub fn imm_ci(inst: u16) -> u32 {
    let raw: u16 = (extract_bit(inst, 12) << 5) | extract_bits(inst, 2..=6);

    sign_extend(raw as u32, 5)
}

#[inline]
pub fn imm_cj(inst: u16) -> u32 {
    let raw: u16 = (extract_bit(inst, 12) << 11)
        | (extract_bit(inst, 11) << 4)
        | (extract_bits(inst, 9..=10) << 8)
        | (extract_bit(inst, 8) << 10)
        | (extract_bit(inst, 7) << 6)
        | (extract_bit(inst, 6) << 7)
        | (extract_bits(inst, 3..=5) << 1)
        | (extract_bit(inst, 2) << 5);

    sign_extend(raw as u32, 11)
}

#[inline]
pub fn imm_cb(inst: u16) -> u32 {
    let raw: u16 = (extract_bit(inst, 12) << 8)
        | (extract_bits(inst, 10..=11) << 3)
        | (extract_bits(inst, 5..=6) << 6)
        | (extract_bits(inst, 3..=4) << 1)
        | (extract_bit(inst, 2) << 5);

    sign_extend(raw as u32, 8)
}

// For Zcmp instructions that expand into a sequence of intructions
#[inline]
fn zcmp_rlist(code: u16) -> u16 {
    extract_bits(code, 4..=7)
}

pub(super) fn zcmp_stack_adj(code: u16) -> u32 {
    let base = match zcmp_rlist(code) {
        4..=7 => 16,
        8..=11 => 32,
        12..=14 => 48,
        15 => 64,
        _ => 0,
    };

    base + 16 * (extract_bits(code, 2..=3) as u32)
}

pub(super) fn zcmp_reg_mask(code: u16) -> u32 {
    let rlist = extract_bits(code, 4..=7);

    match rlist {
        04 => 0b00000000000000000000000000000010,
        05 => 0b00000000000000000000000100000010,
        06 => 0b00000000000000000000001100000010,
        07 => 0b00000000000001000000001100000010,
        08 => 0b00000000000011000000001100000010,
        09 => 0b00000000000111000000001100000010,
        10 => 0b00000000001111000000001100000010,
        11 => 0b00000000011111000000001100000010,
        12 => 0b00000000111111000000001100000010,
        13 => 0b00000001111111000000001100000010,
        14 => 0b00000011111111000000001100000010,
        15 => 0b00001111111111000000001100000010,
        _ => 0,
    }
}

#[inline]
pub(super) fn zcmp_s_mapping(s_raw: Register) -> Register {
    s_raw + 8 + 8 * (((s_raw & 0x6) != 0) as Register)
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
    #![allow(non_snake_case)]

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

    #[test]
    fn test_imm_cj() {
        let inst = 0b1111111111111111;
        assert_eq!(imm_cj(inst), -2i32 as u32);
    }

    #[test]
    fn test_imm_ci() {
        let inst = 0b1111111111111111;
        assert_eq!(imm_ci(inst), -1i32 as u32);
    }

    #[test]
    fn test_imm_cb() {
        let inst = 0b1111111111111111;
        assert_eq!(imm_cb(inst), -2i32 as u32);
    }
}
