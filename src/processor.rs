pub mod cortex_m33;
pub mod hazard3;

pub use cortex_m33::CortexM33;
pub use hazard3::Hazard3;

pub trait CpuArchitecture: Default {
    fn get_pc(&self) -> u32;
    fn set_pc(&mut self, value: u32);
    fn exec(&mut self);
}

pub enum Rp2350Core {
    Arm(CortexM33),
    RiscV(Hazard3),
}

pub struct Rp2350 {
    cores: [Rp2350Core; 2],
}

impl Rp2350 {
    pub fn new() -> Self {
        todo!()
    }

    pub fn exec(&self) {
        todo!()
    }
}
