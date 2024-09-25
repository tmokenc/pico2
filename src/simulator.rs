use crate::clock::Clock;
use crate::constants::*;
use crate::memory::Memory;
use crate::processor::Rp2350;
use crate::sio::Sio;

pub struct Simulator {
    clock: Clock<{ 150 * MHZ }>,
    processor: Rp2350,
    sio: Sio,
    rom: Memory<{ 32 * KB }, 1, 1>,
    sram: Memory<{ 512 * KB }, 1, 1>,
    cache_line: Memory<{ 16 * KB }, 1, 1>,
    boot_ram: Memory<{ 1 * KB }, 3, 4>,
}

impl Simulator {
    pub fn new() -> Self {
        Self {
            clock: Clock::default(),
            processor: Rp2350::new(),
            rom: Memory::new(),
            sram: Memory::new(),
            cache_line: Memory::new(),
            boot_ram: Memory::new(),
            sio: Sio::default(),
        }
    }

    pub fn run(&mut self) {
        loop {
            self.clock.tick();
        }
    }
}
