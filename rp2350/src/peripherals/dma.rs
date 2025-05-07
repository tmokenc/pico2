/**
 * @file peripherals/dma.rs
 * @author Nguyen Le Duy
 * @date 22/04/2025
 * @brief DMA peripheral implementation
 * @todo test against the real hardware
 */
use self::channel::TransferMode;

use super::*;
use crate::bus::{Bus, BusAccessContext, LoadStatus, StoreStatus};
use crate::clock::EventType;
use crate::interrupts::{Interrupt, Interrupts};
use crate::utils::{clear_bits, w1c, Fifo};
use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

mod channel;
mod timer;

use channel::{Channel, TreqSel};
use timer::Timer;

const NOF_CHANNEL: usize = 16;

// Channel 0 at offset 0, up to 0x3fc for channel 15
pub const CHN_READ_ADDR: u16 = 0x000;
pub const CHN_WRITE_ADDR: u16 = 0x004;
pub const CHN_TRANSFER_COUNT: u16 = 0x008;
pub const CHN_CTRL_TRIG: u16 = 0x00C;
pub const CHN_AL1_CTRL: u16 = 0x010;
pub const CHN_AL1_READ_ADDR: u16 = 0x014;
pub const CHN_AL1_WRITE_ADDR: u16 = 0x018;
pub const CHN_AL1_TRANSFER_COUNT_TRIG: u16 = 0x01C;
pub const CHN_AL2_CTRL: u16 = 0x020;
pub const CHN_AL2_TRANS_COUNT: u16 = 0x24;
pub const CHN_AL2_READ_ADDR: u16 = 0x28;
pub const CHN_AL2_WRITE_ADDR_TRIG: u16 = 0x2c;
pub const CHN_AL3_CTRL: u16 = 0x030;
pub const CHN_AL3_WRITE_ADDR: u16 = 0x034;
pub const CHN_AL3_TRANS_COUNT: u16 = 0x038;
pub const CHN_AL3_READ_ADDR_TRIG: u16 = 0x03c;

pub const INTR: u16 = 0x400;

// 4 interrupt channels, the N at the last is the index
pub const INTEN: u16 = 0x404;
pub const INTFN: u16 = 0x408;
pub const INTSN: u16 = 0x40c;

pub const TIMERN: u16 = 0x440;

pub const MULTI_CHAN_TRIGGER: u16 = 0x450;
pub const SNIFF_CTRL: u16 = 0x454;
pub const SNIFF_DATA: u16 = 0x458;
pub const FIFO_LEVELS: u16 = 0x460;
pub const CHAN_ABORT: u16 = 0x464;
pub const N_CHANNELS: u16 = 0x468;

// 16 channels
pub const SECCFG_CHN: u16 = 0x480;

// 4 interrupt channels
pub const SECCFG_IRQN: u16 = 0x4c0;

pub const SECCFG_MISC: u16 = 0x4d0;

pub const MPU_CTRL: u16 = 0x500;

// 8 regions
pub const MPU_BARN: u16 = 0x504;
pub const MPU_LARN: u16 = 0x508;

// 16 channels
pub const CHN_DBG_CTDREQ: u16 = 0x800;
pub const CHN_DBG_TCR: u16 = 0x804;

// Offset between each register
pub const CHANNEL_REGISTER_OFFSET: u16 = 0x040;
pub const INT_REGISTER_OFFSET: u16 = 0x010;
pub const SECCFG_REGISTER_OFFSET: u16 = 0x004;
pub const MPU_REGISTER_OFFSET: u16 = 0x008;
pub const TIMER_REGISTER_OFFSET: u16 = 0x004;

#[derive(Default, Clone)]
pub struct FifoValue {
    pub value: Rc<RefCell<LoadStatus>>,
    pub channel: usize,
}

pub struct Dma {
    pub channels: [Channel; NOF_CHANNEL],
    pub timers: [Timer; 4],
    pub dreg: [bool; 54],
    pub interrupt_raw: u16,
    pub interrupt_enable: [u16; 4],
    pub interrupt_force: [u16; 4],
    pub interrupt_secure: [u8; 4],
    pub seccfg: u16,
    pub fifo: Fifo<FifoValue, NOF_CHANNEL>,
    pub channel_round_robin: Fifo<usize, NOF_CHANNEL>,
    current_read: Option<Rc<RefCell<LoadStatus>>>,
    current_write: Option<Rc<RefCell<StoreStatus>>>,
}

