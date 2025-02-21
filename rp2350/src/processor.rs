pub mod cortex_m33;
pub mod hazard3;
pub mod stats;

use crate::bus::Bus;
use crate::common::*;
use core::ops::{Deref, DerefMut, RangeInclusive};
pub use cortex_m33::CortexM33;
pub use hazard3::Hazard3;
pub use stats::Stats;

pub trait CpuArchitecture: Default {
    fn set_core_id(&mut self, core_id: u8);
    fn get_pc(&self) -> u32;
    fn set_pc(&mut self, value: u32);
    fn tick(&mut self, bus: &mut Bus);
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

    pub(self) fn tick(&mut self, bus: &mut Bus) {
        match self {
            Self::Arm(core) => core.tick(bus),
            Self::RiscV(core) => core.tick(bus),
        }
    }
}

pub struct Rp2350 {
    cores: [Rp2350Core; 2],
    bus: Bus,
    stats: Stats,
}

impl Rp2350 {
    pub fn new() -> Self {
        todo!()
    }

    pub fn tick(&mut self) {
        self.bus.tick();
        self.cores[0].tick(&mut self.bus);
        self.cores[1].tick(&mut self.bus);
    }
}
