/**
 * @file peripherals/sio/timer.rs
 * @author Nguyen Le Duy
 * @date 31/01/2025
 * @brief Implementation of the RISC-V platform timer
 */
use std::cell::RefCell;
use std::rc::Rc;

use crate::clock::{Clock, EventType, Ticks};
use crate::interrupts::Interrupts;
use crate::peripherals::PeripheralAccessContext;
use crate::utils::extract_bit;

pub struct RiscVPlatformTimer {
    pub ctrl: u8,
    pub counter: u64,
    pub cmp: u64,
}

impl Default for RiscVPlatformTimer {
    fn default() -> Self {
        Self {
            ctrl: 0b1101,
            counter: 0,
            cmp: 0xFFFF_FFFF_FFFF_FFFF,
        }
    }
}

impl RiscVPlatformTimer {
    fn next_tick(&self) -> Ticks {
        match extract_bit(self.ctrl, 1) {
            1 => Ticks::CKL_SYS,
            0 => Ticks::_1MHZ,
            _ => unreachable!(),
        }
    }

    pub fn update_interrupt(&self, interrupts: Rc<RefCell<Interrupts>>) {
        let irq = self.cmp == self.counter;
        interrupts
            .borrow_mut()
            .set_irq(Interrupts::SIO_IRQ_MTIMECMP, irq);
    }
}

pub fn update_timer_ctrl(
    timer_ref: Rc<RefCell<RiscVPlatformTimer>>,
    ctrl: u8,
    ctx: &PeripheralAccessContext,
) {
    let mut timer = timer_ref.borrow_mut();
    let last_ctrl = timer.ctrl;
    timer.ctrl = ctrl;
    drop(timer);

    if extract_bit(ctrl, 0) == 0 {
        ctx.clock.cancel(EventType::RiscVTimer);
    } else {
        schedule_timer(timer_ref.clone(), ctx.clock.clone(), ctx.interrupts.clone());
    }

    // Reconfigurated speed => reschedule timer
    // 1 = full speed
    // 0 = 1 per microsecond
    if extract_bit(ctrl, 1) != extract_bit(last_ctrl, 1) {
        ctx.clock.cancel(EventType::RiscVTimer);
        schedule_timer(timer_ref, ctx.clock.clone(), ctx.interrupts.clone());
    }
}

pub fn schedule_timer(
    timer: Rc<RefCell<RiscVPlatformTimer>>,
    clock: Rc<Clock>,
    interrupt: Rc<RefCell<Interrupts>>,
) {
    if clock.is_scheduled(EventType::RiscVTimer) {
        return;
    }

    let tick = timer.borrow().next_tick();

    let clock_ref = clock.clone();
    let interrupt_ref = interrupt.clone();

    clock.schedule(tick, EventType::RiscVTimer, move || {
        {
            let mut timer = timer.borrow_mut();
            timer.counter += 1;
            timer.update_interrupt(interrupt.clone());
        }

        schedule_timer(timer, clock_ref, interrupt_ref);
    });
}
