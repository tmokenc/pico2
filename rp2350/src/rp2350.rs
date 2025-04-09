use crate::bus::Bus;
use crate::clock::Clock;
use crate::common::MHZ;
use crate::gpio::GpioController;
use crate::interrupts::Interrupts;
use crate::processor::{ProcessorContext, Rp2350Core};
use std::cell::RefCell;
use std::rc::Rc;

pub struct Rp2350 {
    pub clock: Rc<RefCell<Clock>>,
    pub bus: Bus,
    pub processor: [Rp2350Core; 2],
    pub gpio: Rc<RefCell<GpioController>>,
    pub interrupts: Rc<RefCell<Interrupts>>,
}

impl Default for Rp2350 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rp2350 {
    pub fn new() -> Self {
        let gpio = Rc::new(RefCell::new(GpioController::default()));
        let interrupts = Rc::new(RefCell::new(Interrupts::default()));
        let clock = Rc::new(RefCell::new(Clock::new(150 * MHZ)));

        let mut processor = [
            Rp2350Core::new(Rc::clone(&interrupts)),
            Rp2350Core::new(Rc::clone(&interrupts)),
        ];
        processor[0].set_core_id(0);
        processor[1].set_core_id(1);

        // By default the second core is sleeping
        processor[1].sleep();

        Self {
            bus: Bus::new(Rc::clone(&gpio), Rc::clone(&interrupts), Rc::clone(&clock)),
            processor,
            clock,
            interrupts,
            gpio,
        }
    }

    pub fn load_program(&mut self, core_id: usize, program: Vec<u32>) {
        todo!()
    }

    pub fn tick(&mut self) {
        self.bus.tick();

        let mut ctx = ProcessorContext {
            bus: &mut self.bus,
            wake_opposite_core: false,
        };

        log::trace!("Ticking core 0");
        self.processor[0].tick(&mut ctx);
        let wake_core_1 = ctx.wake_opposite_core;
        ctx.wake_opposite_core = false;

        log::trace!("Ticking core 1");
        self.processor[1].tick(&mut ctx);
        let wake_core_0 = ctx.wake_opposite_core;

        // only wake after both cores have ticked
        if wake_core_1 {
            log::info!("Waking core 1");
            self.processor[1].wake();
        }

        if wake_core_0 {
            log::info!("Waking core 0");
            self.processor[0].wake();
        }
    }
}
