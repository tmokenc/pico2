pub mod bus;
pub mod clock;
pub mod common;
pub mod error;
pub mod gpio;
pub mod inspector;
pub mod interrupts;
pub mod memory;
pub mod peripherals;
pub mod processor;
pub mod rp2350;
pub mod simulator;

mod utils;

pub type Time = u64;

pub use error::Error as SimulatorError;
pub use inspector::Inspector;
pub use rp2350::Rp2350;
pub type Result<T> = core::result::Result<T, SimulatorError>;
pub type InspectorRef = std::rc::Rc<dyn Inspector>;
