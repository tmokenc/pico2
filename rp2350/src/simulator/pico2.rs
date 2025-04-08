use super::Rp2350;
use crate::common::*;
use crate::memory::GenericMemory;
use thiserror::Error;

#[derive(Debug, Clone, Copy, Error)]
pub enum SimulatorError {
    #[error("The file is too large")]
    FileTooLarge,

    #[error("Invalid address: {0:#X}")]
    InvalidAddress(u32),

    #[error("Invalid target address")]
    MemoryError(#[from] crate::memory::MemoryOutOfBoundsError),

    #[error("Invalid UF2 file")]
    UF2Error(#[from] uf2::Error),
}

/// A wrapper of the RP2350 MCU that represents the Raspberry Pi Pico 2 board.
#[derive(Default)]
pub struct Pico2 {
    pub mcu: Rp2350,
    pub flash: GenericMemory<{ 4 * MB }>,
}

impl Pico2 {
    const FLASH_ADDRESS: u32 = 0x1000_0000;

    pub fn new(flash: &[u8]) -> Self {
        assert!(flash.len() <= 4 * MB);

        Self {
            mcu: Rp2350::new(),
            flash: GenericMemory::new(flash),
        }
    }

    pub fn flash_bin(&mut self, bin: &[u8]) -> Result<(), SimulatorError> {
        if bin.len() > 4 * MB {
            return Err(SimulatorError::FileTooLarge);
        }

        self.mcu.bus.flash.write_slice(Self::FLASH_ADDRESS, bin)?;

        // Reset the MCU to start executing the new program
        self.reset();

        Ok(())
    }

    pub fn flash_uf2(&mut self, uf2: &[u8]) -> Result<(), SimulatorError> {
        if uf2.len() > 4 * MB {
            return Err(SimulatorError::FileTooLarge);
        }

        for block in uf2::read_uf2(uf2)? {
            let offset = block.target_addr - Self::FLASH_ADDRESS;
            self.mcu.bus.flash.write_slice(offset, &block.data)?;
        }

        // Reset the MCU to start executing the new program
        self.reset();

        Ok(())
    }

    pub fn reset(&mut self) {
        self.mcu = Rp2350::new();
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
