use crate::utils::{extract_bit, extract_bits};

#[derive(Clone, Copy)]
pub enum DivMode {
    Div,   // by the fraction divisor
    Level, // b pin
    Rise,  // rising edge on b pin
    Fall,  // falling edge on b pin
}

#[derive(Clone, Copy)]
pub struct PwmChannel {
    pub csr: u8,
    pub div: u16,
    pub ctr: u16,
    pub cc: u32,
    pub top: u16,
    pub irq_wrap_0: bool,
    pub irq_wrap_1: bool,
}

impl Default for PwmChannel {
    fn default() -> Self {
        Self {
            csr: 0,
            div: 1 << 4,
            ctr: 0,
            cc: 0,
            top: 0xffff,
            irq_wrap_0: false,
            irq_wrap_1: false,
        }
    }
}

impl PwmChannel {
    pub fn advance(&mut self) {
        if self.ph_correct() {
            // TODO
        }

        if self.invert_b() {
            // TODO
        }

        if self.invert_a() {
            // TODO
        }

        match self.divmode() {
            DivMode::Div => {
                self.ctr = (self.ctr + 1) % self.div;
            }
            DivMode::Level => {
                if self.ctr == self.div {
                    self.ctr = 0;
                }
            }
            DivMode::Rise => {
                if self.ctr == self.div {
                    self.irq_wrap_0 = true;
                }
            }
            DivMode::Fall => {
                if self.ctr == self.div {
                    self.irq_wrap_1 = true;
                }
            }
        }
    }

    pub fn next_update(&mut self) -> u64 {
        let mut next = 0;

        // calculate the next update time
        let div = self.div >> 4;
        let frac = self.div & 0x0f;

        // TODO

        next
    }

    pub fn is_interrupting(&self, index: u32) -> bool {
        let cmp = (self.cc >> (16 * index)) as u16;
        self.ctr >= cmp
    }

    pub fn update_csr(&mut self, value: u8) {
        let ph_advance = extract_bit(value, 7);
        let ph_ret = extract_bit(value, 6);
        self.csr = value & 0b11_1111;

        if ph_advance == 1 {
            self.ctr += 1;
        }

        if ph_ret == 1 {
            self.ctr -= 1;
        }
    }

    pub fn clear_interrupt(&mut self) {
        self.irq_wrap_0 = false;
        self.irq_wrap_1 = false;
    }

    pub fn enable(&mut self) {
        self.csr |= 1;
    }

    pub fn disable(&mut self) {
        self.csr &= !1;
    }

    pub fn is_enabled(&self) -> bool {
        extract_bit(self.csr, 0) == 1
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
