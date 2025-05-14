/**
 * @file processor.rs
 * @author Nguyen Le Duy
 * @date 02/01/2025
 * @brief Entry point for the Rp2350 simulator.
 */
use crate::bus::{self, Bus};
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
    }

    pub fn set_inspector(&mut self, inspector: Rc<dyn crate::inspector::Inspector>) {
        self.inspector.set_inspector(inspector);
        self.bus.peripherals.inspector = self.inspector.clone();
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

            let address = block.target_addr;

            let result = match address & 0xF000_0000 {
                Bus::XIP => {
                    let address = address & bus::XIP_ADDRESS_MASK;
                    self.bus.flash.write_slice(address, &block.data)
                }
                Bus::SRAM => self.bus.sram.write_slice(address - Bus::SRAM, &block.data),
                _ => {
                    log::warn!("Unsupported target address: {:#X}", block.target_addr);
                    continue;
                }
            };

            if let Err(why) = result {
                log::error!("Failed to write block to flash: {:#}", why);
            }
        }

        // Dump of the data section
        // This does not include from the uf2
        self.reset();
        let data_section = include_bytes!("../data.bin");
        self.bus.set_sram(data_section);

        self.inspector.emit(InspectionEvent::FlashedBinary);

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
        self.processor[0].set_pc(0x1000_0086);
        self.processor[1].set_pc(0x1000_0086);
        self.bus.sram.write_u32(0x0002c44, 0x20002c54).ok();
        self.bus.sram.write_u32(0x0000134, 0x20002c64).ok();
        self.bus.sram.write_u32(0x0000144, 0x20002c74).ok();
        self.bus.sram.write_u32(0x0000134 + 12, 0x20002c94).ok();
        self.bus.sram.write_u32(0x0000053c, 0x10000184).ok();
        self.bus.sram.write_u32(0x00000414, 0x10000184).ok();
        self.bus.sram.write_u32(0x00000400, 0x10000184).ok();
        //self.bus.sram.write_u32(0x20000324, 0x10000184).ok();
        // 0x20000324
        // GP 20000D44
        // dumped registers
        let regs = [
            0x00000000, 0x1000021c, 0x20081f50, 0x20002580, 0x00000000, 0x10006620, 0x0000000f,
            0x20081f50, 0x00000004, 0x20081f50, 0x000f4356, 0x00000000, 0x20004000, 0x400b0000,
            0x000f4365, 0x400b0000, 0x20003b18, 0x00000001, 0x10000036, 0x00000000, 0x00000000,
            0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
            0x00000001, 0x00000000, 0x00006aac, 0x000074d6,
        ];

        for (i, &reg) in regs.iter().enumerate().take(4) {
            self.processor[0].set_register(i as u8, reg);
            self.processor[1].set_register(i as u8, reg);
        }

        self.processor[1].sleep();
    }

    pub fn set_gpio_pin_input(&self, pin_index: u8, value: bool) {
        assert!(pin_index < 30, "Invalid GPIO pin index: {}", pin_index);
        let mut gpio = self.gpio.borrow_mut();

        if let Some(pin) = gpio.get_pin_mut(pin_index) {
            let irq_check = pin.set_input(value);
            if irq_check {
                gpio.update_interrupt();

                // update for PWM
                drop(gpio); // avoid deadlock
                let clock = self.clock.clone();
                let gpio = self.gpio.clone();
                let pwm = self.bus.peripherals.pwm.clone();
                let interrupts = self.interrupts.clone();
                let inspector = self.inspector.clone();

                crate::gpio::update_pwm_b_pin(
                    pin_index, value, pwm, clock, gpio, interrupts, inspector,
                );
            }
        }
    }
}
