/**
 * @file clock.rs
 * @author Nguyen Le Duy
 * @date 02/01/2025
 * @brief Clock module for the Rp2350 simulator to handle the clock and events.
 */
use std::cell::RefCell;
use std::collections::BTreeSet;

use crate::common::MHZ;

pub mod event;
pub mod tick;

pub use event::{Event, EventFn, EventType};
pub use tick::*;

#[derive(Default)]
pub struct Clock {
    pub ticks: RefCell<u64>,
    pub events: RefCell<BTreeSet<Event>>,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            ticks: RefCell::new(0),
            events: RefCell::new(BTreeSet::new()),
        }
    }

    pub fn tick(&self) {
        let ticks = {
            let mut tmp = self.ticks.borrow_mut();
            *tmp += 1;
            *tmp
        };

        let mut events = Vec::new();
        let mut planned_events = self.events.borrow_mut();

        while planned_events
            .first()
            .filter(|v| v.activation_time <= ticks)
            .is_some()
        {
            events.push(planned_events.pop_first().unwrap());
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
    pub fn schedule<T: Into<Ticks>, F: FnOnce() + 'static>(
        &self,
        ticks: T,
        typ: EventType,
        event_fn: F,
    ) -> u64 {
        let ticks = ticks.into().into_ticks_number();
        let activation_time = *self.ticks.borrow() + ticks;
        self.events
            .borrow_mut()
            .insert(Event::new(activation_time, typ, event_fn));

        activation_time
    }

    pub fn is_scheduled(&self, typ: EventType) -> bool {
        self.events.borrow().iter().any(|event| event.typ == typ)
    }

    pub fn cancel(&self, typ: EventType) {
        self.events.borrow_mut().retain(|event| {
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
