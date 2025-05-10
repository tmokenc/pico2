/**
 * @file peripherals/pwm/channel.rs
 * @author Nguyen Le Duy
 * @date 07/05/2025
 * @brief Channel definition for the PWM peripheral
 */
use std::time::Duration;

use crate::clock::Ticks;
use crate::common::MHZ;
use crate::gpio::FunctionSelect;
use crate::utils::{extract_bit, extract_bits};

#[derive(Clone, Copy)]
pub enum DivMode {
    Div,   // by the fraction divisor
    Level, // b pin
    Rise,  // rising edge on b pin
    Fall,  // falling edge on b pin
}

impl DivMode {
    pub fn is_channel_b_input(&self) -> bool {
        matches!(self, DivMode::Level | DivMode::Rise | DivMode::Fall)
    }
}

#[derive(Clone, Copy)]
pub struct PwmChannel {
    pub csr: u8,
    pub div: u16,
    pub ctr: u16,
    pub cc: u32,
    pub top: u16,
    pub wrapped: bool,
    counting_up: bool,
}

impl Default for PwmChannel {
    fn default() -> Self {
        Self {
            csr: 0,
            div: 1 << 4,
            ctr: 0,
            cc: 0,
            top: 0xffff,
            wrapped: false,
            counting_up: true,
        }
    }
}

impl PwmChannel {
    pub fn advance(&mut self) {
        if self.top == 0 {
            return;
        }

        if self.ph_correct() && self.ctr == self.top {
            self.counting_up = false;
        }

        if (self.ph_correct() && !self.counting_up && self.ctr == 0) || !self.ph_correct() {
            self.counting_up = true;
        }

        if self.counting_up {
            self.ctr += 1;
        } else {
            self.ctr -= 1;
        }

        if self.ctr > self.top {
            self.ctr = 0;
        }

        if self.ctr == 0 {
            self.wrapped = true;
        }
    }

    /// calculate the next update time
    pub fn next_update(&self) -> Ticks {
        let div = (self.div >> 4) as f64;
        let frac = (self.div & 0x0f) as f64;

        if div == 0.0 {
            return Ticks::from(256);
        }

        let divisor = div + (frac / 16.0);

        let duration = Duration::from_secs(1)
            .div_f64(150.0 * MHZ as f64)
            .div_f64(divisor);

        Ticks::from(duration)
    }

    pub fn is_interrupting(&self) -> bool {
        self.wrapped
    }

    pub fn update_csr(&mut self, value: u8) {
        let ph_advance = extract_bit(value, 7);
        let ph_ret = extract_bit(value, 6);
        self.csr = value & 0b11_1111;

        if ph_advance == 1 {
            if self.ctr == self.top {
                self.ctr = 0;
            } else {
                self.ctr += 1;
            }

            if self.ctr == 0 {
                self.wrapped = true;
            }
        }

        if ph_ret == 1 {
            if self.ctr == 0 {
                self.ctr = self.top;
            } else {
                self.ctr -= 1;
            }

            if self.ctr == 0 {
                self.wrapped = true;
            }
        }
    }

    pub fn clear_interrupt(&mut self) {
        self.wrapped = false;
    }

    pub fn is_enabled(&self) -> bool {
        extract_bit(self.csr, 0) == 1
    }

    pub fn enable(&mut self) {
        self.csr |= 1;
    }

    pub fn disable(&mut self) {
        self.csr &= !1;
    }

    pub fn ph_correct(&self) -> bool {
        extract_bit(self.csr, 1) == 1
    }

    pub fn invert_a(&self) -> bool {
        extract_bit(self.csr, 2) == 1
    }

    pub fn invert_b(&self) -> bool {
        extract_bit(self.csr, 3) == 1
    }

    pub(super) fn output_a(&self) -> bool {
        let output = self.ctr >= self.cc as u16;
        if self.invert_a() {
            !output
        } else {
            output
        }
    }

    pub(super) fn output_b(&self) -> bool {
        let output = self.ctr >= (self.cc >> 16) as u16;
        if self.invert_b() {
            !output
        } else {
            output
        }
    }

    pub fn divmode(&self) -> DivMode {
        match extract_bits(self.csr, 4..=5) {
            0 => DivMode::Div,
            1 => DivMode::Level,
            2 => DivMode::Rise,
            3 => DivMode::Fall,
            _ => unreachable!(),
        }
    }
}
// 2 function select, a and b
pub(super) fn channel_as_function_select(index: u8) -> (FunctionSelect, FunctionSelect) {
    match index {
        0 => (FunctionSelect::PWM0_A, FunctionSelect::PWM0_B),
        1 => (FunctionSelect::PWM1_A, FunctionSelect::PWM1_B),
        2 => (FunctionSelect::PWM2_A, FunctionSelect::PWM2_B),
        3 => (FunctionSelect::PWM3_A, FunctionSelect::PWM3_B),
        4 => (FunctionSelect::PWM4_A, FunctionSelect::PWM4_B),
        5 => (FunctionSelect::PWM5_A, FunctionSelect::PWM5_B),
        6 => (FunctionSelect::PWM6_A, FunctionSelect::PWM6_B),
        7 => (FunctionSelect::PWM7_A, FunctionSelect::PWM7_B),
        _ => unreachable!(),
    }
}
