use super::{Register, Registers};
use core::ops::RangeInclusive;

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

#[derive(Default)]
pub(super) struct CmppType {
    pub(super) urlist: u8,
    pub(super) spimm: u8,
}

#[derive(Default)]
pub(super) struct CmmvType {
    pub(super) r1s: u8,
    pub(super) r2s: u8,
}

impl From<u32> for RType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7..=11),
            rs1: extract_bits(inst, 15..=19),
            rs2: extract_bits(inst, 20..=24),
        }
    }
}

impl From<u32> for IType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7..=11),
            rs1: extract_bits(inst, 15..=19),
            imm: load_immediate(inst, &[(20, 30, 0)], 31),
        }
    }
}

impl From<u32> for SType {
    fn from(inst: u32) -> Self {
        Self {
            rs1: extract_bits(inst, 15..=19),
            rs2: extract_bits(inst, 20..=24),
            imm: load_immediate(inst, &[(7, 11, 0), (25, 30, 5)], 31),
        }
    }
}

impl From<u32> for BType {
    fn from(inst: u32) -> Self {
        Self {
            rs1: extract_bits(inst, 15..=19),
            rs2: extract_bits(inst, 20..=24),
            imm: load_immediate(inst, &[(7, 7, 11), (8, 11, 1), (25, 30, 5)], 31),
        }
    }
}

impl From<u32> for UType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7..=11),
            imm: load_immediate(inst, &[(12, 30, 12)], 31),
        }
    }
}

impl From<u32> for JType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7..=11),
            imm: load_immediate(inst, &[(12, 19, 12), (20, 20, 11), (21, 30, 1)], 31),
        }
    }
}

impl From<u32> for ZbbType {
    fn from(inst: u32) -> Self {
        Self {
            rd: extract_bits(inst, 7..=11),
            rs1: extract_bits(inst, 15..=19),
        }
    }
}

impl From<u16> for CmppType {
    fn from(inst: u16) -> Self {
        Self {
            urlist: extract_bits(inst, 4..=7) as u8,
            spimm: extract_bits(inst, 2..=3) as u8,
        }
    }
}

impl From<u16> for CmmvType {
    fn from(inst: u16) -> Self {
        Self {
            r1s: extract_bits(inst, 2..=4) as u8,
            r2s: extract_bits(inst, 7..=9) as u8,
        }
    }
}

fn load_immediate(raw: u32, positions: &[(u32, u32, u32)], sign_bit_position: usize) -> u32 {
    let mut value = 0;

    for &(start, end, placement) in positions {
        let bits = extract_bits(raw, start..=end);
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

pub const fn extract_bits<T>(num: T, range: RangeInclusive<T>) -> T {
    let (from, to) = range.into_inner();
    (num >> from) & ((1 << (to - from + 1)) - 1)
}

const fn sign_extend(num: u32, from: u32, to: u32) -> u32 {
    let shift = 32 - to;
    (num << shift) as i32 as u32 >> shift
}
