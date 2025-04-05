//! Common types and constants used in the library.

pub const KB: usize = 1 << 10;
pub const MB: usize = KB << 10;
pub const MHZ: u64 = 1e6 as u64;

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
    Byte,
    HalfWord,
    #[default]
    Word,
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
