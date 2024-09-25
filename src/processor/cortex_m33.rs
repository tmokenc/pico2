use super::CpuArchitecture;

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

    fn exec(&mut self) {
        todo!()
    }
}
