use super::*;
use crate::clock::Clock;
use sha2::Digest;
use std::cell::RefCell;
use std::rc::Rc;

pub const CSR: u16 = 0x0000; // Control and status register
pub const WDATA: u16 = 0x0004; // Write data register
pub const SUM0: u16 = 0x0008; // 256-bit checksum result. Contents are undefined when CSR_SUM_VLD is 0.
pub const SUM1: u16 = 0x000c; // 256-bit checksum result. Contents are undefined when CSR_SUM_VLD is 0.
pub const SUM2: u16 = 0x0010; // 256-bit checksum result. Contents are undefined when CSR_SUM_VLD is 0.
pub const SUM3: u16 = 0x0014; // 256-bit checksum result. Contents are undefined when CSR_SUM_VLD is 0.
pub const SUM4: u16 = 0x0018; // 256-bit checksum result. Contents are undefined when CSR_SUM_VLD is 0.
pub const SUM5: u16 = 0x001c; // 256-bit checksum result. Contents are undefined when CSR_SUM_VLD is 0.
pub const SUM6: u16 = 0x0020; // 256-bit checksum result. Contents are undefined when CSR_SUM_VLD is 0.
pub const SUM7: u16 = 0x0024; // 256-bit checksum result. Contents are undefined when CSR_SUM_VLD is 0.

// Control and status register bits
const CSR_SUM_VLD: u32 = 1 << 2; // Checksum valid
const CSR_WDATA_RDY: u32 = 1 << 1; // Write data ready
const CSR_START: u32 = 1 << 0; // Start checksum calculation
const CSR_ERR_WDATA_NOT_RDY: u32 = 1 << 4;
const CSR_BSWAP: u32 = 1 << 12; // Byte swap

pub struct Sha256 {
    pub bswap: bool,
    pub dma_size: u8,
    pub err_wdata_not_rdy: bool,
    pub sum_vld: bool,
    pub wdata_rdy: bool,
    // 256 bits result
    pub sum: [u8; 32],
    pub writed_count: u8,

    sha_core: sha2::Sha256,
}

impl Default for Sha256 {
    fn default() -> Self {
        Self {
            sum: [0; 32],
            writed_count: 0,
            bswap: true,
            dma_size: 2,
            err_wdata_not_rdy: false,
            sum_vld: true,
            wdata_rdy: true,
            sha_core: sha2::Sha256::new(),
        }
    }
}

impl Sha256 {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Peripheral for Rc<RefCell<Sha256>> {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let inner = self.as_ref().borrow();
        let value = match address {
            CSR => {
                ((inner.wdata_rdy as u32) << 1)
                    | ((inner.sum_vld as u32) << 2)
                    | ((inner.err_wdata_not_rdy as u32) << 4)
                    | ((inner.dma_size as u32) << 8)
                    | ((inner.bswap as u32) << 16)
            }
            WDATA => {
                /* this register should not be read */
                0
            }
            SUM0 | SUM1 | SUM2 | SUM3 | SUM4 | SUM5 | SUM6 | SUM7 => {
                if !inner.sum_vld {
                    0 // undefined
                } else {
                    // get 32-bit value from the sum array
                    let index = (address - SUM0) / 4;
                    let data = inner
                        .sum
                        .chunks_exact(4)
                        .nth(index as usize)
                        .unwrap()
                        .try_into()
                        .unwrap();

                    u32::from_le_bytes(data)
                }
            }
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
        let mut inner = self.as_ref().borrow_mut();

        match address {
            CSR => {
                if value & CSR_START != 0 {
                    inner.wdata_rdy = true;
                    inner.sum_vld = true;
                }

                if value & CSR_ERR_WDATA_NOT_RDY != 0 {
                    inner.err_wdata_not_rdy = false;
                }

                inner.dma_size = ((value >> 8) & 0b11) as u8;
                inner.bswap = value & CSR_BSWAP != 0;
            }
            WDATA => {
                if !inner.wdata_rdy {
                    inner.err_wdata_not_rdy = true;
                    return Ok(()); // nothing to do
                }

                let bytes = if inner.bswap {
                    value.swap_bytes().to_le_bytes()
                } else {
                    value.to_le_bytes()
                };

                inner.sha_core.update(&bytes);
                inner.writed_count += bytes.len() as u8;
                inner.sum_vld = false;

                if inner.writed_count < 64 {
                    return Ok(()); // not enough data
                }

                // Then sleep for 57 cycles to simulate the computation of the hash
                inner.wdata_rdy = false;

                drop(inner);
                let self_clone = Rc::clone(self);

                ctx.clock
                    .as_ref()
                    .borrow_mut()
                    .schedule(57, "TRNG", move || {
                        let mut inner = self_clone.as_ref().borrow_mut();
                        let sha_core = core::mem::take(&mut inner.sha_core);
                        let result = sha_core.finalize();
                        inner.sum = result.into();
                        inner.sum_vld = true;
                        inner.writed_count = 0;
                    });
            }
            SUM0 | SUM1 | SUM2 | SUM3 | SUM4 | SUM5 | SUM6 | SUM7 => { /* Read Only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}
