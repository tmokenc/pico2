/**
 * @file inspector.rs
 * @author Nguyen Le Duy
 * @date 05/05/2025
 * @brief Inspector module for the Rp2350 simulator to track events.
 */
use std::rc::Rc;

use crate::bus::BusError;
use crate::clock::EventType;
use crate::common::{DataSize, Requestor};

#[derive(Debug, Clone)]
pub enum InspectionEvent {
    ClockEventActivated(EventType),
    ClockEventScheduled(EventType),
    ClockEventCanceled(EventType),
    TrngGenerated(u32),
    ExecutedInstruction {
        core: u8,
        instruction: u32,
        address: u32,
        name: &'static str,
        operands: Vec<u32>,
    },
    Exception {
        core: u8,
        exception: u32,
    },

    BusStore {
        requestor: Requestor,
        size: DataSize,
        address: u32,
        value: u32,
    },

    BusLoad {
        requestor: Requestor,
        size: DataSize,
        address: u32,
    },

    BusError {
        error: BusError,
        requestor: Requestor,
        size: DataSize,
        address: u32,
    },

    TickCore(u8),
    WakeCore(u8),

    UartTx {
        uart_index: u8,
        value: u8,
    },
    UartRx {
        uart_index: u8,
        value: u16,
    },
}

pub trait Inspector {
    fn handle_event(&self, event: InspectionEvent);
}

#[derive(Clone)]
pub struct InspectorRef {
    inspector: Rc<dyn Inspector>,
}

impl Default for InspectorRef {
    fn default() -> Self {
        Self {
            inspector: Rc::new(LoggerInspector),
        }
    }
}

impl Inspector for InspectorRef {
    fn handle_event(&self, event: InspectionEvent) {
        self.inspector.handle_event(event);
    }
}

impl InspectorRef {
    pub fn set_inspector(&mut self, inspector: Rc<dyn Inspector>) {
        self.inspector = inspector;
    }

    pub fn emit(&self, event: InspectionEvent) {
        self.inspector.handle_event(event);
    }
}

pub struct LoggerInspector;

impl Inspector for LoggerInspector {
    fn handle_event(&self, event: InspectionEvent) {
        match event {
            InspectionEvent::ClockEventActivated(typ) => {
                log::trace!("Clock event activated: {typ:?}");
            }
            InspectionEvent::ClockEventScheduled(typ) => {
                log::trace!("Clock event scheduled: {typ:?}");
            }
            InspectionEvent::ClockEventCanceled(typ) => {
                log::trace!("Clock event canceled: {typ:?}");
            }
            InspectionEvent::TrngGenerated(value) => {
                log::info!("RNG: generated value: {value}");
            }

            InspectionEvent::Exception { core, exception } => {
                log::info!("Core {core}: Exception: {exception:#010x}");
            }

            InspectionEvent::ExecutedInstruction {
                core,
                instruction,
                address,
                name,
                operands,
            } => {
                log::info!(
                    "Core {core}: Executed instruction: {instruction:#010x} at {address:#010x} - {name}({:?})",
                    operands
                );
            }

            InspectionEvent::TickCore(core) => {
                log::trace!("Core {core}: Tick event");
            }
            InspectionEvent::WakeCore(core) => {
                log::info!("Core {core}: Wake event");
            }

            InspectionEvent::UartTx { uart_index, value } => {
                log::info!("UART TX event on UART {uart_index}: {value}");
            }

            InspectionEvent::UartRx { uart_index, value } => {
                log::info!("UART RX event on UART {uart_index}: {value}");
            }

            InspectionEvent::BusStore {
                requestor,
                size,
                address,
                value,
            } => {
                log::info!("Bus Store: {requestor:?} {size:?} address: {address:#010x} | value: {value:#010x}");
            }

            InspectionEvent::BusLoad {
                requestor,
                size,
                address,
            } => {
                log::info!("Bus Load: {requestor:?} {size:?} address: {address:#010x}");
            }

            InspectionEvent::BusError {
                error,
                requestor,
                size,
                address,
            } => {
                // Detailing about error message
                log::error!("Bus Error: {error:?} {requestor:?} {size:?} address: {address:#010x}");
            }
        }
    }
}
