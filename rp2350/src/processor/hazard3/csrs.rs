use crate::utils::extract_bits;

/// All CSRs are 32-bit, and MXLEN is fixed at 32 bits. CSR addresses not listed in this section are unimplemented.
/// Accessing an unimplemented CSR raises an illegal instruction exception (mcause = 2). This includes all S-mode CSRs.
use super::trap::{Exception, Trap};

pub const MSTATUS_UIE: u32 = 0x00000001;
pub const MSTATUS_SIE: u32 = 0x00000002;
pub const MSTATUS_HIE: u32 = 0x00000004;
pub const MSTATUS_MIE: u32 = 0x00000008;
pub const MSTATUS_UPIE: u32 = 0x00000010;
pub const MSTATUS_SPIE: u32 = 0x00000020;
pub const MSTATUS_HPIE: u32 = 0x00000040;
pub const MSTATUS_MPIE: u32 = 0x00000080;
pub const MSTATUS_SPP: u32 = 0x00000100;
pub const MSTATUS_HPP: u32 = 0x00000600;
pub const MSTATUS_MPP: u32 = 0x00001800;
pub const MSTATUS_FS: u32 = 0x00006000;
pub const MSTATUS_XS: u32 = 0x00018000;
pub const MSTATUS_MPRV: u32 = 0x00020000;
pub const MSTATUS_SUM: u32 = 0x00040000;
pub const MSTATUS_MXR: u32 = 0x00080000;
pub const MSTATUS_TVM: u32 = 0x00100000;
pub const MSTATUS_TW: u32 = 0x00200000;
pub const MSTATUS_TSR: u32 = 0x00400000;
pub const MSTATUS32_SD: u32 = 0x80000000;
// const MSTATUS_UXL: u32 = 0x0000000300000000;
// const MSTATUS_SXL: u32 = 0x0000000C00000000;
// const MSTATUS64_SD: u32 = 0x8000000000000000;

pub const MIP_MEIP: u16 = 1 << 11; // TODO correctly handle this
pub const MIE_MEIE: u32 = 1 << 11;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeMode {
    Machine = 3,
    #[default]
    User = 0,
}

impl From<u32> for PrivilegeMode {
    fn from(value: u32) -> Self {
        match value {
            0 | 1 => PrivilegeMode::User,
            2 | 3 => PrivilegeMode::Machine,
            _ => unreachable!(),
        }
    }
}

pub(super) struct Csrs {
    mcycles: u64,
    medeleg: u32,
    mideleg: u32,
    minstret: u64,
    pub mstatus: u32,
    pub mie: u32,
    mtvec: u32,
    mcounteren: u8,
    mcountinhibit: u8,
    mscratch: u32,
    mepc: u32,
    mcause: u32,
    pub mip: u16,
    pmpcfg: [u32; 4],
    pmpaddr: [u32; 8],
    tselect: u8,
    tdata: [u32; 2],
    dcsr: u32,
    dpc: u32,
    pmpcfgm0: u32,
    meiea: u32,
    meipa: u32,
    meifa: u32,
    meipra: u32,
    meinext: u32,
    meicontext: u32,
    msleep: u32,
    dmdata0: u32,

    pending_write: Option<(u16, u32)>, // write happend only at the end of a step in Hazard3
    pub(super) core_id: u8,
    pub(super) privilege_mode: PrivilegeMode,
}

impl Default for Csrs {
    fn default() -> Self {
        Self {
            mcycles: 0,
            medeleg: 0,
            mideleg: 0,
            minstret: 0,
            mstatus: 0,
            mie: 0,
            mtvec: 0x00001fff << 2,
            mcounteren: 0,
            mcountinhibit: 0,
            mepc: 0,
            mscratch: 0,
            mcause: 0,
            mip: 0,
            pmpcfg: [0; 4],
            pmpaddr: [0; 8],
            tselect: 0,
            tdata: [0; 2],
            dcsr: 0,
            dpc: 0,
            pmpcfgm0: 0,
            meiea: 0,
            meipa: 0,
            meifa: 0,
            meipra: 0,
            meinext: 0,
            meicontext: 0,
            dmdata0: 0,
            core_id: 0,
            msleep: 0,
            privilege_mode: PrivilegeMode::Machine,
            pending_write: None,
        }
    }
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