impl Default for Dma {
    fn default() -> Self {
        Self {
            channels: [
                // no copy, cannot do the [Channel::default; 16]...
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
                Channel::default(),
            ],
            timers: [Timer::default(); 4],
            dreg: [false; 54],
            interrupt_raw: 0,
            interrupt_enable: [0; 4],
            interrupt_force: [0; 4],
            interrupt_secure: [0; 4],
            seccfg: 0,
            fifo: Fifo::default(),
            current_read: None,
            current_write: None,
            channel_round_robin: Fifo::default(),
        }
    }
}

impl Dma {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn tick(&mut self, bus: &mut Bus) {
        // no active channels
        if self.channel_round_robin.is_empty() {
            return;
        }

        self.read(bus);
        self.write(bus);
    }

    fn read(&mut self, bus: &mut Bus) {
        if self
            .current_read
            .as_ref()
            .filter(|v| v.borrow().is_done())
            .is_none()
        {
            return;
        }

        self.current_read = None;

        let mut channel_idx = None;
        for _ in 0..self.fifo.len() {
            let Some(idx) = self.channel_round_robin.pop() else {
                return;
            };

            if self.channels[idx].is_enabled() && self.channels[idx].busy() {
                self.channel_round_robin.push(idx);

                if *self.channels[idx].ready_to_transfer.borrow() {
                    channel_idx = Some(idx);
                    break;
                }
            }
        }

        let Some(channel_idx) = channel_idx else {
            return;
        };

        let ref mut channel = self.channels[channel_idx];

        if channel.transfer_count == 0 {
            return;
        }

        let load_status = bus.load(
            channel.read_addr,
            BusAccessContext {
                secure: channel.secure != 0,
                requestor: Requestor::DmaR,
                size: channel.datasize(),
                signed: false,
                exclusive: false,
                architecture: ArchitectureType::Hazard3,
            },
        );

        match load_status {
            Ok(status) => {
                self.fifo.push(FifoValue {
                    value: Rc::clone(&status),
                    channel: channel_idx,
                });

                self.current_read = Some(status);
                channel.update_read_address();
            }

            Err(why) => {
                // TODO
            }
        }
    }

    fn write(&mut self, bus: &mut Bus) {
        if self
            .current_write
            .as_ref()
            .filter(|v| v.borrow().is_done())
            .is_none()
        {
            return;
        }

        self.current_write = None;

        let Some(fifo_value) = self.fifo.pop() else {
            return;
        };

        let ref mut channel = self.channels[fifo_value.channel];

        if !channel.is_enabled() || !channel.busy() {
            return;
        }

        let data_size = channel.datasize();
        let Some(mut value) = fifo_value.value.borrow_mut().value() else {
            return;
        };

        if channel.bswap() {
            match data_size {
                DataSize::Byte => {}
                DataSize::HalfWord => {
                    let tmp = value as u16;
                    value = tmp.swap_bytes() as u32;
                }
                DataSize::Word => value = value.swap_bytes(),
            }
        }

        let store_status = bus.store(
            channel.write_addr,
            value,
            BusAccessContext {
                secure: channel.secure != 0,
                requestor: Requestor::DmaW,
                size: channel.datasize(),
                signed: false,
                exclusive: false,
                architecture: ArchitectureType::Hazard3,
            },
        );

        match store_status {
            Ok(status) => {
                self.current_write = Some(status);
                channel.update_write_address();

                if channel.transfer_mode() == TransferMode::Endless {
                    return;
                }

                if channel.transfer_count == 0 {
                    channel.set_busy(false);
                    let chain_to = channel.chain_to() as usize;
                    let clock = Rc::clone(&bus.peripherals.clock);
                    let transfer_mode = channel.transfer_mode();

                    if !channel.irq_quiet() {
                        self.interrupt_raw |= 1 << fifo_value.channel;
                        self.update_irq(bus.peripherals.interrupts.borrow_mut().deref_mut());
                    }

                    if chain_to != fifo_value.channel {
                        self.start_channel(chain_to, Rc::clone(&clock));
                    }

                    if transfer_mode == TransferMode::TriggerSelf {
                        self.start_channel(fifo_value.channel, clock);
                    }
                } else {
                    self.schedule_transfer(fifo_value.channel, Rc::clone(&bus.peripherals.clock));
                }
            }

            Err(why) => {
                // TODO
            }
        }
    }

