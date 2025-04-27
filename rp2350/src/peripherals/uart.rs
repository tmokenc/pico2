//! TODO 
//! DMA support
//! Interrupt: UARTINTR and UARTRTINTR
//! Unsure what they do when reading the datasheet

#![allow(dead_code)]

use super::*;
use crate::utils::{extract_bit, extract_bits, w1c, Fifo};
use std::cell::RefCell;
use std::time::Duration;

mod transmit;
mod receive;

use transmit::*;
use receive::*;

pub const UARTDR: u16 = 0x000; // Data Register, UARTDR
pub const UARTRSR: u16 = 0x004; // Receive Status Register/Error Clear Register, UARTRSR/UARTECR
pub const UARTFR: u16 = 0x018; // Flag Register, UARTFR
pub const UARTILPR: u16 = 0x020; // IRDA Low-Power Counter Register, UARTILPR
pub const UARTIBRD: u16 = 0x024; // Integer Baud Rate Register, UARTIBRD
pub const UARTFBRD: u16 = 0x028; // Fractional Baud Rate Register, UARTFBRD
pub const UARTLCR_H: u16 = 0x02C; // Line Control Register, UARTLCR_H
pub const UARTCR: u16 = 0x030; // Control Register, UARTCR
pub const UARTIFLS: u16 = 0x034; // Interrupt FIFO Level Select Register, UARTIFLS
pub const UARTIMSC: u16 = 0x038; // Interrupt Mask Set/Clear Register, UARTIMSC
pub const UARTRIS: u16 = 0x03C; // Raw Interrupt Status Register, UARTRIS
pub const UARTMIS: u16 = 0x040; // Masked Interrupt Status Register, UARTMIS
pub const UARTICR: u16 = 0x044; // Interrupt Clear Register, UARTICR
pub const UARTDMACR: u16 = 0x048; // DMA Control Register, UARTDMACR
pub const UARTPERIPHID0: u16 = 0xFE0; // UARTPeriphID0 Register
pub const UARTPERIPHID1: u16 = 0xFE4; // UARTPeriphID1 Register
pub const UARTPERIPHID2: u16 = 0xFE8; // UARTPeriphID2 Register
pub const UARTPERIPHID3: u16 = 0xFEC; // UARTPeriphID3 Register
pub const UARTPCELLID0: u16 = 0xFF0; // UARTPCellID0 Register
pub const UARTPCELLID1: u16 = 0xFF4; // UARTPCellID1 Register
pub const UARTPCELLID2: u16 = 0xFF8; // UARTPCellID2 Register
pub const UARTPCELLID3: u16 = 0xFFC; // UARTPCellID3 Register

const FIFO_DEPTH: usize = 32;


const FRAME_ERROR: u8 = 0x1 << 0;
const PARITY_ERROR: u8 = 0x1 << 1;
const BREAK_ERROR: u8 = 0x1 << 2;
const OVERRUN_ERROR: u8 = 0x1 << 3;

const FLAG_CTS: u32 = 0x1 << 0;
const FLAG_DSR: u32 = 0x1 << 1;
const FLAG_DCD: u32 = 0x1 << 2;
const FLAG_BUSY: u32 = 0x1 << 3;
const FLAG_RXFE: u32 = 0x1 << 4;
const FLAG_TXFF: u32 = 0x1 << 5;
const FLAG_RXFF: u32 = 0x1 << 6;
const FLAG_TXFE: u32 = 0x1 << 7;
const FLAG_RI: u32 = 0x1 << 8;

const CTRL_UARTEN: u16 = 0x1 << 0;
const CTRL_SIREN: u16 = 0x1 << 1;
const CTRL_SIRLP: u16 = 0x1 << 2;
const CTRL_LBE: u16 = 0x1 << 7;
const CTRL_TXE: u16 = 0x1 << 8;
const CTRL_RXE: u16 = 0x1 << 9;
const CTRL_DTR: u16 = 0x1 << 10;
const CTRL_RTS: u16 = 0x1 << 11;
const CTRL_OUT1: u16 = 0x1 << 12;
const CTRL_OUT2: u16 = 0x1 << 13;
const CTRL_RTSEN: u16 = 0x1 << 14;
const CTRL_CTSEN: u16 = 0x1 << 15;

