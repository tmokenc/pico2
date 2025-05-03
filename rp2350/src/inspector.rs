use std::rc::Rc;

#[derive(Debug, Clone)]
pub enum InspectionEvent {
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

    Tick(u8),
    WakeCore(u8),

    UartTx(u8),
    UartRx(u16),
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
            inspector: Rc::new(DummyInspector),
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

    pub fn raise(&self, event: InspectionEvent) {
        self.inspector.handle_event(event);
    }
}

pub struct DummyInspector;

impl Inspector for DummyInspector {
    fn handle_event(&self, event: InspectionEvent) {
        match event {
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

            InspectionEvent::Tick(core) => {
                log::info!("Core {core}: Tick event");
            }
            InspectionEvent::WakeCore(core) => {
                log::info!("Core {core}: Wake event");
            }

            InspectionEvent::UartTx(core) => {
                log::info!("Core {core}: UART TX event");
            }

            InspectionEvent::UartRx(value) => {
                log::info!("UART RX event: {value}");
            }
        }
    }
}
