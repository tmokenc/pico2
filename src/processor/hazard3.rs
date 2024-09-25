use super::CpuArchitecture;

#[derive(Default)]
pub struct Hazard3 {
    pc: u32,
    registers: [u32; 32],
}

impl CpuArchitecture for Hazard3 {
    fn get_pc(&self) -> u32 {
        self.pc
    }

    fn set_pc(&mut self, value: u32) {
        self.pc = value;
    }

    fn exec(&mut self) {
        todo!()
    }
}