const LINE_CTRL_BRK: u8 = 0x1 << 0;
const LINE_CTRL_PEN: u8 = 0x1 << 1;
const LINE_CTRL_EPS: u8 = 0x1 << 2;
const LINE_CTRL_STP2: u8 = 0x1 << 3;
const LINE_CTRL_FEN: u8 = 0x1 << 4;
const LINE_CTRL_WLEN: u8 = 0x3 << 5;
const LINE_CTRL_SPS: u8 = 0x1 << 7;

const IRQ_UARTRXINTR: u16 = 0x1 << 4;
const IRQ_UARTTXINTR: u16 = 0x1 << 5;

pub struct Uart<const IDX: usize> {
    // receive are 12 bit wide
    rx_fifo: Fifo<u16, FIFO_DEPTH>,
    tx_fifo: Fifo<u8, FIFO_DEPTH>,

    baud_divint: u16,
    baud_divfrac: u8,
    ctrl: u16,
    line_ctrl: u8,
    flags: u32,

    interrupt_status: u16,
    interrupt_mask: u16,
    error: u8,
    fifo_level_select: u8,

    dma_ctrl: u8,
}

impl<const IDX: usize> Default for Uart<IDX> {
    fn default() -> Self {
        Self {
            rx_fifo: Default::default(),
            tx_fifo: Default::default(),

            baud_divint: 0,
            baud_divfrac: 0,
            ctrl: CTRL_RXE | CTRL_TXE,
            line_ctrl: 0,
            flags: FLAG_TXFE | FLAG_RXFE,
            fifo_level_select: (0x2 << 3) | 0x2,

            interrupt_status: Default::default(),
            interrupt_mask: Default::default(),
            error: 0,
            dma_ctrl: 0,
        }
    }
}

impl<const IDX: usize> Uart<IDX> {
    fn update_baudrate(&mut self, divint: impl Into<Option<u16>>, divfrac: impl Into<Option<u8>>) {
        if let Some(divint) = divint.into() {
            self.baud_divint = divint;
        }

        if let Some(divfrac) = divfrac.into() {
            self.baud_divfrac = divfrac;
        }
        // TODO
    }

    fn set_busy(&mut self, busy: bool) {
        if busy {
            self.flags |= FLAG_BUSY;
        } else {
            self.flags &= !FLAG_BUSY;
        }
    }

    fn get_baudrate(&self) -> u32 {
        let baudrate = (self.baud_divint as u32 * 16 + self.baud_divfrac as u32) * 1000;
        baudrate
    }

    fn get_bit_time(&self) -> Duration {
        Duration::from_secs(1) / self.get_baudrate()
    }

    fn fifo_level(&self, level: u8) -> u8 {
        match level {
            0b000 => (FIFO_DEPTH as u8 / 8) * 1,
            0b001 => (FIFO_DEPTH as u8 / 4) * 1,
            0b010 => (FIFO_DEPTH as u8 / 2) * 1,
            0b011 => (FIFO_DEPTH as u8 / 4) * 3,
            0b100 => (FIFO_DEPTH as u8 / 8) * 7,
            _ => (FIFO_DEPTH as u8 / 2) * 1 // reserved, but use the default value here
        }
    }

    fn transmit_interrupt_fifo_level(&self) -> u8 {
        self.fifo_level(extract_bits(self.line_ctrl, 0..=2))
    }

    fn receive_interrupt_fifo_level(&self) -> u8 {
        self.fifo_level(extract_bits(self.line_ctrl, 3..=5))
    }

    fn dma_tx_enabled(&self) -> bool {
        extract_bit(self.dma_ctrl, 1) != 0
    }

    fn dma_rx_enabled(&self) -> bool {
        extract_bit(self.dma_ctrl, 0) != 0
    }

    fn dma_on_err(&self) -> bool {
        extract_bit(self.dma_ctrl, 2) != 0
    }

    fn is_enabled(&self) -> bool {
        extract_bit(self.ctrl, 0) != 0
    }

    fn is_transmit_enabled(&self) -> bool {
        extract_bit(self.ctrl, 8) != 0
    }

