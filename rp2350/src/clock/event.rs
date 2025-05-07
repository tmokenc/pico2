/**
 * @file clock/event.rs
 * @author Nguyen Le Duy
 * @date 02/01/2025
 * @brief Definition of the event type that used in the calendar
 */
use core::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum EventType {
    DmaChannelTimer(usize),
    Pwm(usize),
    UartTx(usize),
    UartRx(usize),
    Timer(usize),
    Sha256,
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EventType::DmaChannelTimer(ch) => write!(f, "DMA Channel {}", ch),
            EventType::Sha256 => write!(f, "SHA256"),
            EventType::UartTx(ch) => write!(f, "UART Tx {}", ch),
            EventType::UartRx(ch) => write!(f, "UART Rx {}", ch),
            EventType::Pwm(ch) => write!(f, "PWM {}", ch),
            EventType::Timer(ch) => write!(f, "Timer {}", ch),
        }
    }
}

pub type EventFn = Box<dyn FnOnce()>;

pub struct Event {
    pub activation_time: u64,
    pub typ: EventType,
    event_fn: EventFn,
}

impl PartialEq for Event {
    fn eq(&self, other: &Self) -> bool {
        self.typ == other.typ
    }
}

impl Eq for Event {}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        let cmp = self.activation_time.partial_cmp(&other.activation_time);

        if cmp == Some(std::cmp::Ordering::Equal) {
            Some(self.typ.partial_cmp(&other.typ).unwrap())
        } else {
            cmp
        }
    }
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let cmp = self.activation_time.cmp(&other.activation_time);
        if cmp == std::cmp::Ordering::Equal {
            self.typ.cmp(&other.typ)
        } else {
            cmp
        }
    }
}

impl Event {
    pub fn new<F: FnOnce() + 'static>(activation_time: u64, typ: EventType, event_fn: F) -> Self {
        Self {
            activation_time,
            typ,
            event_fn: Box::new(event_fn),
        }
    }

    pub fn exec(self) {
        (self.event_fn)();
    }
}
