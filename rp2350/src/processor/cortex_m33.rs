use super::CpuArchitecture;
use super::ProcessorContext;

#[derive(Default)]
pub struct CortexM33 {
    // TODO
}

impl CpuArchitecture for CortexM33 {
    fn set_core_id(&mut self, _core_id: u8) {
        todo!()
    }

    fn get_pc(&self) -> u32 {
        todo!()
    }

    fn set_pc(&mut self, _value: u32) {
        todo!()
    }

    fn tick(&mut self, _ctx: &mut ProcessorContext) {
        todo!()
    }

    fn sleep(&mut self) {
        todo!()
    }

    fn wake(&mut self) {
        todo!()
    }

    fn set_sp(&mut self, _value: u32) {
        todo!()
    }
}
