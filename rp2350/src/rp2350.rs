use crate::bus::Bus;
use crate::clock::Clock;
use crate::common::*;
use crate::gpio::GpioController;
use crate::interrupts::Interrupts;
use crate::processor::{ProcessorContext, Rp2350Core, SleepState};

#[derive(Default)]
pub struct Rp2350 {
    pub clock: Clock<{ 150 * MHZ }>,
    pub bus: Bus,
    pub processor: [Rp2350Core; 2],
    pub gpio: GpioController,
    pub interrupts: Interrupts,
}

impl Rp2350 {
    pub fn new() -> Self {
        let mut processor = [Rp2350Core::new(), Rp2350Core::new()];
        processor[0].set_core_id(0);
        processor[1].set_core_id(1);

        let sleep_state_core0 = SleepState::new(false);
        let sleep_state_core1 = SleepState::new(true);

        processor[0].set_sleep_state(sleep_state_core0.clone());
        processor[1].set_sleep_state(sleep_state_core1.clone());

        processor[0].set_opposite_sleep_state(sleep_state_core1);
        processor[1].set_opposite_sleep_state(sleep_state_core0);

        Self {
            processor,
            clock: Clock::default(),
            bus: Bus::new(),
            interrupts: Interrupts::default(),
            gpio: GpioController::default(),
        }
    }

    pub fn tick(&mut self) {
        self.bus.tick();

        let mut ctx = ProcessorContext {
            bus: &mut self.bus,
            interrupts: &mut self.interrupts,
        };

        self.processor[0].tick(&mut ctx);
        self.processor[1].tick(&mut ctx);
    }
}
