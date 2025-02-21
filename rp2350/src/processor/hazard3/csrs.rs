/// All CSRs are 32-bit, and MXLEN is fixed at 32 bits. CSR addresses not listed in this section are unimplemented.
/// Accessing an unimplemented CSR raises an illegal instruction exception (mcause = 2). This includes all S-mode CSRs.
use super::trap::{Exception, Trap};

#[derive(Default)]
pub(super) struct Csrs {
    // TODO
    mcycles: u64,
    minstret: u64,
    mstatus: u32,
    mie: u32,
    mtvec: u32,
    mscratch: u32,
    mepc: u32,
    mcause: u32,
    pub core_id: u8,
}

impl Csrs {
    const MSTATUS: u16 = 0x300;
    const MISA: u16 = 0x301;
    const MEDELEG: u16 = 0x302;
    const MIDELEG: u16 = 0x303;
    const MIE: u16 = 0x304;
    const MTVEC: u16 = 0x305;
    const MCOUNTEREN: u16 = 0x306;
    const MENVCFG: u16 = 0x30A;
    const MSTATUSH: u16 = 0x310;
    const MENVCFGH: u16 = 0x31A;
    const MCOUNTINHIBIT: u16 = 0x320;
    const MHPMEVENT3: u16 = 0x323;
    const MHPMEVENT31: u16 = 0x33F;
    const MSCRATCH: u16 = 0x340;
    const MEPC: u16 = 0x341;
    const MCAUSE: u16 = 0x342;
    const MTVAL: u16 = 0x343;
    const MIP: u16 = 0x344;

    const PMPCFG0: u16 = 0x3A0;
    const PMPCFG1: u16 = 0x3A1;
    const PMPCFG2: u16 = 0x3A2;
    const PMPCFG3: u16 = 0x3A3;
    const PMPADDR0: u16 = 0x3B0;
    const PMPADDR1: u16 = 0x3B1;
    const PMPADDR2: u16 = 0x3B2;
    const PMPADDR3: u16 = 0x3B3;
    const PMPADDR4: u16 = 0x3B4;
    const PMPADDR5: u16 = 0x3B5;
    const PMPADDR6: u16 = 0x3B6;
    const PMPADDR7: u16 = 0x3B7;
    const PMPADDR8: u16 = 0x3B8;
    const PMPADDR9: u16 = 0x3B9;
    const PMPADDR10: u16 = 0x3BA;
    const PMPADDR11: u16 = 0x3BB;
    const PMPADDR15: u16 = 0x3BF;

    const TSELECT: u16 = 0x7A0;
    const TDATA1: u16 = 0x7A1;
    const TDATA2: u16 = 0x7A2;
    const DCSR: u16 = 0x7B0;
    const DPC: u16 = 0x7B1;
    const MCYCLE: u16 = 0xB00;
    const MINSTRET: u16 = 0xB02;
    const MHPMCOUNTER3: u16 = 0xB03;
    const MHPMCOUNTER31: u16 = 0xB1F;
    const MCYCLEH: u16 = 0xB80;
    const MINSTRETH: u16 = 0xB82;
    const MHPMCOUNTER3H: u16 = 0xB83;
    const MHPMCOUNTER31H: u16 = 0xB9F;
    const PMPCFGM0: u16 = 0xBD0;
    const MEIEA: u16 = 0xBE0;
    const MEIPA: u16 = 0xBE1;
    const MEIFA: u16 = 0xBE2;
    const MEIPRA: u16 = 0xBE3;
    const MEINEXT: u16 = 0xBE4;
    const MEICONTEXT: u16 = 0xBE5;
    const MSLEEP: u16 = 0xBF0;
    const DMDATA0: u16 = 0xBFF;
    const CYCLE: u16 = 0xC00;
    const INSTRET: u16 = 0xC02;
    const CYCLEH: u16 = 0xC80;
    const INSTRETH: u16 = 0xC82;
    const MVENDORID: u16 = 0xF11;
    const MARCHID: u16 = 0xF12;
    const MIMPID: u16 = 0xF13;
    const MHARTID: u16 = 0xF14;
    const MCONFIGPTR: u16 = 0xF15;

    const VENDORID: u32 = 0x0000009 | 0x13;
    const ARCHID: u32 = 0x0000001b;
    const IMPID: u32 = 0x86fc4e3f;

    pub fn tick(&mut self) {
        todo!()
    }

    pub fn set_trap(&mut self, trap: Trap) {
        let mut cause = 0;

        match trap {
            Trap::Exception(ex) => match ex {
                Exception::InstructionAlignment => cause = 0,
                Exception::InstructionFetchFault => cause = 1,
                Exception::IllegalInstruction => cause = 2,
                Exception::BreakPoint => cause = 3,
                Exception::LoadAlignment => cause = 4,
                Exception::LoadFault => cause = 5,
                Exception::StoreAlignment => cause = 6,
                Exception::StoreFault => cause = 7,
                Exception::EcallUMode => cause = 8,
                Exception::EcallMMode => cause = 9,
            },
            Trap::Interrupt(int) => {
                cause = int as u32;
            }
        }
    }
    pub fn read(&self, offset: u16) -> Result<u32, Exception> {
        let result = match offset {
            Self::MVENDORID => Self::VENDORID,
            Self::MARCHID => Self::ARCHID,
            Self::MIMPID => Self::IMPID,
            Self::MHARTID => self.core_id as u32,
            Self::MHPMEVENT3..=Self::MHPMEVENT31
            | Self::MHPMCOUNTER3..=Self::MHPMCOUNTER31
            | Self::MHPMCOUNTER3H..=Self::MHPMCOUNTER31H
            | Self::MTVAL
            | Self::PMPADDR11..=Self::PMPADDR15
            | Self::MCONFIGPTR => {
                // Hardwired to zero
                0
            }

            _ => {
                // Unimplemented CSR
                todo!()
            }
        };

        Ok(result)
    }

    pub(super) fn write(&mut self, csr: u16, value: u32) -> Result<(), Exception> {
        todo!()
    }
}