    fn start_channel(&mut self, channel_idx: usize, clock: Rc<Clock>) {
        let ref mut channel = self.channels[channel_idx];

        if !channel.is_enabled() || channel.busy() {
            return;
        }

        channel.set_busy(true);
        channel.transfer_count = channel.transfer_counter_reload;
        self.channel_round_robin.push(channel_idx);

        if channel.transfer_count > 0 {
            self.schedule_transfer(channel_idx, Rc::clone(&clock));
        }
    }

    fn abort_channel(&mut self, channel_idx: usize) {
        self.channels[channel_idx].set_busy(false);
    }

    fn schedule_transfer(&mut self, channel_idx: usize, clock: Rc<Clock>) {
        let treq = self.channels[channel_idx].treq_sel();
        let mut delay = 0;

        if !self.has_dreg(treq) && treq != TreqSel::Permanent {
            delay = self.get_timer(treq);

            if delay == 0 {
                return;
            }
        }

        let ready_trigger = Rc::clone(&self.channels[channel_idx].ready_to_transfer);
        *ready_trigger.borrow_mut() = false;

        clock.schedule(delay, EventType::DmaChannelTimer(channel_idx), move || {
            *ready_trigger.borrow_mut() = true;
        });
    }

    pub(crate) fn set_dreg(&mut self, dreg_channel: usize, clock: Rc<Clock>) {
        if self.dreg[dreg_channel] {
            return;
        }

        self.dreg[dreg_channel] = true;

        for i in 0..NOF_CHANNEL {
            let ref mut channel = self.channels[i];
            if channel.treq_sel() == TreqSel::Dreg(dreg_channel as u32) {
                if channel.is_enabled() && channel.busy() {
                    self.schedule_transfer(i, Rc::clone(&clock));
                }
            }
        }
    }

    fn has_dreg(&self, treq_sel: TreqSel) -> bool {
        match treq_sel {
            TreqSel::Dreg(val) => self.dreg[val as usize],
            TreqSel::Timer0 | TreqSel::Timer1 | TreqSel::Timer2 | TreqSel::Timer3 => false,
            _ => false,
        }
    }

    fn get_timer(&mut self, treq_sel: TreqSel) -> u64 {
        match treq_sel {
            TreqSel::Timer0 => self.timers[0].ticks(),
            TreqSel::Timer1 => self.timers[1].ticks(),
            TreqSel::Timer2 => self.timers[2].ticks(),
            TreqSel::Timer3 => self.timers[3].ticks(),
            TreqSel::Permanent => 0,
            _ => 0,
        }
    }

    fn timer_has_access(&self, timer_index: usize) -> bool {
        ((self.seccfg >> (2 + timer_index * 2)) & 0b11) != 0
    }

    fn irq_status(&self, idx: usize) -> u16 {
        (self.interrupt_enable[idx] & self.interrupt_raw) | self.interrupt_force[idx]
    }

    fn update_irq(&self, interrupts: &mut Interrupts) {
        const IRQS: [Interrupt; 4] = [
            Interrupts::DMA_IRQ_0,
            Interrupts::DMA_IRQ_1,
            Interrupts::DMA_IRQ_2,
            Interrupts::DMA_IRQ_3,
        ];

        for i in 0..4 {
            let irq = self.irq_status(i);
            interrupts.set_irq(IRQS[i], irq != 0);
        }
    }
}