    const VENDORID: u32 = (0x0000009 << 7) | 0x13;
    const ARCHID: u32 = 0x0000001b;
    const IMPID: u32 = 0x86fc4e3f;

    pub fn is_in_debug_mode(&self) -> bool {
        (self.dcsr & 0b1) != 0
    }

    fn is_u_mode_cycle_enabled(&self) -> bool {
        (self.mcounteren & 0b1) != 0
    }

    fn is_u_mode_instret_enabled(&self) -> bool {
        (self.mcounteren & 0b100) != 0
    }

    pub fn privilege_mode(&self) -> PrivilegeMode {
        self.privilege_mode
    }

    // Trap handle as described in the RP2350 in section 3.8.4
    pub fn trap_handle(&mut self, trap: impl Into<Trap>, pc: u32) -> u32 {
        // 1. Save the address of the interrupted or excepting instruction to MEPC
        self.mepc = pc;
        // 2. Set the MSB of MCAUSE to indicate the cause is an interrupt, or clear it to indicate an exception
        let xcause = trap.into().to_xcause();
        // 3. Write the detailed trap cause to the LSBs of the MCAUSE register
        self.mcause = xcause;
        // 4. Save the current privilege level to MSTATUS.MPP
        self.mstatus = (self.mstatus & !MSTATUS_MPP) | (self.privilege_mode() as u32) << 11;
        // 5. Set the privilege to M-mode (note Hazard3 does not implement S-mode)
        self.privilege_mode = PrivilegeMode::Machine;

        // 6. Save the current value of MSTATUS.MIE to MSTATUS.MPIE
        if (self.mstatus & MSTATUS_MIE) != 0 {
            self.mstatus |= MSTATUS_MPIE;
        } else {
            self.mstatus &= !MSTATUS_MPIE;
        }

        // 7. Disable interrupts by clearing MSTATUS.MIE
        self.mstatus &= !MSTATUS_MIE;

        // 8. Jump to the correct offset from MTVEC depending on the trap cause
        if (self.mtvec & 1) != 0 && (xcause & (1 << 31)) != 0 {
            (self.mtvec & !1) + 4 * (xcause & !(1 << 31))
        } else {
            self.mtvec & !1
        }
    }

    pub fn trap_mret(&mut self) -> u32 {
        // 1. Restore core privilege level to the value of MSTATUS.MP
        self.privilege_mode = PrivilegeMode::from((self.mstatus >> 11) & 0b11);

        // 2. Write 0 (U-mode) to MSTATUS.MPP
        self.mstatus &= !MSTATUS_MPP; // clear MPP

        if self.privilege_mode == PrivilegeMode::Machine {
            self.mstatus &= !MSTATUS_MPRV; // clear MPRV
        }

        // 3. Restore MSTATUS.MIE from MSTATUS.MPIE
        if (self.mstatus & MSTATUS_MPIE) != 0 {
            self.mstatus |= MSTATUS_MIE;
        } else {
            self.mstatus &= !MSTATUS_MIE;
        }

        // 4. Write 1 to MSTATUS.MPIE
        self.mstatus |= MSTATUS_MPIE;

        // 5. Jump to the address in MEPC.
        self.mepc
    }

