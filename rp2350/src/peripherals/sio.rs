use super::*;

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
    mailboxes: (Fifo, Fifo),
    spinlock: SpinLock,
    doorbell: DoorBell,
    timer: RiscVPlatformTimer,
    gpio: Gpio,
    interpolator: [Interpolator; 4],
    tmds: [TmdsEncoder; 2],
}

impl Peripheral for Sio {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        log::warn!("Unimplemented peripheral read");
        Ok(0)
    }

    fn write_raw(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        log::warn!("Unimplemented peripheral write");
        Ok(())
    }
}
