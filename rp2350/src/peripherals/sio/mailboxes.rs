use crate::common::Requestor;
use crate::utils::Fifo;

#[derive(Default)]
pub struct Mailboxes {
    data: [Fifo<u32, 8>; 2],
    roe: [bool; 2], // (Sticky) Read On Empty Error
    wof: [bool; 2], // (Sticky) Write On Full Error
}

impl Mailboxes {
    pub(super) fn state(&self, requestor: Requestor) -> u32 {
        let index = match requestor {
            Requestor::Proc0 => 0,
            Requestor::Proc1 => 1,
            _ => 0,
        };

        let vld = self.data[index].is_empty() as u32;
        let rdy = self.data[index].is_full() as u32;
        let roe = self.roe[index] as u32;
        let wof = self.wof[index] as u32;

        vld | (rdy << 1) | (wof << 2) | (roe << 3)
    }

    /// Core 0 can see the read side of the 1→0 FIFO (RX), and the write side of 0→1 FIFO (TX).
    pub(super) fn read(&mut self, requestor: Requestor) -> u32 {
        let (index, roe_index) = match requestor {
            Requestor::Proc0 => (1, 0),
            Requestor::Proc1 => (0, 1),
            _ => (0, 0),
        };

        match self.data[index].pop() {
            Some(value) => value,
            None => {
                self.roe[roe_index] = true;
                0
            }
        }
    }

    /// Core 1 can see the read side of the 0→1 FIFO (RX), and the write side of 1→0 FIFO (TX).
    pub(super) fn write(&mut self, value: u32, requestor: Requestor) {
        let index = match requestor {
            Requestor::Proc0 => 0,
            Requestor::Proc1 => 1,
            _ => 0,
        };

        if self.data[index].push(value).is_err() {
            self.wof[index] = true;
        }
    }

    pub(super) fn clear_roe(&mut self, requestor: Requestor) {
        match requestor {
            Requestor::Proc0 => self.roe[0] = false,
            Requestor::Proc1 => self.roe[1] = false,
            _ => {}
        }
    }

    pub(super) fn clear_wof(&mut self, requestor: Requestor) {
        match requestor {
            Requestor::Proc0 => self.wof[0] = false,
            Requestor::Proc1 => self.wof[1] = false,
            _ => {}
        }
    }
}