    fn is_receive_enabled(&self) -> bool {
        extract_bit(self.ctrl, 9) != 0
    }

    fn is_fifo_enabled(&self) -> bool {
        extract_bit(self.ctrl, 4) != 0
    }

    fn word_len(&self) -> u8 {
        match extract_bits(self.line_ctrl, 5..=6) {
            0b00 => 5,
            0b01 => 6,
            0b10 => 7,
            0b11 => 8,
            _ => unreachable!(),
        }
    }

    pub fn is_parity_enabled(&self) -> bool {
        extract_bit(self.line_ctrl, 1) != 0
    }

    pub fn is_parity_even(&self) -> bool {
        extract_bit(self.line_ctrl, 2) != 0
    }

    pub fn two_stop_bits(&self) -> bool {
        extract_bit(self.line_ctrl, 3) != 0
    }

    fn stick_parity(&self) -> bool {
        extract_bit(self.line_ctrl, 7) != 0
    }

    fn update_interrupt(&mut self, interrupts: Rc<RefCell<Interrupts>>) {
        if self.is_fifo_enabled() {
            if self.tx_fifo.len() as u8 >= self.transmit_interrupt_fifo_level() {
                self.interrupt_status |= IRQ_UARTTXINTR;
            } else {
                self.interrupt_status &= !IRQ_UARTTXINTR;
            }

            if self.rx_fifo.len() as u8 >= self.receive_interrupt_fifo_level() {
                self.interrupt_status |= IRQ_UARTRXINTR;
            } else {
                self.interrupt_status &= !IRQ_UARTRXINTR;
            }
        } else {
            if self.tx_fifo.len() > 0 {
                self.interrupt_status |= IRQ_UARTTXINTR;
            } else {
                self.interrupt_status &= !IRQ_UARTTXINTR;
            }

            if self.rx_fifo.len() > 0 {
                self.interrupt_status |= IRQ_UARTRXINTR;
            } else {
                self.interrupt_status &= !IRQ_UARTRXINTR;
            }
        }

        let error_irq_mask = 0b1111 << 7;
        self.interrupt_status &= !error_irq_mask;
        self.interrupt_status |= (self.error as u16) << 7;

        let int = self.interrupt_mask & self.interrupt_status;
        let int_num = Interrupts::UART0_IRQ + IDX as u8;

        interrupts.borrow_mut().set_irq(int_num, int != 0);
    }

    fn check_tx_fifo(&mut self) {
        if self.tx_fifo.len() as u8 >= self.transmit_interrupt_fifo_level() {
            self.interrupt_status |= IRQ_UARTTXINTR;
        } else {
            self.interrupt_status &= !IRQ_UARTTXINTR;
        }

        if self.tx_fifo.is_empty() {
            self.flags |= FLAG_TXFE;
        } else {
            self.flags &= !FLAG_TXFE;
        }

        if self.tx_fifo.is_full() {
            self.flags |= FLAG_TXFF;
        } else {
            self.flags &= !FLAG_TXFF;
        }
    }

    fn check_rx_fifo(&mut self) {
        if self.rx_fifo.len() as u8 >= self.receive_interrupt_fifo_level() {
            self.interrupt_status |= IRQ_UARTRXINTR;
        } else {
            self.interrupt_status &= !IRQ_UARTRXINTR;
        }

        if self.rx_fifo.is_empty() {
            self.flags |= FLAG_RXFE;
        } else {
            self.flags &= !FLAG_RXFE;
        }

        if self.rx_fifo.is_full() {
            self.flags |= FLAG_RXFF;
        } else {
            self.flags &= !FLAG_RXFF;
        }
    }
}

impl<const IDX: usize> Peripheral for Rc<RefCell<Uart<IDX>>> {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let mut uart = self.borrow_mut();

