/**
 * @file peripherals/i2c.rs
 * @author Nguyen Le Duy
 * @date 04/05/2025
 * @brief I2C peripheral implementation
 * @todo actually implement the I2C peripheral
 */
use crate::interrupts::{Interrupt, Interrupts};
use crate::utils::{extract_bit, set_bit, set_bit_state, Fifo};

use super::{Peripheral, PeripheralAccessContext, PeripheralError, PeripheralResult};
use std::cell::RefCell;
use std::rc::Rc;

pub const IC_CON: u16 = 0x00; // I2C Control Register
pub const IC_TAR: u16 = 0x04; // I2C Target Address
pub const IC_SAR: u16 = 0x08; // I2C Slave Address
pub const IC_DATA_CMD: u16 = 0x10; // I2C Rx/Tx Data Buffer and Command Register
pub const IC_SS_SCL_HCNT: u16 = 0x14; // Standard Speed I2C Clock SCL High Count Register
pub const IC_SS_SCL_LCNT: u16 = 0x18; // Standard Speed I2C Clock SCL Low Count Register
pub const IC_FS_SCL_HCNT: u16 = 0x1C; // Fast Mode or Fast Mode Plus I2C Clock SCL High Count Register
pub const IC_FS_SCL_LCNT: u16 = 0x20; // Fast Mode or Fast Mode Plus I2C Clock SCL Low Count Register
pub const IC_INTR_STAT: u16 = 0x2C; // I2C Interrupt Status Register
pub const IC_INTR_MASK: u16 = 0x30; // I2C Interrupt Mask Register
pub const IC_RAW_INTR_STAT: u16 = 0x34; // I2C Raw Interrupt Status Register
pub const IC_RX_TL: u16 = 0x38; // I2C Receive FIFO Threshold Register
pub const IC_TX_TL: u16 = 0x3C; // I2C Transmit FIFO Threshold Register
pub const IC_CLR_INTR: u16 = 0x40; // Clear Combined and Individual Interrupt Register
pub const IC_CLR_RX_UNDER: u16 = 0x44; // Clear RX_UNDER Interrupt Register
pub const IC_CLR_RX_OVER: u16 = 0x48; // Clear RX_OVER Interrupt Register
pub const IC_CLR_TX_OVER: u16 = 0x4C; // Clear TX_OVER Interrupt Register
pub const IC_CLR_RD_REQ: u16 = 0x50; // Clear RD_REQ Interrupt Register
pub const IC_CLR_TX_ABRT: u16 = 0x54; // Clear TX_ABRT Interrupt Register
pub const IC_CLR_RX_DONE: u16 = 0x58; // Clear RX_DONE Interrupt Register
pub const IC_CLR_ACTIVITY: u16 = 0x5C; // Clear ACTIVITY Interrupt Register
pub const IC_CLR_STOP_DET: u16 = 0x60; // Clear STOP_DET Interrupt Register
pub const IC_CLR_START_DET: u16 = 0x64; // Clear START_DET Interrupt Register
pub const IC_CLR_GEN_CALL: u16 = 0x68; // Clear GEN_CALL Interrupt Register
pub const IC_ENABLE: u16 = 0x6C; // I2C ENABLE Register
pub const IC_STATUS: u16 = 0x70; // I2C STATUS Register
pub const IC_TXFLR: u16 = 0x74; // I2C Transmit FIFO Level Register
pub const IC_RXFLR: u16 = 0x78; // I2C Receive FIFO Level Register
pub const IC_SDA_HOLD: u16 = 0x7C; // I2C SDA Hold Time Length Register
pub const IC_TX_ABRT_SOURCE: u16 = 0x80; // I2C Transmit Abort Source Register
pub const IC_SLV_DATA_NACK_ONLY: u16 = 0x84; // Generate Slave Data NACK Register
pub const IC_DMA_CR: u16 = 0x88; // DMA Control Register
pub const IC_DMA_TDLR: u16 = 0x8C; // DMA Transmit Data Level Register
pub const IC_DMA_RDLR: u16 = 0x90; // DMA Transmit Data Level Register
pub const IC_SDA_SETUP: u16 = 0x94; // I2C SDA Setup Register
pub const IC_ACK_GENERAL_CALL: u16 = 0x98; // I2C ACK General Call Register
pub const IC_ENABLE_STATUS: u16 = 0x9C; // I2C Enable Status Register
pub const IC_FS_SPKLEN: u16 = 0xA0; // I2C SS, FS or FM+ spike suppression limit
pub const IC_CLR_RESTART_DET: u16 = 0xA8; // Clear RESTART_DET Interrupt Register
pub const IC_COMP_PARAM_1: u16 = 0xF4; // Component Parameter Register 1
pub const IC_COMP_VERSION: u16 = 0xF8; // I2C Component Version Register
pub const IC_COMP_TYPE: u16 = 0xFC; // I2C Component Type Register