    pub fn tick(&mut self) {
        if self.mcountinhibit & 1 == 0 {
            self.mcycles = self.mcycles.wrapping_add(1);
        }

        if self.mcountinhibit & 0b100 == 0 {
            self.minstret = self.minstret.wrapping_add(1);
        }

        if let Some((addr, data)) = self.pending_write.take() {
            self._write(addr, data);
        }
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
        log::info!("read csr: {:#x}", offset);

        let result = match offset {
            Self::MVENDORID => Self::VENDORID,
            Self::MARCHID => Self::ARCHID,
            Self::MIMPID => Self::IMPID,
            Self::MHARTID => self.core_id as u32,
            Self::MSTATUS => self.mstatus,
            Self::MISA => 0b0100_0000_1001_0000_0001_0001_0000_0101, // rv32ima_zicsr_zifencei_zba_zbb_zbs_zbkb_zca_zcb_zcmp
            Self::MEDELEG => self.medeleg,
            Self::MIDELEG => 0,
            Self::MIE => self.mie,
            Self::MTVEC => self.mtvec,
            Self::MCOUNTEREN => self.mcounteren as u32,
            Self::MCOUNTINHIBIT => self.mcountinhibit as u32,
            Self::MSCRATCH => self.mscratch,
            Self::MEPC => self.mepc,
            Self::MCAUSE => self.mcause,
            Self::MIP => self.mip as u32,
            Self::PMPCFG0 => self.pmpcfg[0],
            Self::PMPCFG1 => self.pmpcfg[1],
            Self::PMPCFG2 => self.pmpcfg[2],
            Self::PMPCFG3 => self.pmpcfg[3],
            Self::PMPADDR0..=Self::PMPADDR7 => {
                let idx = (offset - Self::PMPADDR0) as usize;
                self.pmpaddr[idx]
            }
            Self::PMPADDR8 => 0x01ffffff,
            Self::PMPADDR9 => 0x13ffffff,
            Self::PMPADDR10 => 0x35ffffff,
            Self::TSELECT => self.tselect as u32,
            Self::TDATA1 => self.tdata[0],
            Self::TDATA2 => self.tdata[1],
            Self::DCSR => self.dcsr,
            Self::DPC => {
                if !self.is_in_debug_mode() {
                    return Err(Exception::IllegalInstruction);
                }
                self.dpc
            }
            Self::MCYCLE => self.mcycles as u32,
            Self::MINSTRET => self.minstret as u32,
            Self::MCYCLEH => (self.mcycles >> 32) as u32,
            Self::MINSTRETH => (self.minstret >> 32) as u32,
            Self::PMPCFGM0 => self.pmpcfgm0,
            Self::MEIEA => self.meiea & !0b11111,
            Self::MEIPA => self.meipa & !0b11111,
            Self::MEIFA => self.meifa & !0b11111,
            Self::MEIPRA => self.meipra & !0b11111,
            Self::MEINEXT => self.meinext,
            Self::MEICONTEXT => self.meicontext,
            Self::MSLEEP => self.msleep,
            Self::DMDATA0 => {
                if !self.is_in_debug_mode() {
                    return Err(Exception::IllegalInstruction);
                }

                self.dmdata0
            }

            Self::CYCLE => {
                if self.privilege_mode == PrivilegeMode::Machine || self.is_u_mode_cycle_enabled() {
                    self.mcycles as u32
                } else {
                    0 // don't know if this is correct or should raise an exception
                }
            }

            Self::INSTRET => {
                if self.privilege_mode == PrivilegeMode::Machine || self.is_u_mode_instret_enabled()
                {
                    self.minstret as u32
                } else {
                    0 // don't know if this is correct or should raise an exception
                }
            }

            Self::CYCLEH => {
                if self.privilege_mode == PrivilegeMode::Machine || self.is_u_mode_cycle_enabled() {
                    (self.mcycles >> 32) as u32
                } else {
                    0 // don't know if this is correct or should raise an exception
                }
            }

            Self::INSTRETH => {
                if self.privilege_mode == PrivilegeMode::Machine || self.is_u_mode_instret_enabled()
                {
                    (self.minstret >> 32) as u32
                } else {
                    0 // don't know if this is correct or should raise an exception
                }
            }

            Self::MENVCFG
            | Self::MENVCFGH
            | Self::MSTATUSH
            | Self::MHPMEVENT3..=Self::MHPMEVENT31
            | Self::MHPMCOUNTER3..=Self::MHPMCOUNTER31
            | Self::MHPMCOUNTER3H..=Self::MHPMCOUNTER31H
            | Self::MTVAL
            | Self::PMPADDR11..=Self::PMPADDR15
            | Self::MCONFIGPTR => 0, // hardwired to 0

            // Unimplemented CSR
            _ => return Err(Exception::IllegalInstruction),
        };

        Ok(result)
    }

