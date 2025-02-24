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

    pub fn core_specifics<'a>(&'a self) -> (CoreSpecificSio<'a>, CoreSpecificSio<'a>) {
        let core_0 = CoreSpecificSio {
            cpu_id: false,
            mailbox: &self.mailboxes.0,
            spinlock: &self.spinlock,
            doorbell: &self.doorbell,
            timer: &self.timer,
            gpio: &self.gpio,
            interpolator: &self.interpolator,
            tmds: &self.tmds,
        };

        let core_1 = CoreSpecificSio {
            cpu_id: true,
            mailbox: &self.mailboxes.1,
            spinlock: &self.spinlock,
            doorbell: &self.doorbell,
            timer: &self.timer,
            gpio: &self.gpio,
            interpolator: &self.interpolator,
            tmds: &self.tmds,
        };

        (core_0, core_1)
    }

    pub fn update(&mut self, core_0: CoreSpecificSio, core_1: CoreSpecificSio) {
        // self.mailboxes.0 = core_0.mailbox;
        // self.mailboxes.1 = core_1.mailbox;
        // self.spinlock = core_0.spinlock;
        // self.doorbell = core_0.doorbell;
        // self.timer = core_0.timer;
        // self.gpio = core_0.gpio;
        // self.interpolator = core_0.interpolator;
        // self.tmds = core_0.tmds;
    }

    // pub fn bus_interface(&self) -> BusInterface {
    //     todo!()
    // }

    pub fn read(&mut self) -> u32 {
        todo!()
    }

    pub fn write(&mut self, val: u32) {
        todo!()
    }
}