        let value = match address {
            UARTDR => {
                // TODO
                let value = match uart.rx_fifo.pop() {
                    Some(value) => value as u32,
                    None => 0,
                };

                uart.flags &= !FLAG_RXFF;

                if uart.rx_fifo.is_empty() {
                    uart.flags |= FLAG_RXFE;
                } else {
                    uart.flags &= !FLAG_RXFE;
                }

                uart.update_interrupt(ctx.interrupts.clone());

                value as u32
            }

            UARTRSR => uart.error as u32,
            UARTFR => uart.flags,
            UARTIFLS => uart.fifo_level_select as u32,

            UARTIBRD => uart.baud_divint as u32,
            UARTFBRD => uart.baud_divfrac as u32,

            UARTLCR_H => uart.line_ctrl as u32,
            UARTCR => uart.ctrl as u32,

            UARTIMSC => uart.interrupt_mask as u32,
            UARTICR | UARTRIS => uart.interrupt_status as u32,
            UARTMIS => (uart.interrupt_status & uart.interrupt_mask) as u32,


            UARTDMACR => uart.dma_ctrl as u32,

            UARTPERIPHID0 => 0x11,
            UARTPERIPHID1 => 0x1 << 4,
            UARTPERIPHID2 => (0x3 << 4) | 4,
            UARTPERIPHID3 => 0,
            UARTPCELLID0 => 0x0D,
            UARTPCELLID1 => 0xF0,
            UARTPCELLID2 => 0x05,
            UARTPCELLID3 => 0xB1,


            UARTILPR  => 0, // TODO

            _ => return Err(PeripheralError::OutOfBounds),
        };

        Ok(value)
    }

    fn write_raw(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        let mut uart = self.borrow_mut();
        match address {
            UARTRSR  // TODO
            | UARTFR
            | UARTILPR 
            | UARTIFLS => (), // TODO

            UARTDR => {
                // TODO not sure if the data should be added to FIFO while the UART is disabled 
                // or transmit is disabled
                if uart.is_fifo_enabled() {
                    let _ = uart.tx_fifo.push(value as u8); // if the FIFO is full, the value will be dropped
                } else if uart.tx_fifo.is_empty() {
                    uart.tx_fifo.push(value as u8).unwrap();
                }

                uart.check_tx_fifo();
                
                if uart.is_enabled() && uart.is_transmit_enabled() {
                    drop(uart);
                    start_transmitting(self.clone(), ctx);
                }
            }
                              
            UARTCR => {
                uart.ctrl = value as u16;

                if uart.is_enabled() {
                    uart.flags &= !FLAG_BUSY;
                    let is_transmit_enabled = uart.is_transmit_enabled();
                    let is_receive_enabled = uart.is_receive_enabled();
                    drop(uart); // avoid deadlock

                    if is_transmit_enabled {
                        start_transmitting(self.clone(), ctx);
                    }
                    
                    if is_receive_enabled {
                        start_receiving(self.clone(), ctx);
                    } else {
                        abort_receiving(self.clone(), ctx);
                    }

                }
            }
            UARTLCR_H => uart.line_ctrl = value as u8,
            UARTDMACR => uart.dma_ctrl = value as u8 & 0b11,

            UARTIMSC => {
                uart.interrupt_mask = value as u16 & 0b11_1111_1111;
                uart.update_interrupt(ctx.interrupts.clone());
            }

            UARTICR => {
                let mut interrupt = uart.interrupt_status as u32;
                w1c(&mut interrupt, value, 0b11_1111_1111);
                uart.interrupt_status = interrupt as _;
                uart.update_interrupt(ctx.interrupts.clone());
            }

            UARTIBRD => uart.update_baudrate(value as u16, None),
            UARTFBRD => uart.update_baudrate(None, value as u8),

            UARTRIS | UARTMIS 
            | UARTPERIPHID0 | UARTPERIPHID1 | UARTPERIPHID2 | UARTPERIPHID3 | UARTPCELLID0
            | UARTPCELLID1 | UARTPCELLID2 | UARTPCELLID3 => (), // Ignore writes to read-only
                                                               // registers

            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}

pub(self) fn get_even_parity(value: u8, word_len: u8) -> u8 {
    let mut parity = 0;

    for i in 0..word_len {
        parity ^= extract_bit(value, i);
    }

    parity
}

pub(self) fn get_odd_parity(value: u8, word_len: u8) -> u8 {
    let mut parity = 0;

    for i in 0..word_len {
        parity ^= extract_bit(value, i);
    }

    parity
}

