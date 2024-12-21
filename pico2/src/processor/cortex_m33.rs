use super::CpuArchitecture;
use super::Stats;

#[derive(Default)]
pub struct CortexM33 {
    // TODO
}

impl CpuArchitecture for CortexM33 {
    fn get_pc(&self) -> u32 {
        todo!()
    }

    fn set_pc(&mut self, value: u32) {
        todo!()
    }

    fn tick(&mut self) {
        todo!()
    }

    fn stats(&self) -> &Stats {
        todo!()
    }
}
