use super::CpuArchitecture;
use super::ProcessorContext;
use super::SleepState;
use super::Stats;

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

    fn tick(&mut self, ctx: &mut ProcessorContext) {
        todo!()
    }

    fn set_opposite_sleep_state(&mut self, opposite: SleepState) {
        todo!()
    }

    fn set_sleep_state(&mut self, sleep_state: SleepState) {
        todo!()
    }

    fn stats(&self) -> &Stats {
        todo!()
    }
}
