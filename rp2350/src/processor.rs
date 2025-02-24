pub mod cortex_m33;
pub mod hazard3;
pub mod stats;

use crate::bus::Bus;
use crate::common::*;
use crate::interrupts::Interrupts;
use core::ops::{Deref, DerefMut, RangeInclusive};
pub use cortex_m33::CortexM33;
pub use hazard3::Hazard3;
pub use stats::Stats;
use std::rc::Rc;

pub struct ProcessorContext<'a> {
    bus: &'a mut Bus,
    interrupts: &'a mut Interrupts,
}

pub trait CpuArchitecture: Default {
    fn set_core_id(&mut self, core_id: u8);
    fn get_pc(&self) -> u32;
    fn set_pc(&mut self, value: u32);
    fn tick(&mut self, ctx: &mut ProcessorContext);
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

    fn set_core_id(&mut self, core_id: u8) {
        match self {
            Self::Arm(core) => core.set_core_id(core_id),
            Self::RiscV(core) => core.set_core_id(core_id),
        }
    }

    pub(self) fn tick(&mut self, ctx: &mut ProcessorContext) {
        match self {
            Self::Arm(core) => core.tick(ctx),
            Self::RiscV(core) => core.tick(ctx),
        }
    }
}

pub struct Rp2350 {
    bus: Bus,
    cores: [Rp2350Core; 2],
    stats: Stats,
    interrupts: Interrupts,
}

impl Rp2350 {
    pub fn new() -> Self {
        let mut cores = [Rp2350Core::new(), Rp2350Core::new()];
        cores[0].set_core_id(0);
        cores[1].set_core_id(1);

        Self {
            bus: Bus::new(),
            cores: cores,
            stats: Stats::default(),
            interrupts: Interrupts::default(),
        }
    }

    pub fn tick(&mut self) {
        self.bus.tick();

        let mut ctx = ProcessorContext {
            bus: &mut self.bus,
            interrupts: &mut self.interrupts,
        };

        self.cores[0].tick(&mut ctx);
        self.cores[1].tick(&mut ctx);
        self.interrupts.update();
    }
}
