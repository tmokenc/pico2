pub mod cortex_m33;
pub mod hazard3;
pub mod stats;

use crate::bus::Bus;
use crate::interrupts::{Interrupt, Interrupts};
pub use cortex_m33::CortexM33;
pub use hazard3::Hazard3;
pub use stats::Stats;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ProcessorContext<'a> {
    pub bus: &'a mut Bus,
    pub wake_opposite_core: bool,
}

pub trait CpuArchitecture {
    fn set_core_id(&mut self, core_id: u8);
    fn get_pc(&self) -> u32;
    fn set_pc(&mut self, value: u32);
    // fn set_irq(&mut self, irq: Interrupt);
    fn tick(&mut self, ctx: &mut ProcessorContext);
    fn sleep(&mut self);
    fn wake(&mut self);
    fn stats(&self) -> &Stats;
}

pub enum Rp2350Core {
    Arm(CortexM33),
    RiscV(Hazard3),
}

impl Rp2350Core {
    pub fn new(interrupts: Rc<RefCell<Interrupts>>) -> Self {
        Self::RiscV(Hazard3::new(interrupts))
    }

    pub fn set_core_id(&mut self, core_id: u8) {
        match self {
            Self::Arm(core) => core.set_core_id(core_id),
            Self::RiscV(core) => core.set_core_id(core_id),
        }
    }

    pub fn tick(&mut self, ctx: &mut ProcessorContext) {
        match self {
            Self::Arm(core) => core.tick(ctx),
            Self::RiscV(core) => core.tick(ctx),
        }
    }

    pub fn sleep(&mut self) {
        match self {
            Self::Arm(core) => core.sleep(),
            Self::RiscV(core) => core.sleep(),
        }
    }

    pub fn wake(&mut self) {
        match self {
            Self::Arm(core) => core.wake(),
            Self::RiscV(core) => core.wake(),
        }
    }
}