impl Peripheral for Rc<RefCell<Dma>> {
    fn read(&self, addr: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let dma = self.borrow();

        let value = match parse_offset(addr) {
            DmaOffset::Channel { index, offset } => {
                let ref channel = dma.channels[index];

                match offset {
                    CHN_READ_ADDR
                    | CHN_AL1_READ_ADDR
                    | CHN_AL2_READ_ADDR
                    | CHN_AL3_READ_ADDR_TRIG => channel.read_addr,

                    CHN_WRITE_ADDR
                    | CHN_AL1_WRITE_ADDR
                    | CHN_AL2_WRITE_ADDR_TRIG
                    | CHN_AL3_WRITE_ADDR => channel.write_addr,

                    CHN_TRANSFER_COUNT
                    | CHN_AL1_TRANSFER_COUNT_TRIG
                    | CHN_AL2_TRANS_COUNT
                    | CHN_AL3_TRANS_COUNT => channel.transfer_count,

                    CHN_CTRL_TRIG | CHN_AL1_CTRL | CHN_AL2_CTRL | CHN_AL3_CTRL => channel.ctrl,

                    SECCFG_CHN => channel.secure as u32,

                    CHN_DBG_CTDREQ => channel.dreq_counter,
                    CHN_DBG_TCR => channel.transfer_counter_reload,

                    _ => return Err(PeripheralError::OutOfBounds),
                }
            }

            DmaOffset::Interrupt { index, offset } => match offset {
                INTEN => dma.interrupt_enable[index] as u32,
                INTFN => dma.interrupt_force[index] as u32,
                INTSN => dma.irq_status(index) as u32,
                SECCFG_IRQN => dma.interrupt_secure[index] as u32,
                _ => return Err(PeripheralError::OutOfBounds),
            },

            DmaOffset::Timer { index } => {
                if !ctx.secure && dma.timer_has_access(index) {
                    return Err(PeripheralError::OutOfBounds);
                }

                let timer = dma.timers[index];
                timer.into()
            }

            DmaOffset::Mpu { .. } => {
                /* TODO */
                0
            }

            DmaOffset::Default => match addr {
                INTR => dma.interrupt_raw as u32,
                MULTI_CHAN_TRIGGER => 0,
                SNIFF_CTRL => 0, // not implemented
                SNIFF_DATA => 0,
                FIFO_LEVELS => 0, // TODO
                CHAN_ABORT => 0,
                N_CHANNELS => NOF_CHANNEL as u32,
                SECCFG_MISC => dma.seccfg as u32,
                MPU_CTRL => 0, // TODO
                _ => return Err(PeripheralError::OutOfBounds),
            },
        };

        Ok(value)
    }

    fn write_raw(
        &mut self,
        addr: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        let mut dma = self.borrow_mut();

        match parse_offset(addr) {
            DmaOffset::Channel { index, offset } => {
                let ref mut channel = dma.channels[index];

                match offset {
                    CHN_READ_ADDR
                    | CHN_AL1_READ_ADDR
                    | CHN_AL2_READ_ADDR
                    | CHN_AL3_READ_ADDR_TRIG => channel.read_addr = value,

                    CHN_WRITE_ADDR
                    | CHN_AL1_WRITE_ADDR
                    | CHN_AL2_WRITE_ADDR_TRIG
                    | CHN_AL3_WRITE_ADDR => channel.write_addr = value,

                    CHN_TRANSFER_COUNT
                    | CHN_AL1_TRANSFER_COUNT_TRIG
                    | CHN_AL2_TRANS_COUNT
                    | CHN_AL3_TRANS_COUNT => {
                        if !matches!(value >> 28, 0x0 | 0x1 | 0xf) {
                            return Err(PeripheralError::Reserved);
                        }

                        channel.transfer_counter_reload = value;
                    }

                    CHN_CTRL_TRIG  // dont fmt it
                    | CHN_AL1_CTRL
                    | CHN_AL2_CTRL
                    | CHN_AL3_CTRL => {
                        let ref mut ctrl = channel.ctrl;
                        w1c(ctrl, value, 0b11 << 29);

                        let rw_mask = 0b111111 << 26;
                        clear_bits(ctrl, !rw_mask);
                        *ctrl |= value & rw_mask;

                        if channel.is_enabled(){
                            if channel.busy() {
                                dma.schedule_transfer(index, Rc::clone(&ctx.clock));
                            }
                        } else {
                            channel.set_busy(false);
                            // TODO
                        }
                    }

                    SECCFG_CHN => channel.secure = (value & 0b111) as u8,

                    CHN_DBG_CTDREQ => channel.dreq_counter = 0,
                    CHN_DBG_TCR => { /* Read Only */ }

                    _ => return Err(PeripheralError::OutOfBounds),
                }

                if matches!(
                    offset,
                    CHN_CTRL_TRIG
                        | CHN_AL1_TRANSFER_COUNT_TRIG
                        | CHN_AL2_WRITE_ADDR_TRIG
                        | CHN_AL3_READ_ADDR_TRIG
                ) {
                    if value > 0 {
                        dma.start_channel(index, Rc::clone(&ctx.clock));
                    } else {
                        // Null trigger
                        dma.interrupt_raw |= 1 << index;
                        dma.update_irq(ctx.interrupts.borrow_mut().deref_mut());
                    }
                }
            }

            DmaOffset::Interrupt { index, offset } => {
                match offset {
                    INTEN => dma.interrupt_enable[index] = value as u16,
                    INTFN => dma.interrupt_force[index] = value as u16,
                    INTSN => {
                        let mut int = dma.interrupt_raw as u32;
                        w1c(&mut int, value, 0xFFFF);
                        dma.interrupt_raw = int as u16;
                    }

                    SECCFG_IRQN => dma.interrupt_secure[index] = (value & 0b11) as u8,
                    _ => return Err(PeripheralError::OutOfBounds),
                }

                let mut irq = ctx.interrupts.borrow_mut();
                dma.update_irq(irq.deref_mut());
            }

            DmaOffset::Timer { index } => {
                if !ctx.secure && dma.timer_has_access(index) {
                    return Err(PeripheralError::OutOfBounds);
                }

                dma.timers[index] = value.into();
            }

            DmaOffset::Mpu { .. } => { /* TODO */ }

            DmaOffset::Default => match addr {
                INTR => {
                    dma.interrupt_raw = value as u16; // TODO
                    let mut irq = ctx.interrupts.borrow_mut();
                    dma.update_irq(irq.deref_mut());
                }
                MULTI_CHAN_TRIGGER => {
                    for index in 0..NOF_CHANNEL {
                        if (value & (1 << index)) != 0 {
                            dma.start_channel(index, Rc::clone(&ctx.clock));
                        }
                    }
                }
                SNIFF_CTRL | SNIFF_DATA => { /* Not implemented */ }
                FIFO_LEVELS => { /* read only */ }
                CHAN_ABORT => {
                    for index in 0..NOF_CHANNEL {
                        if (value & (1 << index)) != 0 {
                            dma.abort_channel(index);
                        }
                    }
                }
                N_CHANNELS => { /* Read only */ }
                SECCFG_MISC => dma.seccfg = (value & 0b1_1111_1111) as u16,
                MPU_CTRL => {} // TODO
                _ => return Err(PeripheralError::OutOfBounds),
            },
        }

        Ok(())
    }
}

