/**
 * @file tracker.rs
 * @author Nguyen Le Duy
 * @date 04/05/2025
 * @brief Tracker module for the simulator
 */
use rp2350::inspector::*;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::ops::Deref;

#[derive(Default)]
pub struct Tracker(RefCell<TrackerInner>);

impl Deref for Tracker {
    type Target = RefCell<TrackerInner>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Instruction {
    pub name: &'static str,
    pub code: u32,
    pub address: u32,
}

#[derive(Default)]
pub struct ProcessorTracker {
    pub inst_count: u64,
    pub instruction_count: HashMap<&'static str, u64>,
    pub instruction_log: VecDeque<Instruction>,
}

pub struct UartTracker {
    pub tx: VecDeque<u8>,
    pub rx: VecDeque<u16>,
    pub max_buffer_size: usize,
}

impl Default for UartTracker {
    fn default() -> Self {
        Self {
            tx: VecDeque::new(),
            rx: VecDeque::new(),
            max_buffer_size: 4096, // Default size to 4096 bytes
        }
    }
}

pub struct TrackerInner {
    pub processor: [ProcessorTracker; 2],
    pub uart: [UartTracker; 2],
    pub last_generated_trng: Option<u32>,
    pub nof_instruction_log: usize,
}

impl Default for TrackerInner {
    fn default() -> Self {
        Self {
            processor: Default::default(),
            uart: Default::default(),
            last_generated_trng: None,
            nof_instruction_log: 50,
        }
    }
}

impl Inspector for Tracker {
    fn handle_event(&self, event: InspectionEvent) {
        // LoggerInspector.handle_event(event.clone());

        let mut inner = self.0.borrow_mut();

        // Handle the event
        match event {
            InspectionEvent::TrngGenerated(value) => {
                inner.last_generated_trng = Some(value);
            }
            InspectionEvent::ExecutedInstruction {
                core,
                instruction,
                address,
                name,
                ..
            } => {
                let instruction = Instruction {
                    name,
                    code: instruction,
                    address,
                };
                let max_len = inner.nof_instruction_log;
                let processor = &mut inner.processor[core as usize];

                push_to_buffer(&mut processor.instruction_log, instruction, max_len);
                *processor.instruction_count.entry(name).or_insert(0) += 1;
                processor.inst_count += 1;
            }

            InspectionEvent::UartTx { uart_index, value } => {
                let uart = &mut inner.uart[uart_index as usize];
                push_to_buffer(&mut uart.tx, value, uart.max_buffer_size);
            }

            InspectionEvent::UartRx { uart_index, value } => {
                let uart = &mut inner.uart[uart_index as usize];
                push_to_buffer(&mut uart.rx, value, uart.max_buffer_size);
            }

            // reset the tracker
            InspectionEvent::FlashedBinary => {
                core::mem::take(&mut *inner);
            }

            _ => {
                // No action needed for other events
            }
        }
    }
}

fn push_to_buffer<T>(buffer: &mut VecDeque<T>, value: T, max_size: usize) {
    if max_size == 0 {
        buffer.clear();
        return;
    }

    if buffer.len() >= max_size {
        buffer.pop_front();
    }
    buffer.push_back(value);
}
