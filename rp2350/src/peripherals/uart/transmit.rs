/**
 * @file peripherals/uart/transmit.rs
 * @author Nguyen Le Duy
 * @date 23/04/2025
 * @brief Transmit state machine for UART
 */
use super::{get_even_parity, get_odd_parity, Uart};
use crate::clock::{Clock, EventType};
use crate::gpio::{FunctionSelect, GpioController};
use crate::inspector::{InspectionEvent, InspectorRef};
use crate::interrupts::Interrupts;
use crate::peripherals::PeripheralAccessContext;
use crate::utils::extract_bit;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy)]
enum TransmitState {
    Idle,
    StartBit,
    DataBit(u8),
    ParityBit,
    StopBit(u8),
}

pub(super) fn start_transmitting<const IDX: usize>(
    uart_ref: Rc<RefCell<Uart<IDX>>>,
    ctx: &PeripheralAccessContext,
) {
    if ctx.clock.is_scheduled(EventType::UartTx(IDX)) {
        return;
    }

    let clock = ctx.clock.clone();
    let interrupts = ctx.interrupts.clone();
    let inspector = ctx.inspector.clone();
    let gpio = ctx.gpio.clone();

    transmit(
        uart_ref.clone(),
        0,
        TransmitState::Idle,
        clock,
        interrupts,
        gpio,
        inspector,
    );
}

fn transmit<const IDX: usize>(
    uart_ref: Rc<RefCell<Uart<IDX>>>,
    data: u8,
    state: TransmitState,
    clock: Rc<Clock>,
    interrupts: Rc<RefCell<Interrupts>>,
    gpio_ref: Rc<RefCell<GpioController>>,
    inspector: InspectorRef,
) {
    let mut uart = uart_ref.borrow_mut();
    let bit_time = uart.get_bit_time();

    if !uart.is_enabled() || !uart.is_transmit_enabled() {
        return;
    }

    let next_state: TransmitState;
    let bit: bool;

    match state {
        TransmitState::Idle => {
            match uart.tx_fifo.pop() {
                None => {
                    uart.update_interrupt(interrupts);
                }
                Some(value) => {
                    inspector.emit(InspectionEvent::UartTx {
                        uart_index: IDX as u8,
                        value,
                    });
                    uart.check_tx_fifo();
                    uart.set_busy(true);
                    drop(uart);
                    transmit(
                        uart_ref,
                        value,
                        TransmitState::StartBit,
                        clock,
                        interrupts,
                        gpio_ref,
                        inspector,
                    );
                }
            }
            return;
        }

        TransmitState::StartBit => {
            next_state = TransmitState::DataBit(0);
            bit = false;
        }

        TransmitState::DataBit(index) => {
            bit = extract_bit(data, index) != 0;

            if index < (uart.word_len() - 1) {
                next_state = TransmitState::DataBit(index + 1);
            } else {
                if uart.is_parity_enabled() {
                    next_state = TransmitState::ParityBit;
                } else {
                    next_state = TransmitState::StopBit(0);
                }
            }
        }

        TransmitState::ParityBit => {
            next_state = TransmitState::StopBit(0);
            bit = match uart.is_parity_even() {
                true => get_even_parity(data, uart.word_len()) != 0,
                false => get_odd_parity(data, uart.word_len()) != 0,
            };
        }

        TransmitState::StopBit(i) => {
            bit = true;
            if uart.two_stop_bits() && i == 0 {
                next_state = TransmitState::StopBit(1);
            } else {
                uart.set_busy(false);
                next_state = TransmitState::Idle;
            }
        }
    }

    drop(uart);

    gpio_ref
        .borrow_mut()
        .set_pin_output(tx_gpio_func::<IDX>(), bit);

    clock
        .clone()
        .schedule(bit_time, EventType::UartTx(IDX), move || {
            transmit(
                uart_ref, data, next_state, clock, interrupts, gpio_ref, inspector,
            );
        });
}

const fn tx_gpio_func<const IDX: usize>() -> FunctionSelect {
    match IDX {
        0 => FunctionSelect::UART0_TX,
        1 => FunctionSelect::UART1_TX,
        _ => unreachable!(),
    }
}
