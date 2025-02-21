//! Common types and constants used in the library.

pub const KB: usize = 1 << 10;
pub const MB: usize = KB << 10;
pub const MHZ: u64 = 1e6 as u64;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Requestor {
    #[default]
    Proc0,
    Proc1,
    Dma,
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
