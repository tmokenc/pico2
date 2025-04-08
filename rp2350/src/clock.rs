use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;
use std::time::Duration;

pub type EventFn<const MHZ: u64> = fn(&mut Clock<MHZ>);

pub struct Event<const MHZ: u64> {
    pub activation_time: u64,
    pub name: String,
    canceled: bool,
    event_fn: EventFn<MHZ>,
}

impl<const MHZ: u64> Event<MHZ> {
    pub fn cancel(&mut self) {
        self.canceled = true;
    }
}

impl<const MHZ: u64> PartialOrd for Event<MHZ> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const MHZ: u64> PartialEq for Event<MHZ> {
    fn eq(&self, other: &Self) -> bool {
        self.activation_time == other.activation_time
    }
}

impl<const MHZ: u64> Eq for Event<MHZ> {}

impl<const MHZ: u64> Ord for Event<MHZ> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.activation_time.cmp(&other.activation_time)
    }
}

#[derive(Default)]
pub struct Clock<const MHZ: u64> {
    pub ticks: u64,
    pub events: BTreeSet<Rc<RefCell<Event<MHZ>>>>,
}

impl<const MHZ: u64> Clock<MHZ> {
    pub fn tick(&mut self) {
        self.ticks += 1;

        loop {
            if let Some(event) = self.events.first() {
                // check for activation of the event
                if event.borrow().activation_time > self.ticks {
                    break;
                }
            }

            let event = self.events.pop_first().unwrap();
            let event = event.borrow();

            if !event.canceled {
                log::info!("Event {} activated at tick {}", event.name, self.ticks);
                (event.event_fn)(self);
            }
        }
    }

    pub fn schedule(
        &mut self,
        ticks: u64,
        name: &str,
        event_fn: EventFn<MHZ>,
    ) -> Rc<RefCell<Event<MHZ>>> {
        let event = Rc::new(RefCell::new(Event {
            activation_time: self.ticks + ticks,
            name: name.to_string(),
            canceled: false,
            event_fn,
        }));

        log::info!("Scheduling event {} at tick {}", name, self.ticks + ticks);
        self.events.insert(Rc::clone(&event));

        event
    }
}
