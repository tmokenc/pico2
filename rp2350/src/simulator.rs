pub mod pico2;

pub use crate::rp2350::Rp2350;
pub use pico2::Pico2;

#[derive(Default)]
pub struct Simulator {
    rp2350: Rp2350,
}

pub struct SimulatorController {}

impl Simulator {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn run(&mut self) -> SimulatorController {
        loop {
            self.rp2350.tick();
        }
    }

    pub fn stop(&mut self) {
        // self.clock.stop();
    }
}
