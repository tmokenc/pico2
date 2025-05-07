/**
 * @file peripherals/dma/timer.rs
 * @author Nguyen Le Duy
 * @date 29/04/2025
 * @brief DMA timer implementation
 */
#[derive(Clone, Copy, Debug)]
pub struct Timer {
    pub x: u16,
    pub y: u16,
}

impl Default for Timer {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

impl From<u32> for Timer {
    fn from(value: u32) -> Self {
        Self {
            x: (value >> 16) as u16,
            y: value as u16,
        }
    }
}

impl From<Timer> for u32 {
    fn from(timer: Timer) -> Self {
        ((timer.x as u32) << 16) | (timer.y as u32)
    }
}

impl Timer {
    pub fn ticks(&self) -> u64 {
        let nom = self.x as u64;
        let denom = self.y as u64;

        if denom == 0 {
            return 0;
        }

        nom / denom
    }
}
