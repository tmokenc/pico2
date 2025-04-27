use super::Rp2350;
use crate::common::*;
use core::ops::{Deref, DerefMut};

/// A wrapper of the RP2350 MCU that represents the Raspberry Pi Pico 2 board.
#[derive(Default)]
pub struct Pico2 {
    pub mcu: Rp2350,
    pub is_flashed: bool,
}

impl Deref for Pico2 {
    type Target = Rp2350;

    fn deref(&self) -> &Self::Target {
        &self.mcu
    }
}

impl DerefMut for Pico2 {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.mcu
    }
}

impl Pico2 {
    pub fn new(flash: &[u8]) -> Self {
        assert!(flash.len() <= 4 * MB);

        Self {
            mcu: Rp2350::new(),
            is_flashed: false,
        }
    }

    pub fn step(&mut self) {
        self.mcu.tick();
    }

    pub fn led_state(&self) -> LedState {
        if self.mcu.gpio.borrow().get_pin(25).value.is_high() {
            LedState::On
        } else {
            LedState::Off
        }
    }

    pub fn set_gpio(&mut self, index: u8, value: f32) {
        assert!(index < 30, "Invalid GPIO pin index: {}", index);
        let mut gpio = self.mcu.gpio.borrow_mut();
        gpio.get_pin_mut(index).value.set_value(value);
    }
}
