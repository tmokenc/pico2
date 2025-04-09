use std::collections::BTreeMap;
use std::time::Duration;

pub type EventFn = Box<dyn FnOnce()>;

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
            if let Some((atime, _event)) = self.events.first_key_value() {
                // check for activation of the event
                if *atime > self.ticks {
                    break;
                }
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
}
