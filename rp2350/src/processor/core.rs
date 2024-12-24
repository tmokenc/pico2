#[path = "cortex_m33.rs"]
pub mod cortex_m33;
#[path = "hazard3.rs"]
pub mod hazard3;
#[path = "stats.rs"]
pub mod stats;

pub use cortex_m33::CortexM33;
pub use hazard3::Hazard3;
use std::ops::{Deref, DerefMut};

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
