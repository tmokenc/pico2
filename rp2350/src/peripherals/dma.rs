use crate::interrupts::Interrupt;

use super::*;

use crate::common::Requestor;

#[derive(Default, Clone, Copy)]
pub struct Channel {
    pub control: u32,
    pub source: u32,
    pub destination: u32,
    pub size: u32,
    pub status: u32,
}

pub struct Dma {
    pub interrupts: Interrupt,
    pub channels: [Channel; 8],
}

impl Default for Dma {
    fn default() -> Self {
        Self {
            interrupts: Interrupt::default(),
            channels: [Channel::default(); 8],
        }
    }
}

impl Dma {
    pub fn new(interrupts: Interrupt) -> Self {}
}
