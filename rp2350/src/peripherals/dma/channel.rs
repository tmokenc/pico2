use std::cell::RefCell;
use std::rc::Rc;

use crate::common::DataSize;
use crate::utils::{extract_bit, extract_bits, set_bit_state};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreqSel {
    Timer0,
    Timer1,
    Timer2,
    Timer3,
    Permanent,
    Dreg(u32),
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum TransferMode {
    #[default]
    Normal,
    TriggerSelf,
    Endless,
}

#[derive(Default, Clone)]
pub struct Channel {
    pub read_addr: u32,
    pub write_addr: u32,
    pub transfer_count: u32,
    pub ctrl: u32,
    pub dreq_counter: u32,
    pub secure: u8,
    pub transfer_counter_reload: u32,
    pub ready_to_transfer: Rc<RefCell<bool>>,
}

impl Channel {
    pub fn transfer_mode(&self) -> TransferMode {
        match self.transfer_count >> 28 {
            0x0 => TransferMode::Normal,
            0x1 => TransferMode::TriggerSelf,
            0xf => TransferMode::Endless,
            _ => unreachable!(),
        }
    }
    pub fn is_enabled(&self) -> bool {
        extract_bit(self.ctrl, 0) != 0
    }

    pub fn high_priority(&self) -> bool {
        extract_bit(self.ctrl, 1) != 0
    }

    pub fn datasize(&self) -> DataSize {
        match extract_bits(self.ctrl, 2..=3) {
            0 => DataSize::Byte,
            1 => DataSize::HalfWord,
            _ => DataSize::Word,
        }
    }

    pub fn incr_read(&self) -> bool {
        extract_bit(self.ctrl, 4) != 0
    }

    pub fn incr_read_rev(&self) -> bool {
        extract_bit(self.ctrl, 5) != 0
    }

    pub fn incr_write(&self) -> bool {
        extract_bit(self.ctrl, 6) != 0
    }

    pub fn incr_write_rev(&self) -> bool {
        extract_bit(self.ctrl, 7) != 0
    }

    pub fn ring_size(&self) -> u32 {
        extract_bits(self.ctrl, 8..=11)
    }

    pub fn ring_sel(&self) -> bool {
        extract_bit(self.ctrl, 12) != 0
    }

    pub fn chain_to(&self) -> u32 {
        extract_bits(self.ctrl, 13..=16)
    }

    pub fn treq_sel(&self) -> TreqSel {
        match extract_bits(self.ctrl, 17..=22) {
            0x3b => TreqSel::Timer0,
            0x3c => TreqSel::Timer1,
            0x3d => TreqSel::Timer2,
            0x3e => TreqSel::Timer3,
            0x3f => TreqSel::Permanent,
            v => TreqSel::Dreg(v),
        }
    }

    pub fn irq_quiet(&self) -> bool {
        extract_bit(self.ctrl, 23) != 0
    }

    pub fn bswap(&self) -> bool {
        extract_bit(self.ctrl, 24) != 0
    }

    pub fn sniff_en(&self) -> bool {
        extract_bit(self.ctrl, 25) != 0
    }

    pub fn busy(&self) -> bool {
        extract_bit(self.ctrl, 26) != 0
    }

    pub fn set_busy(&mut self, busy: bool) {
        set_bit_state(&mut self.ctrl, 26, busy);
    }

    pub fn write_err(&self) -> bool {
        extract_bit(self.ctrl, 29) != 0
    }

    pub fn read_err(&self) -> bool {
        extract_bit(self.ctrl, 30) != 0
    }

    pub fn ahb_err(&self) -> bool {
        extract_bit(self.ctrl, 31) != 0
    }

    fn ring_mask(&self) -> u32 {
        let ring_size = self.ring_size();

        if ring_size == 0 {
            return 0;
        }

        (1 << ring_size) - 1
    }

    fn addr_wrap(&self, base_addr: u32, new_addr: u32) -> u32 {
        let ring_mask = self.ring_mask();

        if ring_mask == 0 {
            return new_addr;
        }

        let addr = new_addr & ring_mask;
        let upper_addr = base_addr & !ring_mask;

        upper_addr | addr
    }

    pub fn update_read_address(&mut self) {
        let data_size = self.datasize() as u32;

        let mut addr = match (self.incr_read(), self.incr_read_rev()) {
            (true, true) => self.read_addr.wrapping_sub(data_size),
            (true, false) => self.read_addr.wrapping_add(data_size),
            (false, true) => self.read_addr.wrapping_add(data_size * 2),
            (false, false) => self.read_addr,
        };

        // read are wrapped on ring_self == 0
        if !self.ring_sel() {
            addr = self.addr_wrap(self.read_addr, addr);
        }

        self.read_addr = addr;
    }

    pub fn update_write_address(&mut self) {
        let data_size = self.datasize() as u32;

        let mut addr = match (self.incr_write(), self.incr_write_rev()) {
            (true, true) => self.write_addr.wrapping_sub(data_size),
            (true, false) => self.write_addr.wrapping_add(data_size),
            (false, true) => self.write_addr.wrapping_add(data_size * 2),
            (false, false) => self.write_addr,
        };

        // write are wrapped on ring_self == 1
        if self.ring_sel() {
            addr = self.addr_wrap(self.write_addr, addr);
        }

        self.write_addr = addr;
    }
}