pub struct I2c<const IDX: usize> {
    pub ctrl: u32,
    pub ic_enable: u8,
    pub ic_status: u8,
    pub target_address: u32,
    pub slave_address: u32,
    pub fsclk_hcnt: u16,
    pub fsclk_lcnt: u16,
    pub ssclk_hcnt: u16,
    pub ssclk_lcnt: u16,
    pub sda_setup: u8,
    pub ack_general_call: bool,
    pub ic_fs_spklen: u8,
    pub receive_data_level: u8,
    pub transmit_data_level: u8,
    pub dma_ctrl: u8,
    pub generate_nack: bool,
    pub tx_fifo: Fifo<u32, 16>,
    pub rx_fifo: Fifo<u32, 16>,

    interrupt_raw: u32,
    interrupt_mask: u32,
}

impl<const IDX: usize> Default for I2c<IDX> {
    fn default() -> Self {
        Self {
            ctrl: (1 << 6) | (1 << 5) | (0x2 << 1) | (1 << 0),
            target_address: 0x055,
            slave_address: 0x055,
            fsclk_hcnt: 0x0006,
            fsclk_lcnt: 0x000d,
            ssclk_hcnt: 0x0028,
            ssclk_lcnt: 0x002f,
            sda_setup: 0x64,
            ack_general_call: true,
            ic_fs_spklen: 0x07,
            receive_data_level: 0,
            transmit_data_level: 0,
            dma_ctrl: 0,
            generate_nack: false,
            ic_enable: 0,
            ic_status: 0b110,
            tx_fifo: Fifo::default(),
            rx_fifo: Fifo::default(),

            interrupt_raw: 0,
            interrupt_mask: 0,
        }
    }
}

impl<const IDX: usize> I2c<IDX> {
    pub fn is_enabled(&self) -> bool {
        extract_bit(self.ic_enable, 0) == 1
    }

    pub fn is_slave_active(&self) -> bool {
        extract_bit(self.ic_status, 6) == 1
    }

    pub fn is_master_active(&self) -> bool {
        extract_bit(self.ic_status, 5) == 1
    }

    pub fn receive_dma_enabled(&self) -> bool {
        extract_bit(self.dma_ctrl, 0) == 1
    }

    pub fn transmit_dma_enabled(&self) -> bool {
        extract_bit(self.dma_ctrl, 1) == 1
    }

    pub fn interrupt(&self) -> u32 {
        self.interrupt_raw & self.interrupt_mask
    }

    pub fn update_status(&mut self) {
        // TODO Master/slave active (FSM not in idle state)
        // enable / activity status
        //
        let mut status = self.ic_status as u32;

        set_bit_state(&mut status, 4, self.rx_fifo.is_full());
        set_bit_state(&mut status, 3, !self.rx_fifo.is_empty());
        set_bit_state(&mut status, 2, self.tx_fifo.is_full());
        set_bit_state(&mut status, 1, !self.tx_fifo.is_empty());
        self.ic_status = status as u8;
    }

    pub fn update_interrupt(&mut self, interrupts: Rc<RefCell<Interrupts>>) {
        // TODO update

        let irq = self.interrupt();

        interrupts
            .borrow_mut()
            .set_irq(Self::num_interrupt(), irq != 0);
    }

    fn num_interrupt() -> Interrupt {
        match IDX {
            0 => Interrupts::I2C0_IRQ,
            1 => Interrupts::I2C1_IRQ,
            _ => unreachable!(),
        }
    }
}

impl<const IDX: usize> Peripheral for Rc<RefCell<I2c<IDX>>> {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let mut i2c = self.borrow_mut();