    fn has_permission(&self, csr: u16) -> bool {
        (csr < 1 << 12) && (extract_bits(csr, 8..=9) <= (self.privilege_mode() as u16))
    }

    pub(super) fn write(&mut self, csr: u16, value: u32) -> Result<(), Exception> {
        if !self.has_permission(csr) {
            return Err(Exception::IllegalInstruction);
        }

        self.pending_write = Some((csr, value));

        // Validate CSR address
        let is_valid = matches!(csr,
            Self::MSTATUS
            | Self::MEDELEG
            | Self::MIDELEG
            | Self::MIE
            | Self::MTVEC
            | Self::MCOUNTEREN
            | Self::MCOUNTINHIBIT
            | Self::MSCRATCH
            | Self::MEPC
            | Self::MCAUSE
            | Self::MIP
            | Self::PMPCFG0
            | Self::PMPCFG1
            | Self::PMPADDR0..=Self::PMPADDR7
            | Self::TSELECT
            | Self::TDATA1
            | Self::TDATA2
            | Self::DCSR
            | Self::DPC
            | Self::MCYCLE
            | Self::MINSTRET
            | Self::MCYCLEH
            | Self::MINSTRETH
            | Self::PMPCFGM0
            | Self::MEIEA
            | Self::MEIPA
            | Self::MEIFA
            | Self::MEIPRA
            | Self::MEINEXT
            | Self::MEICONTEXT
            | Self::MSLEEP
            | Self::DMDATA0
            | Self::MISA
            | Self::MENVCFG
            | Self::MENVCFGH
            | Self::MSTATUSH
            | Self::MHPMEVENT3..=Self::MHPMEVENT31
            | Self::MTVAL
            | Self::PMPCFG2 | Self::PMPCFG3
            | Self::PMPADDR8..=Self::PMPADDR15
            | Self::MHPMCOUNTER3..=Self::MHPMCOUNTER31
            | Self::MHPMCOUNTER3H..=Self::MHPMCOUNTER31H
            | Self::CYCLE
            | Self::INSTRET
            | Self::CYCLEH
            | Self::INSTRETH
            | Self::MVENDORID
            | Self::MARCHID
            | Self::MIMPID
            | Self::MHARTID
            | Self::MCONFIGPTR
        );

        if !is_valid {
            return Err(Exception::IllegalInstruction);
        }

        Ok(())
    }

