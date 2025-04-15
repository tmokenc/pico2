use crate::clock::*;
use crate::interrupts::Interrupts;
use crate::utils::{clear_bit, set_bit, set_bit_state};
use std::cell::RefCell;
use std::rc::Rc;

use super::*;

use crate::common::{DataSize, Requestor};

#[derive(Clone, Copy, Debug)]
pub struct Timer {
    pub x: u16,
    pub y: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreqSel {
    Timer0,
    Timer1,
    Timer2,
    Timer3,
    Permanent,
    Dreg(u32),
}

#[derive(Default, Clone, Copy)]
pub struct Channel {
    pub read_addr: u32,
    pub write_addr: u32,
    pub transfer_count: u32,
    pub ctrl: u32,
}

impl Channel {
    fn is_enabled(&self) -> bool {
        (self.ctrl & 0x1) != 0
    }

    fn high_priority(&self) -> bool {
        (self.ctrl & 1 << 1) != 0
    }

    fn datasize(&self) -> DataSize {
        match (self.ctrl >> 2) & 0b11 {
            0 => DataSize::Byte,
            1 => DataSize::HalfWord,
            _ => DataSize::Word,
        }
    }

    fn inc_read(&self) -> bool {
        (self.ctrl & 1 << 4) != 0
    }

    fn inc_read_rev(&self) -> bool {
        (self.ctrl & 1 << 5) != 0
    }

    fn inc_write(&self) -> bool {
        (self.ctrl & 1 << 6) != 0
    }

    fn inc_write_rev(&self) -> bool {
        (self.ctrl & 1 << 7) != 0
    }

    fn ring_size(&self) -> u32 {
        (self.ctrl >> 8) & 0b1111
    }

    fn ring_sel(&self) -> bool {
        (self.ctrl & 1 << 12) != 0
    }

    fn chain_to(&self) -> u32 {
        (self.ctrl >> 13) & 0b1111
    }

    fn treq_sel(&self) -> TreqSel {
        match (self.ctrl >> 17) & 0b111111 {
            0x3b => TreqSel::Timer0,
            0x3c => TreqSel::Timer1,
            0x3d => TreqSel::Timer2,
            0x3e => TreqSel::Timer3,
            0x3f => TreqSel::Permanent,
            v => TreqSel::Dreg(v),
        }
    }

    fn irq_quiet(&self) -> bool {
        (self.ctrl & 1 << 23) != 0
    }

    fn bswap(&self) -> bool {
        (self.ctrl & 1 << 24) != 0
    }

    fn sniff_en(&self) -> bool {
        (self.ctrl & 1 << 25) != 0
    }

    fn busy(&self) -> bool {
        (self.ctrl & 1 << 26) != 0
    }

    fn set_busy(&mut self, busy: bool) {
        set_bit_state(&mut self.ctrl, 26, busy);
    }

    fn write_err(&self) -> bool {
        (self.ctrl & 1 << 29) != 0
    }

    fn read_err(&self) -> bool {
        (self.ctrl & 1 << 30) != 0
    }

    fn ahb_err(&self) -> bool {
        // should be a logical or
        (self.ctrl & 1 << 31) != 0
    }
}

pub struct Dma {
    pub channels: [Channel; 8],
    pub timers: [Timer; 4],
    pub dreg: [bool; 54],
}

impl Default for Dma {
    fn default() -> Self {
        Self {
            channels: [Channel::default(); 8],
            timers: [Timer { x: 0, y: 0 }; 4],
            dreg: [false; 54],
        }
    }
}

impl Dma {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn tick(&mut self) {
        todo!()
    }

    fn start_channel(&mut self, channel_idx: usize) {
        let ref mut channel = self.channels[channel_idx];

        if !channel.is_enabled() || channel.busy() {
            return;
        }

        channel.set_busy(true);

        if channel.transfer_count > 0 {
            self.schedule_transfer(channel_idx);
        }
    }

    fn schedule_transfer(&mut self, channel_idx: usize) {
        let ref mut channel = self.channels[channel_idx];

        let mut delay = 0;
        let treq = channel.treq_sel();

        if !self.has_dreg(treq) && treq != TreqSel::Permanent {
            delay = self.get_timer(treq);
        }

        // self.clock
        //     .schedule(delay, "DMA transfer".to_string(), move |clock| {
        //         let mut channel = &mut clock.channels[channel_idx];

        //         if channel.is_enabled() && channel.busy() {
        //             // perform the transfer
        //             self.perform_transfer(channel);
        //         }

        //         if channel.transfer_count == 0 {
        //             channel.set_busy(false);
        //             self.interrupts.borrow_mut().set_irq(Requestor::Dma, true);
        //         }
        //     });

        todo!()
    }

    fn has_dreg(&self, treq_sel: TreqSel) -> bool {
        match treq_sel {
            TreqSel::Dreg(val) => self.dreg[val as usize],
            TreqSel::Timer0 | TreqSel::Timer1 | TreqSel::Timer2 | TreqSel::Timer3 => false,
            _ => false,
        }
    }

    fn get_timer(&mut self, treq_sel: TreqSel) -> u64 {
        let ref timer = match treq_sel {
            TreqSel::Timer0 => self.timers[0],
            TreqSel::Timer1 => self.timers[1],
            TreqSel::Timer2 => self.timers[2],
            TreqSel::Timer3 => self.timers[3],
            TreqSel::Permanent => return 0,
            _ => return 0,
        };

        if timer.y == 0 {
            return 0;
        }

        let x = timer.x as u64;
        let y = timer.y as u64;

        ((x / y) * MHZ) / (150 * MHZ)
    }
}

impl Peripheral for Dma {
    fn read(&self, addr: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        todo!()
    }

    fn write_raw(
        &mut self,
        addr: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        todo!()
    }
}
