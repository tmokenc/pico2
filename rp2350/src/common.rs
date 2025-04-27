//! Common types and constants used in the library.

pub const KB: usize = 1 << 10;
pub const MB: usize = KB << 10;
pub const MHZ: u64 = 1e6 as u64;

pub const fn is_supported_uf2_family_id(family_id: u32) -> bool {
    matches!(
        family_id,
        0xe48bff57 // RP2XXX_ABSOLUTE
        | 0xe48bff58 // RP2XXX_DATA
        | 0xe48bff59 // RP2350_ARM_S
        | 0xe48bff5a // RP2350_RISC_V
        | 0xe48bff5b // RP2350_ARM_NS
    )
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Requestor {
    #[default]
    Proc0,
    Proc1,
    DmaR,
    DmaW,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ArchitectureType {
    #[default]
    Hazard3,
    CortexM33,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DataSize {
    Byte = 1,
    HalfWord = 2,
    #[default]
    Word = 4,
}

impl Requestor {
    pub fn is_dma(&self) -> bool {
        matches!(self, Requestor::DmaR | Requestor::DmaW)
    }

    pub fn is_proc(&self) -> bool {
        matches!(self, Requestor::Proc0 | Requestor::Proc1)
    }
}

pub enum LedState {
    On,
    Off,
}
