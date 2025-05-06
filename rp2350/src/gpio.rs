/**
 * @file gpio.rs
 * @author Nguyen Le Duy
 * @date 14/04/2025
 * @brief GPIO module for the RP2350
 */
//
pub mod drive_strength;
pub mod function_select;
pub mod r#override;
pub mod pin;
pub mod state;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interrupts::Interrupts;
use crate::utils::extract_bit;

pub use drive_strength::*;
pub use function_select::*;
pub use pin::*;
pub use r#override::*;
pub use state::*;

type PinIndex = u8;

pub enum GpioFunction {
    Input,
    Output,
    AltFunction,
}

const IRQ_LEVEL_LOW: u8 = 1 << 0;
const IRQ_LEVEL_HIGH: u8 = 1 << 1;
const IRQ_EDGE_LOW: u8 = 1 << 2;
const IRQ_EDGE_HIGH: u8 = 1 << 3;

#[derive(Debug, Clone, Copy, Default)]
pub struct GpioPinOutputOption {
    pub enable: bool,
    pub value: bool,
}

#[derive(Debug, Clone, Default)]
pub struct GpioPinOutputs {
    pub outputs: HashMap<FunctionSelect, GpioPinOutputOption>,
    pub sio_output_enable: u32,
    pub sio_output_value: u32,
}

pub struct GpioController {
    pub pins: [GpioPin; 30],
    interrupts: Rc<RefCell<Interrupts>>,
    outputs: GpioPinOutputs,
    // pub qspi: [GpioPin; 4],
}

impl Default for GpioController {
    fn default() -> Self {
        let outputs = GpioPinOutputs::default();
        let pins: [GpioPin; 30] = (0u8..30)
            .map(|i| GpioPin::new(i))
            .collect::<Vec<GpioPin>>()
            .try_into()
            .unwrap();

        GpioController {
            pins,
            outputs,
            interrupts: Default::default(),
        }
    }
}

impl GpioController {
    pub fn new(interrupts: Rc<RefCell<Interrupts>>) -> Self {
        Self {
            interrupts,
            ..Default::default()
        }
    }

    pub fn reset(&mut self) {
        let Self { interrupts, .. } = core::mem::take(self);
        self.interrupts = interrupts;
    }

    pub fn get_pin(&self, index: u8) -> Option<&GpioPin> {
        self.pins.get(index as usize)
    }

    pub fn get_pin_mut(&mut self, index: u8) -> Option<&mut GpioPin> {
        self.pins.get_mut(index as usize)
    }

    pub fn select(&mut self, funcsel: FunctionSelect) -> Option<&mut GpioPin> {
        self.pins.iter_mut().find(|v| v.func_sel() == funcsel)
    }

    fn raw_output(&self, funcsel: FunctionSelect, index: PinIndex) -> GpioPinOutputOption {
        if funcsel == FunctionSelect::SIO {
            let enable = extract_bit(self.outputs.sio_output_enable, index as _) != 0;
            let value = extract_bit(self.outputs.sio_output_value, index as _) != 0;
            return GpioPinOutputOption { enable, value };
        }

        self.outputs
            .outputs
            .get(&funcsel)
            .copied()
            .unwrap_or_default()
    }

    pub fn pin_status(&self, index: PinIndex) -> u32 {
        assert!(index < 30);
        let ref pin = self.pins[index as usize];
        let funcsel = pin.func_sel();
        let raw_output = self.raw_output(funcsel, index);
        let output_enable = pin.oe_override().apply_bool(raw_output.enable);
        let output_value = pin.out_override().apply_bool(raw_output.value);

        let irq_to_proc = if pin.interrupting() { 1 } else { 0 };
        let irq_from_pad = if pin.interrupt_status() != 0 { 1 } else { 0 };
        let in_to_peripheral = pin.input_value() as u32;
        let in_from_pad = pin.raw_input_value as u32;
        let oe_to_pad = output_enable as u32;
        let oe_from_peripheral = raw_output.enable as u32;
        let out_to_pad = output_value as u32;
        let out_from_peripheral = raw_output.value as u32;

        (irq_to_proc << 26)
            | (irq_from_pad << 24)
            | (in_to_peripheral << 19)
            | (in_from_pad << 17)
            | (oe_to_pad << 13)
            | (oe_from_peripheral << 12)
            | (out_to_pad << 9)
            | (out_from_peripheral << 8)
    }

    pub fn pin_state(&self, index: PinIndex) -> PinState {
        assert!(index < 30);
        let ref pin = self.pins[index as usize];
        let funcsel = pin.func_sel();
        let raw_output = self.raw_output(funcsel, index);

        let output_enable = pin.oe_override().apply_bool(raw_output.enable);
        let output_value = pin.out_override().apply_bool(raw_output.value);

        if output_enable {
            let value = match output_value {
                true => OutputState::High,
                false => OutputState::Low,
            };

            PinState::Output(value, funcsel)
        } else {
            match (pin.pull_up_enable(), pin.pull_down_enable()) {
                (true, true) => PinState::Input(InputState::BusKeeper),
                (true, false) => PinState::Input(InputState::PullUp),
                (false, true) => PinState::Input(InputState::PullDown),
                (false, false) => PinState::Input(InputState::Floating),
            }
        }
    }

    pub fn set_pin_input(&mut self, index: u8, value: bool) {
        if let Some(pin) = self.get_pin_mut(index) {
            let irq_check = pin.set_input(value);
            if irq_check {
                self.update_interrupt();
            }
        }
    }

    pub fn set_pin_output(&mut self, funcsel: FunctionSelect, value: bool) {
        let entry = self.outputs.outputs.entry(funcsel).or_default();
        entry.value = value;
        self.update_interrupt();
    }

    pub fn set_pin_output_enable(&mut self, funcsel: FunctionSelect, value: bool) {
        let entry = self.outputs.outputs.entry(funcsel).or_default();
        entry.enable = value;
        self.update_interrupt();
    }

    pub fn update_pin_ctrl(&mut self, index: u8, value: u32) {
        if let Some(pin) = self.get_pin_mut(index) {
            pin.ctrl = value;
        }

        self.update_interrupt();
    }

    pub fn update_pin_pads(&mut self, index: u8, value: u32) {
        if let Some(pin) = self.get_pin_mut(index) {
            pin.pad = value;
        }

        self.update_interrupt();
    }

    pub fn update_pin_irq(&mut self, index: u8, value: u8) {
        if let Some(pin) = self.get_pin_mut(index) {
            pin.update_interrupt(value);
        }

        self.update_interrupt();
    }

    pub fn update_sio(&mut self, enable: u32, value: u32) {
        self.outputs.sio_output_enable = enable;
        self.outputs.sio_output_value = value;
    }

    pub fn update_interrupt(&self) {
        let interrupt = self.pins.iter().any(GpioPin::interrupting);
        self.interrupts
            .borrow_mut()
            .set_irq(Interrupts::IQ_IRQ_BANK0, interrupt);
    }
}
