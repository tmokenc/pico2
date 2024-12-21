pub mod clock;
pub mod constants;
pub mod memory;
pub mod peripherals;
pub mod processor;
pub mod simulator;
pub mod sio;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub type Time = u64;
