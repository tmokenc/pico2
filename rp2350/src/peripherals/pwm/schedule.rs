/**
 * @file peripherals/pwm/schedule.rs
 * @author Nguyen Le Duy
 * @date 09/05/2025
 * @brief Scheduling system for the PWM peripheral
 */
use super::*;
use crate::clock::EventType;
use crate::gpio::GpioController;
use crate::inspector::InspectorRef;
use crate::interrupts::Interrupts;
use std::cell::RefCell;
use std::rc::Rc;

/// GPIO controller may call this function to update the PWM on pin B input signal
/// This should call only on pin change
pub fn channel_b_update(
    pwm_ref: Rc<RefCell<Pwm>>,
    channel_idx: usize,
    pin_state: bool,
    clock_ref: Rc<Clock>,
    gpio_ref: Rc<RefCell<GpioController>>,
    interrupts_ref: Rc<RefCell<Interrupts>>,
    inspector: InspectorRef,
) {
    let mut pwm = pwm_ref.borrow_mut();
    let ref mut channel = pwm.channels[channel_idx];

    if !channel.is_enabled() {
        return;
    }

    match (channel.divmode(), pin_state) {
        (DivMode::Level, true) => {
            drop(pwm);
            start_channel(
                pwm_ref,
                channel_idx,
                clock_ref,
                gpio_ref,
                interrupts_ref,
                inspector,
            );
        }
        (DivMode::Level, false) => {
            drop(pwm);
            stop_channel(channel_idx, clock_ref);
        }
        (DivMode::Rise, true) | (DivMode::Fall, false) => {
            channel.advance();
            pwm.update_gpio(gpio_ref, channel_idx);
            pwm.update_interrupt(interrupts_ref);
        }
        _ => return,
    }
}

pub(super) fn stop_channel(channel: usize, clock_ref: Rc<Clock>) {
    clock_ref.cancel(EventType::Pwm(channel));
}

pub(super) fn start_channel(
    pwm_ref: Rc<RefCell<Pwm>>,
    channel: usize,
    clock_ref: Rc<Clock>,
    gpio: Rc<RefCell<GpioController>>,
    interrupts: Rc<RefCell<Interrupts>>,
    inspector: InspectorRef,
) {
    let pwm = pwm_ref.borrow();
    let is_channel_enabled = pwm.channels[channel].is_enabled();
    let next_tick = pwm.channels[channel].next_update();
    drop(pwm);

    if is_channel_enabled {
        let clock = clock_ref.clone();
        clock.schedule(next_tick, EventType::Pwm(channel), move || {
            channel_update(pwm_ref, channel, clock_ref, gpio, interrupts, inspector)
        });
    }
}

pub(super) fn channel_update(
    pwm_ref: Rc<RefCell<Pwm>>,
    channel_idx: usize,
    clock_ref: Rc<Clock>,
    gpio_ref: Rc<RefCell<GpioController>>,
    interrupts_ref: Rc<RefCell<Interrupts>>,
    inspector: InspectorRef,
) {
    let ticks = {
        let mut pwm = pwm_ref.borrow_mut();
        let ref mut channel = pwm.channels[channel_idx];
        channel.advance();
        let ticks = channel.next_update();
        pwm.update_gpio(gpio_ref.clone(), channel_idx);
        pwm.update_interrupt(interrupts_ref.clone());
        ticks
    };

    let pwm = pwm_ref.clone();
    let clock = clock_ref.clone();
    let gpio = gpio_ref.clone();
    let interrupts = interrupts_ref.clone();
    let inspector = inspector.clone();

    clock_ref.schedule(ticks, EventType::Pwm(channel_idx), move || {
        channel_update(pwm, channel_idx, clock, gpio, interrupts, inspector)
    });
}
