use crate::bus::Bus;
use crate::clock::Clock;
use crate::common::MB;
use crate::gpio::GpioController;
use crate::inspector::{InspectionEvent, InspectorRef};
use crate::interrupts::Interrupts;
use crate::processor::{ProcessorContext, Rp2350Core};
use crate::Result;
use std::cell::RefCell;
use std::rc::Rc;

pub struct Rp2350 {
    pub clock: Rc<Clock>,
    pub bus: Bus,
    pub processor: [Rp2350Core; 2],
    pub dma: Rc<RefCell<crate::peripherals::Dma>>,
    pub gpio: Rc<RefCell<GpioController>>,
    pub interrupts: Rc<RefCell<Interrupts>>,
    inspector: InspectorRef,
}

impl Default for Rp2350 {
    fn default() -> Self {
        Self::new()
    }
}

impl Rp2350 {
    pub fn new() -> Self {
        let interrupts = Rc::new(RefCell::new(Interrupts::default()));
        let gpio = Rc::new(RefCell::new(GpioController::new(interrupts.clone())));
        let clock = Rc::new(Clock::new());

        let mut processor = [Rp2350Core::new(), Rp2350Core::new()];
        processor[0].set_core_id(0);
        processor[1].set_core_id(1);

        let inspector = InspectorRef::default();
        let bus = Bus::new(
            Rc::clone(&gpio),
            Rc::clone(&interrupts),
            Rc::clone(&clock),
            inspector.clone(),
        );
        let dma = Rc::clone(&bus.peripherals.dma);

        Self {
            bus,
            dma,
            inspector,
            processor,
            clock,
            interrupts,
            gpio,
        }
    }

    pub fn reset(&mut self) {
        self.bus.reset();
        self.processor[0] = Rp2350Core::new();
        self.processor[1] = Rp2350Core::new();
        self.processor[0].set_core_id(0);
        self.processor[1].set_core_id(1);
        self.gpio.borrow_mut().reset();
        self.interrupts.borrow_mut().reset();

        self.skip_bootrom();
    }

    pub fn set_inspector(&mut self, inspector: Rc<dyn crate::inspector::Inspector>) {
        self.inspector.set_inspector(inspector);
    }

    pub fn flash_bin(&mut self, bin: &[u8]) -> Result<()> {
        if bin.len() > 4 * MB {
            return Err(crate::SimulatorError::FileTooLarge);
        }

        self.bus.flash.write_slice(0, bin).ok();
        Ok(())
    }

    pub fn flash_uf2(&mut self, uf2: &[u8]) -> Result<()> {
        for block in uf2::read_uf2(uf2)? {
            let Some(family_id) = block.family_id else {
                log::warn!("No family ID found in UF2 block");
                continue;
            };

            if crate::common::is_supported_uf2_family_id(family_id) {
                log::debug!(
                    "Flashing block: {:#X} -> {:#X}",
                    block.target_addr,
                    block.data.len()
                );
            } else {
                log::warn!("Unsupported UF2 family ID: {:#X}", family_id);
            }

            let offset = block.target_addr - Bus::XIP;
            let offset = offset & 0x1FFF_FFFF;
            if let Err(why) = self.bus.flash.write_slice(offset, &block.data) {
                log::error!("Failed to write block to flash: {:#}", why);
            }
        }

        Ok(())
    }

    pub fn tick(&mut self) {
        self.clock.tick();
        self.bus.tick();

        let mut ctx = ProcessorContext {
            bus: &mut self.bus,
            inspector: self.inspector.clone(),
            interrupts: Rc::clone(&self.interrupts),
            wake_opposite_core: false,
        };

        self.inspector.emit(InspectionEvent::TickCore(0));
        self.processor[0].tick(&mut ctx);

        let wake_core_1 = ctx.wake_opposite_core;
        ctx.wake_opposite_core = false;

        self.inspector.emit(InspectionEvent::TickCore(1));
        self.processor[1].tick(&mut ctx);
        let wake_core_0 = ctx.wake_opposite_core;

        self.dma.borrow_mut().tick(&mut self.bus);

        // only wake after both cores have ticked
        if wake_core_1 {
            self.inspector.emit(InspectionEvent::WakeCore(1));
            self.processor[1].wake();
        }

        if wake_core_0 {
            self.inspector.emit(InspectionEvent::WakeCore(0));
            self.processor[0].wake();
        }
    }

    pub fn skip_bootrom(&mut self) {
        // self.processor[0].set_pc(0x1000_0086);
        // self.processor[1].set_pc(0x1000_0086);
        self.processor[0].set_pc(0x1000_008a);
        self.processor[1].set_pc(0x1000_008a);
        self.processor[0].set_sp(0x2008_0000); // SRAM4
        self.processor[1].set_sp(0x2008_1000); // SRAM5
        self.processor[1].sleep();
    }
}
