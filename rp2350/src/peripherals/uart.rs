use crate::gpio::GpioPinValue;
use super::*;
use crate::utils::Fifo;
use std::cell::RefCell;

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

pub struct Uart {
    // receive are 12 bit wide
    rx_fifo: Fifo<u16, 32>,
    tx_fifo: Fifo<u8, 32>,

    baud_divint: u16,
    baud_divfrac: u8,
    ctrl: u32,
    line_ctrl: u32,

    is_interrupting: bool,
    pub tx: GpioPinValue,
}

impl Default for Uart {
    fn default() -> Self {
        Self {
            rx_fifo: Fifo::default(),
            tx_fifo: Fifo::default(),

            baud_divint: 0,
            baud_divfrac: 0,
            ctrl: 0,
            line_ctrl: 0,

            is_interrupting: false,
            tx: GpioPinValue::default(),
        }
    }
}

impl Uart {
    fn update_baudrate(&mut self, divint: u16, divfrac: u8) {
        todo!()
    }
}

impl Peripheral for Uart {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let value = match address {
            UARTDR  // TODO
            | UARTRSR 
            | UARTFR 
            | UARTILPR 
            | UARTLCR_H 
            | UARTCR
            | UARTIFLS 
            | UARTIMSC 
            | UARTRIS 
            | UARTMIS 
            | UARTICR 
            | UARTDMACR => 0, // TODO

            UARTIBRD => self.baud_divint as u32,
            UARTFBRD => self.baud_divfrac as u32,

            UARTPERIPHID0 => 0x11,
            UARTPERIPHID1 => 0x1 << 4,
            UARTPERIPHID2 => (0x3 << 4) | 4,
            UARTPERIPHID3 => 0,
            UARTPCELLID0 => 0x0D,
            UARTPCELLID1 => 0xF0,
            UARTPCELLID2 => 0x05,
            UARTPCELLID3 => 0xB1,
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
        match address {
            UARTDR  // TODO
            | UARTRSR 
            | UARTFR 
            | UARTILPR 
            | UARTLCR_H 
            | UARTCR
            | UARTIFLS 
            | UARTIMSC 
            | UARTICR 
            | UARTDMACR => (), // TODO

            UARTIBRD => self.update_baudrate(value as u16, self.baud_divfrac),
            UARTFBRD => self.update_baudrate(self.baud_divint, value as u8),

            UARTRIS | UARTMIS 
            | UARTPERIPHID0 | UARTPERIPHID1 | UARTPERIPHID2 | UARTPERIPHID3 | UARTPCELLID0
            | UARTPCELLID1 | UARTPCELLID2 | UARTPCELLID3 => (), // Ignore writes to read-only
                                                               // registers

            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}