    // The actual write happens here
    fn _write(&mut self, csr: u16, value: u32) {
        match csr {
            Self::MSTATUS => {
                let privilege = PrivilegeMode::from((value >> 11) & 0b11);
                let mstatus = value & !(0b11 << 11); // clear MPP
                self.mstatus = mstatus | (privilege as u32); // round the privilege mode
            }
            Self::MEDELEG => self.medeleg = value,
            Self::MIDELEG => self.mideleg = value,
            Self::MIE => self.mie = value,
            Self::MTVEC => self.mtvec = value,
            Self::MCOUNTEREN => self.mcounteren = value as u8,
            Self::MCOUNTINHIBIT => self.mcountinhibit = value as u8,
            Self::MSCRATCH => self.mscratch = value,
            Self::MEPC => self.mepc = value,
            Self::MCAUSE => self.mcause = value,
            // 11th bit of MIP is read-only
            Self::MIP => self.mip = (value as u16 & 0xFF00) | (self.mip & 0b0000_1000_0000_0000),
            Self::PMPCFG0 => self.pmpcfg[0] = value,
            Self::PMPCFG1 => self.pmpcfg[1] = value,
            Self::PMPADDR0..=Self::PMPADDR7 => {
                let idx = (csr - Self::PMPADDR0) as usize;
                self.pmpaddr[idx] = value & 0x3fffffff;
            }
            Self::TSELECT => self.tselect = value as u8,
            Self::TDATA1 => {
                // 27th bit is only writable from Debug Mode
                let mask = if self.is_in_debug_mode() {
                    0b0000_1000_0000_0000_1111_0000_0111_1100
                } else {
                    0b0000_0000_0000_0000_1111_0000_0111_1100
                };
                let value = value & mask;

                // 31:28 is hardwired to 2
                self.tdata[0] = value | 2 << 28;
            }
            Self::TDATA2 => self.tdata[1] = value,
            Self::DCSR => {
                let value = value & 0b0000_1111_1111_1111_1111_0000_0000_0111;
                // 31:28 is hardwired to 4
                self.dcsr = value | 4 << 28;
            }
            Self::DPC => self.dpc = value,
            Self::MCYCLE => {
                self.mcycles &= 0xFFFF_FFFF_0000_0000; // clear lower 32 bits
                self.mcycles |= value as u64; // set lower 32 bits
            }
            Self::MINSTRET => {
                self.minstret &= 0xFFFF_FFFF_0000_0000; // clear lower 32 bits
                self.minstret |= value as u64; // set lower 32 bits
            }
            Self::MCYCLEH => {
                self.mcycles &= 0x0000_0000_FFFF_FFFF; // clear upper 32 bits
                self.mcycles |= (value as u64) << 32; // set upper 32 bits
            }
            Self::MINSTRETH => {
                self.minstret &= 0x0000_0000_FFFF_FFFF; // clear upper 32 bits
                self.minstret |= (value as u64) << 32; // set upper 32 bits
            }
            Self::PMPCFGM0 => self.pmpcfgm0 = value,

            // ------ Interrupt handler CSRs ----------
            Self::MEIEA => {
                // TODO
                self.meiea = value;
            }
            Self::MEIPA => {
                // TODO
                self.meipa = value;
            }
            Self::MEIFA => {
                // TODO
                self.meifa = value;
            }
            Self::MEIPRA => {
                // TODO
                self.meipra = value;
            }
            Self::MEINEXT => {
                if value & 1 != 0 {
                    // TODO
                    // update meicontext according to the IRQ number and preemption priority of the
                    // interrupt indicated in noirq/irp
                }
                self.meinext = value & !0b1; // last bit is self clearing bit
            }

            Self::MEICONTEXT => {
                if value & 0b10 != 0 {
                    // TODO
                    // Write-1 self-clearing field. Writing 1 will clear mie.mtie and mie.msie,
                    // and present their prior values in the mtiesave and msiesave of this register. 
                    // This makes it safe to re-enable IRQs (via mstatus.mie) without the possibility 
                    // of being preempted by the standard timer and soft interrupt handlers, 
                    // which may not be aware of Hazard3’s interrupt hardware.
                }

                let updated_value = value & 0b10;// second bit is self clearing bit

                self.meicontext &= 0b1100; // 2nd and 3rd bits are read only
                self.meicontext |= updated_value;
            }

            // -- End of Interrupt handler CSRs -- 

            Self::MSLEEP => self.msleep = value,
            Self::DMDATA0 => self.dmdata0 = value,

            Self::MISA
            | Self::MENVCFG
            | Self::MENVCFGH
            | Self::MSTATUSH
            | Self::MHPMEVENT3..=Self::MHPMEVENT31
            | Self::MTVAL
            | Self::PMPCFG2 | Self::PMPCFG3 // read only according to the spec
            | Self::PMPADDR8..=Self::PMPADDR15
            | Self::MHPMCOUNTER3..=Self::MHPMCOUNTER31
            | Self::MHPMCOUNTER3H..=Self::MHPMCOUNTER31H
            | Self::CYCLE
            | Self::INSTRET
            | Self::CYCLEH
            | Self::INSTRETH
            | Self::MVENDORID
            | Self::MARCHID
            | Self::MIMPID
            | Self::MHARTID
            | Self::MCONFIGPTR => { /* Read-only */ }
            // Unimplemented CSR
            _ => unreachable!(),
        }
    }
}