// A helper enum to parse the offset
enum DmaOffset {
    Channel { index: usize, offset: u16 },
    Interrupt { index: usize, offset: u16 },
    Timer { index: usize },
    Mpu { index: usize, offset: u16 },
    Default,
}

// Return the channel index and the base offset to be read
fn parse_offset(offset: u16) -> DmaOffset {
    match offset {
        CHN_READ_ADDR..=0x3fc => DmaOffset::Channel {
            index: (offset / CHANNEL_REGISTER_OFFSET) as usize,
            offset: offset & 0b11111,
        },

        CHN_DBG_CTDREQ..=0xbc4 => DmaOffset::Channel {
            index: ((offset - CHN_DBG_CTDREQ) / CHANNEL_REGISTER_OFFSET) as usize,
            offset: (offset & 0b11111) + CHN_DBG_CTDREQ,
        },

        INTEN..=0x43c => DmaOffset::Interrupt {
            index: ((offset - INTEN) / INT_REGISTER_OFFSET) as usize,
            offset: (offset & 0b111) + INTEN,
        },

        TIMERN..=0x44c => DmaOffset::Timer {
            index: ((offset - TIMERN) / TIMER_REGISTER_OFFSET) as usize,
        },

        SECCFG_CHN..=0x4bc => DmaOffset::Channel {
            index: ((offset - SECCFG_CHN) / SECCFG_REGISTER_OFFSET) as usize,
            offset: (offset & 0b11) + SECCFG_CHN,
        },

        SECCFG_IRQN..=0x4cc => DmaOffset::Interrupt {
            index: ((offset - SECCFG_IRQN) / SECCFG_REGISTER_OFFSET) as usize,
            offset: (offset & 0b11) + SECCFG_IRQN,
        },

        MPU_BARN..=0x540 => DmaOffset::Mpu {
            index: ((offset - MPU_BARN) / MPU_REGISTER_OFFSET) as usize,
            offset: (offset & 0b111) + MPU_BARN,
        },

        _ => DmaOffset::Default,
    }
}
