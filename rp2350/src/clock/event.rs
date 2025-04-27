use core::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EventType {
    DmaChannelTimer(usize),
    UartTx(usize),
    UartRx(usize),
    Sha256,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::DmaChannelTimer(ch) => write!(f, "DMA Channel {}", ch),
            EventType::Sha256 => write!(f, "SHA256"),
            EventType::UartTx(ch) => write!(f, "UART Tx {}", ch),
            EventType::UartRx(ch) => write!(f, "UART Rx {}", ch),
        }
    }
}

pub type EventFn = Box<dyn FnOnce()>;

// pub type IntervalFn = Box<dyn Fn()>;

// pub struct Interval {
//     pub name: String,
//     pub period: Duration,
//     pub last_time: Duration,
//     interval_fn: IntervalFn,
// }

pub struct Event {
    pub typ: EventType,
    event_fn: EventFn,
}

impl Event {
    pub fn new<F: FnOnce() + 'static>(typ: EventType, event_fn: F) -> Self {
        Self {
            typ,
            event_fn: Box::new(event_fn),
        }
    }

    pub fn exec(self) {
        (self.event_fn)();
    }
}
