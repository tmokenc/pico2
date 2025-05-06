/**
 * @file gpio/pin.rs
 * @author Nguyen Le Duy
 * @date 14/04/2025
 * @brief Definition of the GPIO pin
 */
//
use crate::utils::{extract_bit, extract_bits};

use super::*;
type InterruptCheck = bool;

#[derive(Default, Clone, Debug)]
pub struct GpioPin {
    pub raw_input_value: bool,
    pub ctrl: u32,
    pub pad: u32,
    pub index: u8,
    pub interrupt_raw: u8,
    pub interrupt_mask: u8,
    pub interrupt_force: u8,
    pub previous_value: bool,
}

impl GpioPin {
    pub fn new(index: u8) -> Self {
        Self {
            index,
            raw_input_value: false,
            ctrl: 0x1f,
            pad: 0b1_0001_0110,
            interrupt_raw: 0,
            interrupt_mask: 0,
            interrupt_force: 0,
            previous_value: false,
        }
    }

    pub fn func_sel(&self) -> FunctionSelect {
        let index = self.ctrl & 0b1111;
        FUNCTION_SELECTS[self.index as usize]
            .get(index as usize)
            .copied()
            .unwrap_or(FunctionSelect::None)
    }

    pub fn out_override(&self) -> Override {
        Override::from(extract_bits(self.ctrl, 12..=13))
    }

    pub fn in_override(&self) -> Override {
        Override::from(extract_bits(self.ctrl, 16..=17))
    }

    pub fn oe_override(&self) -> Override {
        Override::from(extract_bits(self.ctrl, 14..=15))
    }

    pub fn irq_override(&self) -> Override {
        Override::from(extract_bits(self.ctrl, 28..=29))
    }

    // from pad
    // 1 == Fast
    pub fn slew_rate(&self) -> bool {
        extract_bit(self.pad, 0) == 1
    }

    // Has priority over output enable from peripherals
    pub fn output_disable(&self) -> bool {
        extract_bit(self.pad, 7) == 1
    }

    pub fn drive_strength(&self) -> DriveStrength {
        DriveStrength::from(extract_bits(self.pad, 4..=5))
    }

    pub fn input_enable(&self) -> bool {
        extract_bit(self.ctrl, 6) == 1
    }

    pub fn pad_isolation_control(&self) -> bool {
        extract_bit(self.pad, 8) == 1
    }

    pub fn schmitt(&self) -> bool {
        extract_bit(self.pad, 1) == 1
    }

    pub fn pull_up_enable(&self) -> bool {
        extract_bit(self.pad, 3) == 1
    }

    pub fn pull_down_enable(&self) -> bool {
        extract_bit(self.pad, 2) == 1
    }

    pub fn input_value(&self) -> bool {
        self.in_override().apply_bool(self.raw_input_value)
    }

    pub fn interrupt_status(&self) -> u8 {
        (self.interrupt_raw & self.interrupt_mask) | self.interrupt_force
    }

    pub fn interrupting(&self) -> bool {
        let irq = self.interrupt_status();
        self.irq_override().apply_bool(irq != 0)
    }

    pub fn update_interrupt(&mut self, value: u8) {
        if (value & IRQ_EDGE_LOW) != 0 && (self.interrupt_raw & IRQ_EDGE_LOW) != 0 {
            self.interrupt_raw &= !IRQ_EDGE_LOW;
        }

        if (value & IRQ_EDGE_HIGH) != 0 && (self.interrupt_raw & IRQ_EDGE_HIGH) != 0 {
            self.interrupt_raw &= !IRQ_EDGE_HIGH;
        }
    }

    pub fn set_input(&mut self, value: bool) -> InterruptCheck {
        self.raw_input_value = value;
        let last_irq = self.interrupt_status();

        if value && self.input_enable() {
            self.interrupt_raw |= IRQ_EDGE_HIGH | IRQ_LEVEL_HIGH;
            self.interrupt_raw &= !IRQ_LEVEL_LOW;
        } else {
            self.interrupt_raw |= IRQ_EDGE_LOW | IRQ_LEVEL_LOW;
            self.interrupt_raw &= !IRQ_LEVEL_HIGH;
        }

        last_irq == self.interrupt_status()
    }
}
