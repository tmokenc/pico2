use std::cell::RefCell;
use std::rc::Rc;

pub type Interrupt = u8;

pub struct InterruptIter(u64);

impl Iterator for InterruptIter {
    type Item = Interrupt;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }

        let trailing_zeros = self.0.trailing_zeros();
        self.0 &= !(1 << trailing_zeros);
        Some(trailing_zeros as u8)
    }
}

#[derive(Default)]
pub struct Interrupts {
    global: u64,
    core1: u64,
}

impl Interrupts {
    pub const TIMER0_IRQ_0: Interrupt = 0;
    pub const TIMER0_IRQ_1: Interrupt = 1;
    pub const TIMER0_IRQ_2: Interrupt = 2;
    pub const TIMER0_IRQ_3: Interrupt = 3;
    pub const TIMER1_IRQ_0: Interrupt = 4;
    pub const TIMER1_IRQ_1: Interrupt = 5;
    pub const TIMER1_IRQ_2: Interrupt = 6;
    pub const TIMER1_IRQ_3: Interrupt = 7;
    pub const PWM_IRQ_WRAP_0: Interrupt = 8;
    pub const PWM_IRQ_WRAP_1: Interrupt = 9;
    pub const DMA_IRQ_0: Interrupt = 10;
    pub const DMA_IRQ_1: Interrupt = 11;
    pub const DMA_IRQ_2: Interrupt = 12;
    pub const DMA_IRQ_3: Interrupt = 13;
    pub const USBCTRL_IRQ: Interrupt = 14;
    pub const PIO0_IRQ_0: Interrupt = 15;
    pub const PIO0_IRQ_1: Interrupt = 16;
    pub const PIO1_IRQ_0: Interrupt = 17;
    pub const PIO1_IRQ_1: Interrupt = 18;
    pub const PIO2_IRQ_0: Interrupt = 19;
    pub const PIO2_IRQ_1: Interrupt = 20;
    pub const IQ_IRQ_BANK0: Interrupt = 21;
    pub const IQ_IRQ_BANK0_NS: Interrupt = 22;
    pub const IQ_IRQ_QSPI: Interrupt = 23;
    pub const IQ_IRQ_QSPI_NS: Interrupt = 24;
    pub const SIO_IRQ_FIFO: Interrupt = 25;
    pub const SIO_IRQ_BELL: Interrupt = 26;
    pub const SIO_IRQ_FIFO_NS: Interrupt = 27;
    pub const SIO_IRQ_BELL_NS: Interrupt = 28;
    pub const SIO_IRQ_MTIMECMP: Interrupt = 29;
    pub const CLOCKS_IRQ: Interrupt = 30;
    pub const SPI0_IRQ: Interrupt = 31;
    pub const SPI1_IRQ: Interrupt = 32;
    pub const UART0_IRQ: Interrupt = 33;
    pub const UART1_IRQ: Interrupt = 34;
    pub const ADC_IRQ_FIFO: Interrupt = 35;
    pub const I2C0_IRQ: Interrupt = 36;
    pub const I2C1_IRQ: Interrupt = 37;
    pub const OTP_IRQ: Interrupt = 38;
    pub const TRNG_IRQ: Interrupt = 39;
    pub const PROC0_IRQ_CTI: Interrupt = 40;
    pub const PROC1_IRQ_CTI: Interrupt = 41;
    pub const PLL_SYS_IRQ: Interrupt = 42;
    pub const PLL_USB_IRQ: Interrupt = 43;
    pub const POWMAN_IRQ_POW: Interrupt = 44;
    pub const POWMAN_IRQ_TIMER: Interrupt = 45;

    // Never firing
    pub const _SPAREIRQ_IRQ_0: Interrupt = 46;
    pub const _SPAREIRQ_IRQ_1: Interrupt = 47;
    pub const _SPAREIRQ_IRQ_2: Interrupt = 48;
    pub const _SPAREIRQ_IRQ_3: Interrupt = 49;
    pub const _SPAREIRQ_IRQ_4: Interrupt = 50;
    pub const _SPAREIRQ_IRQ_5: Interrupt = 51;

    // Core local interrupts are located from 21th to 29th bits
    const CORE_LOCAL_IRQS_MASK: u64 = 0x1FF << 21;

    /// Enable the IRQ for the given core
    pub fn set_irq(&mut self, irq: Interrupt, value: bool) {
        if value {
            self.global |= 1 << irq;
        } else {
            self.clear_irq(irq);
        }
    }

    pub fn set_core_local_irq(&mut self, core: u8, irq: Interrupt, value: bool) {
        if value {
            if core == 0 {
                self.global |= 1 << irq;
            } else {
                self.core1 |= 1 << irq;
            }
        } else {
            self.clear_core_local_irq(core, irq);
        }
    }

    pub fn clear_irq(&mut self, irq: Interrupt) {
        self.global &= !(1 << irq);
    }

    pub fn clear_core_local_irq(&mut self, core: u8, irq: Interrupt) {
        if core == 0 {
            self.global &= !(1 << irq);
        } else {
            self.core1 &= !(1 << irq);
        }
    }

    pub fn iter(&self, core: u8) -> InterruptIter {
        if core == 0 {
            InterruptIter(self.global)
        } else {
            let global = self.global;
            let core_local = self.core1;

            // clear bits that are core local from global then combine with core 1 local
            InterruptIter((global & !Self::CORE_LOCAL_IRQS_MASK) | core_local)
        }
    }

    pub fn update(&mut self) {
        // do nothing for now...
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interrupts() {
        let mut interrupts = Interrupts::default();

        assert!(interrupts.iter(0).next().is_none());
        assert!(interrupts.iter(1).next().is_none());

        interrupts.set_irq(Interrupts::TIMER0_IRQ_0, true);

        assert_eq!(interrupts.iter(0).next(), Some(Interrupts::TIMER0_IRQ_0));
        interrupts.clear_irq(Interrupts::TIMER0_IRQ_0);

        assert!(interrupts.iter(0).next().is_none());
        interrupts.set_irq(Interrupts::TIMER0_IRQ_1, false);

        assert!(interrupts.iter(0).next().is_none());
    }
}
