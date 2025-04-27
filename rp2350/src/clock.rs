use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Duration;

use crate::bus::Bus;
use crate::common::MHZ;

pub mod event;

pub use event::{Event, EventFn, EventType};

pub trait IntoTicks {
    fn into_tick(self) -> u64;
}

impl IntoTicks for u64 {
    fn into_tick(self) -> u64 {
        self
    }
}

impl IntoTicks for Duration {
    fn into_tick(self) -> u64 {
        let base = 1.0 / (150 * MHZ) as f64;
        let base = Duration::from_secs_f64(base);
        self.div_duration_f64(base).ceil() as u64
    }
}

#[derive(Default)]
pub struct Clock {
    pub clk_spd: u64,
    pub ticks: RefCell<u64>,
    pub events: RefCell<BTreeMap<u64, Event>>,
}

impl Clock {
    pub fn new(clk_spd: u64) -> Self {
        Self {
            clk_spd,
            ticks: RefCell::new(0),
            events: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn tick(&self, bus: &mut Bus) {
        let ticks = {
            let mut tmp = self.ticks.borrow_mut();
            *tmp += 1;
            *tmp
        };
        let mut events = Vec::new();
        let mut planned_events = self.events.borrow_mut();

        while planned_events
            .first_key_value()
            .filter(|v| v.0 <= &ticks)
            .is_some()
        {
            events.push(planned_events.pop_first().unwrap().1);
        }

        // Avoid deadlock if the event functions tries to schedule another event
        drop(planned_events);

        for event in events {
            log::info!("Event {} activated at tick {}", event.typ, ticks);
            event.exec();
        }
    }

    /// Schedule an event to be executed after a certain number of ticks.
    /// Return the activation time of the event.
    /// Combining with the name, it can be used to cancel the event.
    pub fn schedule<T: IntoTicks, F: FnOnce() + 'static>(
        &self,
        ticks: T,
        typ: EventType,
        event_fn: F,
    ) -> u64 {
        let ticks = ticks.into_tick();
        let activation_time = *self.ticks.borrow() + ticks;
        log::info!("Scheduling event {} at tick {}", typ, activation_time);
        self.events
            .borrow_mut()
            .insert(activation_time, Event::new(typ, event_fn));

        activation_time
    }

    pub fn is_scheduled(&self, typ: EventType) -> bool {
        self.events
            .borrow()
            .iter()
            .any(|(_, event)| event.typ == typ)
    }

    pub fn cancel(&self, typ: EventType) {
        self.events.borrow_mut().retain(|&atime, event| {
            if event.typ == typ {
                log::info!("Cancelling event {} at tick {}", typ, *self.ticks.borrow());
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
