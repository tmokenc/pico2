pub mod cortex_m33;
pub mod hazard3;
pub mod stats;

use crate::constants::*;
use crate::memory::{GenericMemory, MemoryAccess, WriteAccess};
use crate::sio::CoreSpecificSio;
use crate::Result;
pub use cortex_m33::CortexM33;
pub use hazard3::Hazard3;
pub use stats::Stats;
use std::ops::{Deref, DerefMut, RangeInclusive};

pub(self) type BootRom = GenericMemory<{ 32 * KB }, 1, 0>;
/// Section 4.4.1
/// The 26-bit XIP address space is mirrored multiple times in the RP2350 address space, decoded
/// on bits 27:26 of the system bus address:
/// • 0x10… : Cached XIP access
/// • 0x14… : Uncached XIP access
/// • 0x18… : Cache maintenance writes
/// • 0x1c… : Uncached, untranslated XIP access — bypass QMI address translation
/// Note: Aware of Cache maintenance writes  as in Section 4.4.1.1.
pub(self) type XipCache = GenericMemory<{ 16 * KB }, 1, 1>;
pub(self) type Sram = GenericMemory<{ 520 * KB }, 1, 1>;
/// Section 4.3
/// Boot RAM is hardwired to permit Secure access only (Arm) or Machine-mode access only (RISC-V). It is physically
/// impossible to execute code from boot RAM, regardless of MPU configuration, as it is on the APB peripheral bus
/// segment, which is not wired to the processor instruction fetch ports.
pub(self) type BootRam = GenericMemory<{ 1 * KB }, 3, 4>;

pub trait CpuArchitecture {
    fn get_pc(&self) -> u32;
    fn set_pc(&mut self, value: u32);
    fn tick(&mut self);
    fn stats(&self) -> &Stats;
}

pub enum Rp2350Core {
    Arm(CortexM33),
    RiscV(Hazard3),
}

impl Rp2350Core {
    pub fn new() -> Self {
        todo!()
    }
}

impl Deref for Rp2350Core {
    type Target = dyn CpuArchitecture;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Arm(core) => core,
            Self::RiscV(core) => core,
        }
    }
}

impl DerefMut for Rp2350Core {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Arm(core) => core,
            Self::RiscV(core) => core,
        }
    }
}
pub struct Rp2350 {
    cores: [Rp2350Core; 2],
    boot_rom: BootRom,
    boot_ram: BootRam,
    xip: XipCache,
    sram: Sram,

    // apb_peripherals: Memory<64 * KB, 1, 1>,
    // ahb_peripherals: Memory<64 * KB, 1, 1>,
    // sio_peripherals: Memory<64 * KB, 1, 1>,
    stats: Stats,
}

impl Rp2350 {
    pub fn new() -> Self {
        todo!()
    }

    pub fn exec(&mut self) {
        todo!()
    }

    pub(self) fn write(&mut self, addr: u32, value: u32) -> Result<WriteAccess> {
        match addr {
            // Bootroom, no AMOs
            0x0000_0000..=0x0000_7FFF => {
                // Section 4.1
                // Writing to the ROM has no effect, and no bus fault is generated on write.
                // just do nothing
                Ok(WriteAccess { access_time: 0 })
            }
            // XIP,Cached, no AMOs
            0x1000_0000..=0x13FF_FFFF => {
                todo!()
            }
            // XIP,Uncached, no AMOs
            0x1400_0000..=0x17FF_FFFF => {
                todo!()
            }
            // XIP, Cache Maintenance, write only
            0x1800_0000..=0x1bFF_FFFF => {
                todo!()
            }
            // XIP, Uncached + Untranslated, no AMOs
            0x1c00_0000..=0x1FFF_FFFF => {
                todo!()
            }
            // SRAM, any (6 accesses can be done in a single cycle)
            0x2000_0000..=0x2008_1FFF => self.sram.write_u32(addr - 0x2000_0000, value),

            // APB Peripherals, no AMOs, no fetch, non-idempotent
            0x4000_0000..=0x4FFF_FFFF => {
                todo!()
            }
            // AHB Peripherals, no AMOs, no fetch, non-idempotent
            0x5000_0000..=0x5FFF_FFFF => {
                todo!()
            }
            // SIO Peripherals, no AMOs, no fetch, non-idempotent
            0xD000_0000..=0xDFFF_FFFF => {
                todo!()
            }
            _ => {
                todo!("Invalid address, should return BUS fault")
            }
        }
    }
}
