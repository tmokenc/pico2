pub mod bus;
pub mod clock;
pub mod common;
pub mod gpio;
pub mod interrupts;
pub mod memory;
pub mod peripherals;
pub mod processor;
pub mod rp2350;
pub mod simulator;

mod utils;

pub type Time = u64;

pub use rp2350::Rp2350;
