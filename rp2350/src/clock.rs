use std::collections::BTreeMap;
use std::time::Duration;

use crate::common::MHZ;

pub type EventFn = Box<dyn FnOnce()>;
pub type IntervalFn = Box<dyn Fn()>;

pub struct Interval {
    pub name: String,
    pub period: Duration,
    pub last_time: Duration,
    interval_fn: IntervalFn,
}

pub struct Event {
    pub name: String,
    event_fn: EventFn,
}

#[derive(Default)]
pub struct Clock {
    pub clk_spd: u64,
    pub ticks: u64,
    pub events: BTreeMap<u64, Event>,
}

impl Clock {
    pub fn new(clk_spd: u64) -> Self {
        Self {
            clk_spd,
            ticks: 0,
            events: BTreeMap::new(),
        }
    }

    pub fn tick(&mut self) {
        self.ticks += 1;

        loop {
            if self
                .events
                .first_key_value()
                .filter(|v| v.0 <= &self.ticks)
                .is_none()
            {
                break;
            }

            let (_atime, event) = self.events.pop_first().unwrap();
            log::info!("Event {} activated at tick {}", event.name, self.ticks);
            (event.event_fn)();
        }
    }

    /// Schedule an event to be executed after a certain number of ticks.
    /// Return the activation time of the event.
    /// Combining with the name, it can be used to cancel the event.
    pub fn schedule<F: FnOnce() + 'static>(&mut self, ticks: u64, name: &str, event_fn: F) -> u64 {
        let activation_time = self.ticks + ticks;
        log::info!("Scheduling event {} at tick {}", name, self.ticks + ticks);
        self.events.insert(
            activation_time,
            Event {
                name: name.to_string(),
                event_fn: Box::new(event_fn) as EventFn,
            },
        );

        activation_time
    }

    pub fn cancel(&mut self, activation_time: u64, name: &str) {
        self.events.retain(|&atime, event| {
            if atime == activation_time && event.name == name {
                log::info!("Cancelling event {} at tick {}", name, self.ticks);
                false
            } else {
                true
            }
        });
    }

    pub fn clk_sys(&self) -> u64 {
        150 * MHZ
    }

    pub fn clk_ref(&self) -> u64 {
        12 * MHZ
    }

    pub fn clk_peri(&self) -> u64 {
        150 * MHZ
    }

    pub fn clk_usb(&self) -> u64 {
        48 * MHZ
    }

    pub fn clk_adc(&self) -> u64 {
        48 * MHZ
    }

    pub fn clk_hstx(&self) -> u64 {
        150 * MHZ
    }
}
