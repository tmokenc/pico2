use super::CpuArchitecture;
use super::Stats;
use crate::bus::Bus;

#[derive(Default)]
pub struct CortexM33 {
    // TODO
}

impl CpuArchitecture for CortexM33 {
    fn set_core_id(&mut self, core_id: u8) {
        todo!()
    }

    fn get_pc(&self) -> u32 {
        todo!()
    }

    fn set_pc(&mut self, value: u32) {
        todo!()
    }

    fn tick(&mut self, bus: &mut Bus) {
        todo!()
    }

    fn stats(&self) -> &Stats {
        todo!()
    }
}
