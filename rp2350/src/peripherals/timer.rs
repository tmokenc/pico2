/**
 * @file peripherals/timer.rs
 * @author Nguyen Le Duy
 * @date 02/05/2025
 * @brief Timer peripheral implementation
 */
use crate::clock::{EventType, Ticks};
use crate::interrupts::Interrupt;
use crate::utils::extract_bit;

use super::*;
use std::cell::RefCell;
use std::rc::Rc;

pub const TIMEHW: u16 = 0x00; // Write to bits 63:32 of time
pub const TIMELW: u16 = 0x04; // Write to bits 31:0 of time
pub const TIMEHR: u16 = 0x08; // Read from bits 63:32 of time
pub const TIMELR: u16 = 0x0C; // Read from bits 31:0 of time
pub const ALARM0: u16 = 0x10; // Arm alarm 0, and configure the time it will fire
pub const ALARM1: u16 = 0x14; // Arm alarm 1, and configure the time it will fire
pub const ALARM2: u16 = 0x18; // Arm alarm 2, and configure the time it will fire
pub const ALARM3: u16 = 0x1C; // Arm alarm 3, and configure the time it will fire
pub const ARMED: u16 = 0x20; // Indicates the armed/disarmed status of each alarm
pub const TIMERAWH: u16 = 0x24; // Raw read from bits 63:32 of time (no side effects)
pub const TIMERAWL: u16 = 0x28; // Raw read from bits 31:0 of time (no side effects)
pub const DBGPAUSE: u16 = 0x2C; // Set bits high to enable pause when the corresponding debug ports are active
pub const PAUSE: u16 = 0x30; // Set high to pause the timer
pub const LOCKED: u16 = 0x34; // Set locked bit to disable write access to timer
pub const SOURCE: u16 = 0x38; // Selects the source for the timer
pub const INTR: u16 = 0x3C; // Raw Interrupts
pub const INTE: u16 = 0x40; // Interrupt Enable
pub const INTF: u16 = 0x44; // Interrupt Force
pub const INTS: u16 = 0x48; // Interrupt status after masking & forcing

#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub enum CountSource {
    #[default]
    _1MHz,
    ClkSys,
}

impl From<CountSource> for u32 {
    fn from(source: CountSource) -> Self {
        match source {
            CountSource::_1MHz => 0,
            CountSource::ClkSys => 1,
        }
    }
}

impl From<u32> for CountSource {
    fn from(source: u32) -> Self {
        match source {
            0 => CountSource::_1MHz,
            1 => CountSource::ClkSys,
            _ => panic!("Invalid CountSource value"),
        }
    }
}

#[derive(Default)]
pub struct Alarm {
    pub time: u32,
    pub armed: bool,
    pub interrupting: bool,
}

#[derive(Default)]
pub struct Timer<const IDX: usize> {
    pub counter: u64,
    pub alarm: [Alarm; 4],
    pub interrup_mask: u8,
    pub interrupt_force: u8,
    pub is_paused: bool,
    pub is_locked: bool,
    pub source: CountSource,
}

impl<const IDX: usize> Timer<IDX> {
    fn interrupt_raw(&self) -> u8 {
        let mut raw = 0;

        for i in 0..4 {
            raw |= (self.alarm[i].interrupting as u8) << i;
        }

        raw
    }

    fn interrupt_status(&self) -> u8 {
        let mut status = 0;

        for i in 0..4 {
            if self.alarm[i].armed && self.alarm[i].interrupting {
                status |= 1 << i;
            }
        }

        (status | self.interrupt_force) & self.interrup_mask
    }

    fn update_interrupts(&mut self, interrupts: Rc<RefCell<Interrupts>>) {
        let mut interrupts = interrupts.borrow_mut();

        for i in 0..4 {
            let num = self.interrupt_num(i);
            interrupts.set_irq(num, self.alarm[i].interrupting);
        }
    }

