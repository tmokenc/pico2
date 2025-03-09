pub mod cortex_m33;
pub mod hazard3;
pub mod sleep_state;
pub mod stats;

use crate::bus::Bus;
use crate::interrupts::Interrupts;
pub use cortex_m33::CortexM33;
pub use hazard3::Hazard3;
pub use sleep_state::SleepState;
pub use stats::Stats;

pub struct ProcessorContext<'a> {
    pub bus: &'a mut Bus,
    pub interrupts: &'a mut Interrupts,
}

pub trait CpuArchitecture: Default {
    fn set_core_id(&mut self, core_id: u8);
    fn get_pc(&self) -> u32;
    fn set_pc(&mut self, value: u32);
    fn tick(&mut self, ctx: &mut ProcessorContext);
    fn set_opposite_sleep_state(&mut self, opposite: SleepState);
    fn set_sleep_state(&mut self, sleep_state: SleepState);
    fn stats(&self) -> &Stats;
}

pub enum Rp2350Core {
    Arm(CortexM33),
    RiscV(Hazard3),
}
impl Default for Rp2350Core {
    fn default() -> Self {
        Self::RiscV(Hazard3::default())
    }
}

impl Rp2350Core {
    pub fn new() -> Self {
        Self::default()
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

    pub fn set_opposite_sleep_state(&mut self, opposite: SleepState) {
        match self {
            Self::Arm(core) => core.set_opposite_sleep_state(opposite),
            Self::RiscV(core) => core.set_opposite_sleep_state(opposite),
        }
    }

    pub fn set_sleep_state(&mut self, sleep_state: SleepState) {
        match self {
            Self::Arm(core) => core.set_sleep_state(sleep_state),
            Self::RiscV(core) => core.set_sleep_state(sleep_state),
        }
    }
}
