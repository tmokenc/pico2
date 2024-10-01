pub mod doorbell;
pub mod fifo;
pub mod gpio;
pub mod interpolator;
pub mod spinlock;
pub mod timer;
pub mod tmds;

use doorbell::DoorBell;
use fifo::Fifo;
use gpio::Gpio;
use interpolator::Interpolator;
use spinlock::SpinLock;
use timer::RiscVPlatformTimer;
use tmds::TmdsEncoder;

pub struct BusInterface {
    // TODO
}

#[derive(Default)]
pub struct Sio {
    cpu_ids: (bool, bool),
    mailboxes: (Fifo, Fifo),
    spinlock: SpinLock,
    doorbell: DoorBell,
    timer: RiscVPlatformTimer,
    gpio: Gpio,
    interpolator: [Interpolator; 4],
    tmds: [TmdsEncoder; 2],
}

impl Sio {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn bus_interface(&self) -> BusInterface {
        todo!()
    }

    pub fn read(&mut self) -> u32 {
        todo!()
    }

    pub fn write(&mut self, val: u32) {
        todo!()
    }
}