    fn interrupt_num(&self, alarm_index: usize) -> Interrupt {
        match (IDX, alarm_index) {
            (0, 0) => Interrupts::TIMER0_IRQ_0,
            (0, 1) => Interrupts::TIMER0_IRQ_1,
            (0, 2) => Interrupts::TIMER0_IRQ_2,
            (0, 3) => Interrupts::TIMER0_IRQ_3,
            (1, 0) => Interrupts::TIMER1_IRQ_0,
            (1, 1) => Interrupts::TIMER1_IRQ_1,
            (1, 2) => Interrupts::TIMER1_IRQ_2,
            (1, 3) => Interrupts::TIMER1_IRQ_3,
            _ => unreachable!(),
        }
    }
}

impl<const IDX: usize> Peripheral for Rc<RefCell<Timer<IDX>>> {
    fn read(&self, address: u16, _ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        let timer = self.borrow();

        let value = match address {
            TIMEHR => (timer.counter >> 32) as u32, // TODO side effects???
            TIMELR => timer.counter as u32,
            ALARM0 => timer.alarm[0].time,
            ALARM1 => timer.alarm[1].time,
            ALARM2 => timer.alarm[2].time,
            ALARM3 => timer.alarm[3].time,
            ARMED => {
                let mut armed = 0;
                for i in 0..4 {
                    if timer.alarm[i].armed {
                        armed |= 1 << i;
                    }
                }
                armed
            }
            TIMERAWH => (timer.counter >> 32) as u32,
            TIMERAWL => timer.counter as u32,
            DBGPAUSE => 0, // TODO not yet implemented debug
            PAUSE => timer.is_paused as u32,
            LOCKED => timer.is_locked as u32,
            SOURCE => timer.source.into(),
            INTR => timer.interrupt_raw() as u32,
            INTE => timer.interrup_mask as u32,
            INTF => timer.interrupt_force as u32,
            INTS => timer.interrupt_status() as u32,

            TIMEHW | TIMELW => {
                0 /* Do nothing, these are write-only */
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
        let mut timer = self.borrow_mut();

        if timer.is_locked {
            // write access is locked
            return Ok(());
        }

        match address {
            TIMEHW => {
                timer.counter = (timer.counter & 0x00000000FFFFFFFF) | ((value as u64) << 32);
            }
            TIMELW => {
                timer.counter = (timer.counter & 0xFFFFFFFF00000000) | (value as u64);
            }
            ALARM0 => {
                timer.alarm[0].time = value;
                timer.alarm[0].armed = true;
            }
            ALARM1 => {
                timer.alarm[1].time = value;
                timer.alarm[1].armed = true;
            }
            ALARM2 => {
                timer.alarm[2].time = value;
                timer.alarm[2].armed = true;
            }
            ALARM3 => {
                timer.alarm[3].time = value;
                timer.alarm[3].armed = true;
            }
            ARMED => {
                for i in 0..4 {
                    timer.alarm[i].armed = extract_bit(value, i as u32) == 1;
                }
            }
            PAUSE => timer.is_paused = extract_bit(value, 0) == 1,
            LOCKED => timer.is_locked = extract_bit(value, 0) == 1,
            SOURCE => {
                timer.source = CountSource::from(value);
                drop(timer);
                reschedule_timer_tick(self.clone(), ctx.clock.clone(), ctx.interrupts.clone());
            }
            INTR => {
                for i in 0..4 {
                    if extract_bit(value, i) == 1 {
                        timer.alarm[i as usize].interrupting = false;
                    }
                }
            }
            INTE => {
                timer.interrup_mask = (value as u8) & 0b1111;
                timer.update_interrupts(ctx.interrupts.clone());
            }
            INTF => {
                timer.interrupt_force = (value as u8) & 0b1111;
                timer.update_interrupts(ctx.interrupts.clone());
            }

            DBGPAUSE => {} // TODO not yet implemented debug
            INTS | TIMERAWH | TIMERAWL | TIMEHR | TIMELR => { /* read only */ }
            _ => return Err(PeripheralError::OutOfBounds),
        };
        Ok(())
    }
}

pub(super) fn start_timer<const IDX: usize>(
    timer: Rc<RefCell<Timer<IDX>>>,
    clock: Rc<Clock>,
    interrupts: Rc<RefCell<Interrupts>>,
) {
    // Schedule the first tick

    let next_tick = match timer.borrow().source {
        CountSource::_1MHz => Ticks::_1MHZ,
        CountSource::ClkSys => Ticks::CKL_SYS,
    };

    let clock_clone = clock.clone();
    clock.schedule(next_tick, EventType::Timer(IDX), move || {
        timer_tick(timer, clock_clone, interrupts)
    });
}

pub fn reschedule_timer_tick<const IDX: usize>(
    timer_ref: Rc<RefCell<Timer<IDX>>>,
    clock: Rc<Clock>,
    interrupts_ref: Rc<RefCell<Interrupts>>,
) {
    if clock.is_scheduled(EventType::Timer(IDX)) {
        // Cancel the scheduled event
        clock.cancel(EventType::Timer(IDX));
    }

    start_timer(timer_ref, clock, interrupts_ref);
}

fn timer_tick<const IDX: usize>(
    timer_ref: Rc<RefCell<Timer<IDX>>>,
    clock: Rc<Clock>,
    interrupts_ref: Rc<RefCell<Interrupts>>,
) {
    let mut timer = timer_ref.borrow_mut();
    if !timer.is_paused {
        timer.counter += 1;
        let counter = timer.counter as u32;

        for alarm in timer.alarm.iter_mut() {
            if alarm.armed && counter == alarm.time {
                alarm.interrupting = true;
            }
        }

        timer.update_interrupts(interrupts_ref.clone());
    }

    // Schedule the next tick
    let next_tick = match timer.source {
        CountSource::_1MHz => Ticks::_1MHZ,
        CountSource::ClkSys => Ticks::CKL_SYS,
    };

    let timer_ref = timer_ref.clone();
    let clock_ref = clock.clone();
    let interrupts_ref = interrupts_ref.clone();

    clock.schedule(next_tick, EventType::Timer(IDX), move || {
        timer_tick(timer_ref, clock_ref, interrupts_ref)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::Clock;
    use crate::interrupts::Interrupts;

    macro_rules! setup {
        ($name:ident, $idx:expr, $clock:ident, $interrupt:ident) => {
            #[allow(unused_mut)]
            let mut $name = Rc::new(RefCell::new(Timer::<$idx>::default()));
            start_timer($name.clone(), $clock.clone(), $interrupt.clone());
        };
    }

    #[test]
    fn test_timer() {
        let clock = Rc::new(Clock::new());
        let interrupts = Rc::new(RefCell::new(Interrupts::default()));
        setup!(timer, 0, clock, interrupts);

        for _ in 0..149 {
            clock.tick();
            let timer = timer.borrow();
            let interrupt = interrupts.borrow();
            assert_eq!(timer.counter, 0);
            assert_eq!(timer.interrupt_raw(), 0);
            assert_eq!(timer.interrupt_status(), 0);
            assert_eq!(interrupt.iter(0).next(), None);
        }

        clock.tick();
        let timer = timer.borrow();
        assert_eq!(timer.counter, 1);
    }

    #[test]
    fn test_timer_concurrent() {
        let clock = Rc::new(Clock::new());
        let interrupts = Rc::new(RefCell::new(Interrupts::default()));
        setup!(timer, 0, clock, interrupts);
        setup!(timer1, 1, clock, interrupts);

        let peri_ctx = PeripheralAccessContext {
            clock: clock.clone(),
            interrupts: interrupts.clone(),
            ..Default::default()
        };

        timer
            .write(SOURCE, 1, &peri_ctx) // use CLK_SYS
            .unwrap();

        assert_eq!(timer.borrow().source, CountSource::ClkSys);

        for i in 1..150 {
            clock.tick();
            let timer = timer.borrow();
            let timer1 = timer1.borrow();
            assert_eq!(timer.counter, i);
            assert_eq!(timer1.counter, 0);
        }

        clock.tick();
        let timer1 = timer1.borrow();
        assert_eq!(timer1.counter, 1);
    }
}
