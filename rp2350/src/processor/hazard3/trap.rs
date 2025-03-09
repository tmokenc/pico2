use crate::interrupts::*;

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum Exception {
    InstructionAlignment = 0x0,
    InstructionFetchFault = 0x1,
    IllegalInstruction = 0x2,
    BreakPoint = 0x3,
    LoadAlignment = 0x4,
    LoadFault = 0x5,
    StoreAlignment = 0x6,
    StoreFault = 0x7,
    EcallUMode = 0x8,
    EcallMMode = 0x9,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(super) enum Trap {
    Exception(Exception),
    Interrupt(Interrupt),
}

impl From<Exception> for Trap {
    fn from(ex: Exception) -> Self {
        Self::Exception(ex)
    }
}

impl From<Interrupt> for Trap {
    fn from(int: Interrupt) -> Self {
        Self::Interrupt(int)
    }
}

impl Trap {
    pub fn to_xcause(self) -> u32 {
        match self {
            Trap::Exception(ex) => ex as u32,
            Trap::Interrupt(int) => (1 << 31) | (int as u32),
        }
    }
}