        let value = match address {
            IC_CON => 0x00,      // I2C Control Register
            IC_TAR => 0x04,      // I2C Target Address
            IC_SAR => 0x08,      // I2C Slave Address
            IC_DATA_CMD => 0x10, // I2C Rx/Tx Data Buffer and Command Register
            IC_SS_SCL_HCNT => i2c.ssclk_hcnt as u32,
            IC_SS_SCL_LCNT => i2c.ssclk_lcnt as u32,
            IC_FS_SCL_HCNT => i2c.fsclk_hcnt as u32,
            IC_FS_SCL_LCNT => i2c.fsclk_lcnt as u32,
            IC_INTR_STAT => i2c.interrupt() as u32,
            IC_INTR_MASK => i2c.interrupt_mask as u32,
            IC_RAW_INTR_STAT => i2c.interrupt_raw as u32,
            IC_RX_TL => 0x38,              // I2C Receive FIFO Threshold Register
            IC_TX_TL => 0x3C,              // I2C Transmit FIFO Threshold Register
            IC_CLR_INTR => 0x40,           // Clear Combined and Individual Interrupt Register
            IC_CLR_RX_UNDER => 0x44,       // Clear RX_UNDER Interrupt Register
            IC_CLR_RX_OVER => 0x48,        // Clear RX_OVER Interrupt Register
            IC_CLR_TX_OVER => 0x4C,        // Clear TX_OVER Interrupt Register
            IC_CLR_RD_REQ => 0x50,         // Clear RD_REQ Interrupt Register
            IC_CLR_TX_ABRT => 0x54,        // Clear TX_ABRT Interrupt Register
            IC_CLR_RX_DONE => 0x58,        // Clear RX_DONE Interrupt Register
            IC_CLR_ACTIVITY => 0x5C,       // Clear ACTIVITY Interrupt Register
            IC_CLR_STOP_DET => 0x60,       // Clear STOP_DET Interrupt Register
            IC_CLR_START_DET => 0x64,      // Clear START_DET Interrupt Register
            IC_CLR_GEN_CALL => 0x68,       // Clear GEN_CALL Interrupt Register
            IC_ENABLE => 0x6C,             // I2C ENABLE Register
            IC_STATUS => 0x70,             // I2C STATUS Register
            IC_TXFLR => 0x74,              // I2C Transmit FIFO Level Register
            IC_RXFLR => 0x78,              // I2C Receive FIFO Level Register
            IC_SDA_HOLD => 0x7C,           // I2C SDA Hold Time Length Register
            IC_TX_ABRT_SOURCE => 0x80,     // I2C Transmit Abort Source Register
            IC_SLV_DATA_NACK_ONLY => 0x84, // Generate Slave Data NACK Register
            IC_DMA_CR => 0x88,             // DMA Control Register
            IC_DMA_TDLR => i2c.transmit_data_level as u32,
            IC_DMA_RDLR => i2c.receive_data_level as u32,
            IC_SDA_SETUP => 0x94, // I2C SDA Setup Register
            IC_ACK_GENERAL_CALL => i2c.ack_general_call as u32,
            IC_ENABLE_STATUS => 0x9C, // I2C Enable Status Register
            IC_FS_SPKLEN => i2c.ic_fs_spklen as u32,
            IC_CLR_RESTART_DET => 0xA8, // Clear RESTART_DET Interrupt Register
            IC_COMP_PARAM_1 => 0,
            IC_COMP_VERSION => 0x3230312a,
            IC_COMP_TYPE => 0x44570140,
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
        let mut i2c = self.borrow_mut();
        match address {
            IC_CON => {}
            IC_TAR => {}
            IC_SAR => {}
            IC_DATA_CMD => {}
            IC_SS_SCL_HCNT => i2c.ssclk_hcnt = value as u16,
            IC_SS_SCL_LCNT => i2c.ssclk_lcnt = value as u16,
            IC_FS_SCL_HCNT => i2c.fsclk_hcnt = value as u16,
            IC_FS_SCL_LCNT => i2c.fsclk_lcnt = value as u16,
            IC_INTR_MASK => i2c.interrupt_mask = value,
            IC_RX_TL => {}
            IC_TX_TL => {}
            IC_ENABLE => {
                let value = value & 0b111;
                i2c.ic_enable = value as u8;
                // TODO
            }
            IC_SDA_HOLD => {}
            IC_SLV_DATA_NACK_ONLY => {
                if i2c.is_enabled() && !i2c.is_slave_active() {
                    i2c.generate_nack = (value & 1) == 1;
                }
            }
            IC_DMA_CR => i2c.dma_ctrl = (value & 0b11) as u8,
            IC_DMA_TDLR => i2c.transmit_data_level = value as u8,
            IC_DMA_RDLR => i2c.receive_data_level = value as u8,
            IC_SDA_SETUP => {}
            IC_ACK_GENERAL_CALL => i2c.ack_general_call = (value & 1) == 1,
            IC_FS_SPKLEN => {
                if i2c.is_enabled() {
                    i2c.ic_fs_spklen = value as u8
                }
            }
            IC_INTR_STAT | IC_RAW_INTR_STAT | IC_CLR_INTR | IC_CLR_RX_UNDER | IC_CLR_RX_OVER
            | IC_CLR_TX_OVER | IC_CLR_RD_REQ | IC_CLR_TX_ABRT | IC_CLR_RX_DONE
            | IC_CLR_ACTIVITY | IC_CLR_STOP_DET | IC_CLR_START_DET | IC_CLR_GEN_CALL
            | IC_STATUS | IC_TXFLR | IC_RXFLR | IC_TX_ABRT_SOURCE | IC_ENABLE_STATUS
            | IC_CLR_RESTART_DET | IC_COMP_PARAM_1 | IC_COMP_VERSION | IC_COMP_TYPE => { /* Read-only registers */
            }
            _ => return Err(PeripheralError::OutOfBounds),
        }

        Ok(())
    }
}
