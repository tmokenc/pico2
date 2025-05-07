/**
 * @file processor.rs
 * @author Nguyen Le Duy
 * @date 02/01/2025
 * @brief Definition of the processor module for the Rp2350 architecture.
 */
pub mod cortex_m33;
pub mod hazard3;

use crate::bus::Bus;
use crate::interrupts::Interrupts;
use crate::InspectorRef;
pub use cortex_m33::CortexM33;
pub use hazard3::Hazard3;
use std::cell::RefCell;
use std::rc::Rc;

pub struct ProcessorContext<'a> {
    pub bus: &'a mut Bus,
    pub inspector: InspectorRef,
    pub interrupts: Rc<RefCell<Interrupts>>,
    pub wake_opposite_core: bool,
}

pub trait CpuArchitecture {
    fn set_core_id(&mut self, core_id: u8);
    fn get_pc(&self) -> u32;
    fn set_pc(&mut self, value: u32);
    fn set_sp(&mut self, value: u32);
    fn tick(&mut self, ctx: &mut ProcessorContext);
    fn sleep(&mut self);
    fn wake(&mut self);
}

pub enum Rp2350Core {
    Arm(CortexM33),
    RiscV(Hazard3),
}

impl Rp2350Core {
    pub fn new() -> Self {
        Self::RiscV(Hazard3::new())
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

    pub fn set_pc(&mut self, value: u32) {
        match self {
            Self::Arm(core) => core.set_pc(value),
            Self::RiscV(core) => core.set_pc(value),
        }
    }

    pub fn set_sp(&mut self, value: u32) {
        match self {
            Self::Arm(core) => core.set_sp(value),
            Self::RiscV(core) => core.set_sp(value),
        }
    }

    pub fn set_register(&mut self, reg: u8, value: u32) {
        match self {
            Self::RiscV(core) => core.registers.write(reg, value),
            _ => {} // TODO
        }
    }
}
