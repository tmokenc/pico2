use crate::gpio::GpioPinValue;
use super::*;
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
pub const UARTPeriphID0: u16 = 0xFE0; // UARTPeriphID0 Register
pub const UARTPeriphID1: u16 = 0xFE4; // UARTPeriphID1 Register
pub const UARTPeriphID2: u16 = 0xFE8; // UARTPeriphID2 Register
pub const UARTPeriphID3: u16 = 0xFEC; // UARTPeriphID3 Register
pub const UARTPCellID0: u16 = 0xFF0; // UARTPCellID0 Register
pub const UARTPCellID1: u16 = 0xFF4; // UARTPCellID1 Register
pub const UARTPCellID2: u16 = 0xFF8; // UARTPCellID2 Register
pub const UARTPCellID3: u16 = 0xFFC; // UARTPCellID3 Register

#[derive(Debug, Default)]
pub struct Uart {
    tx: GpioPinValue,
    rx: GpioPinValue,
}

impl Peripheral for Uart {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let value = match address {
            UARTDR  // TODO
            | UARTRSR 
            | UARTFR 
            | UARTILPR 
            | UARTIBRD 
            | UARTFBRD 
            | UARTLCR_H 
            | UARTCR
            | UARTIFLS 
            | UARTIMSC 
            | UARTRIS 
            | UARTMIS 
            | UARTICR 
            | UARTDMACR => 0, // TODO

            UARTPeriphID0 => 0x11,
            UARTPeriphID1 => 0x1 << 4,
            UARTPeriphID2 => (0x3 << 4) | 4,
            UARTPeriphID3 => 0,
            UARTPCellID0 => 0x0D,
            UARTPCellID1 => 0xF0,
            UARTPCellID2 => 0x05,
            UARTPCellID3 => 0xB1,
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
            | UARTIBRD 
            | UARTFBRD 
            | UARTLCR_H 
            | UARTCR
            | UARTIFLS 
            | UARTIMSC 
            | UARTRIS 
            | UARTMIS 
            | UARTICR 
            | UARTDMACR => (), // TODO

            UARTPeriphID0 | UARTPeriphID1 | UARTPeriphID2 | UARTPeriphID3 | UARTPCellID0
            | UARTPCellID1 | UARTPCellID2 | UARTPCellID3 => (), // Ignore writes to read-only
                                                               // registers

            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}
