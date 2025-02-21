use crate::clock::Clock;
use crate::common::*;
use crate::processor::Rp2350;

pub struct Simulator {
    clock: Clock<{ 150 * MHZ }>,
    processor: Rp2350,
}

impl Simulator {
    pub fn new() -> Self {
        todo!()
        // Self {
        //     clock: Clock::default(),
        //     processor: Rp2350::new(),
        //     rom: Memory::new(),
        //     sram: Memory::new(),
        //     cache_line: Memory::new(),
        //     boot_ram: Memory::new(),
        //     sio: Sio::default(),
        // }
    }

    pub fn run(&mut self) {
        loop {
            self.clock.tick();
        }
    }

    pub fn stop(&mut self) {
        // self.clock.stop();
    }
}
