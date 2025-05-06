/**
 * @file peripherals/uart/receive.rs
 * @author Nguyen Le Duy
 * @date 23/04/2025
 * @brief Receive state machine for UART
 */
use crate::clock::Clock;
use crate::clock::EventType;
use crate::gpio::FunctionSelect;
use crate::gpio::GpioController;
use crate::inspector::InspectorRef;
use crate::interrupts::Interrupts;
use crate::peripherals::uart::Uart;
use crate::peripherals::PeripheralAccessContext;
use crate::InspectionEvent;
use std::cell::RefCell;
use std::rc::Rc;

use super::get_even_parity;
use super::get_odd_parity;
use super::FRAME_ERROR;
use super::OVERRUN_ERROR;
use super::PARITY_ERROR;

pub(super) fn start_receiving<const IDX: usize>(
    uart: Rc<RefCell<Uart<IDX>>>,
    ctx: &PeripheralAccessContext,
) {
    if ctx.clock.is_scheduled(EventType::UartRx(IDX)) {
        return;
    }

    receive(
        uart.clone(),
        0,
        ReceiveState::StartReceiving,
        ctx.clock.clone(),
        ctx.interrupts.clone(),
        ctx.gpio.clone(),
        ctx.inspector.clone(),
    );
}

pub(super) fn abort_receiving<const IDX: usize>(
    _uart: Rc<RefCell<Uart<IDX>>>,
    ctx: &PeripheralAccessContext,
) {
    // TODO change flags?
    ctx.clock.cancel(EventType::UartRx(IDX));
}

#[derive(Clone, Copy)]
enum ReceiveState {
    StartReceiving,
    Idle { last_bit: u8 },
    DataBit { index: u8 },
    ParityBit,
    StopBit { index: u8 },
}

fn receive<const IDX: usize>(
    uart_ref: Rc<RefCell<Uart<IDX>>>,
    mut data: u8,
    state: ReceiveState,
    clock: Rc<Clock>,
    interrupts: Rc<RefCell<Interrupts>>,
    gpio_ref: Rc<RefCell<GpioController>>,
    inspector: InspectorRef,
) {
    let mut uart = uart_ref.borrow_mut();
    let bit_time = uart.get_bit_time();

    let mut next_state: ReceiveState = state;
    let mut gpio = gpio_ref.borrow_mut();

    if let Some(gpio_pin) = gpio.select(rx_gpio_func::<IDX>()) {
        let bit = gpio_pin.input_value() as u8;

        match state {
            ReceiveState::StartReceiving => {
                next_state = ReceiveState::Idle { last_bit: bit };
            }

            ReceiveState::Idle { last_bit } => {
                if last_bit == 1 && bit == 0 {
                    // Start bit detected
                    next_state = ReceiveState::DataBit { index: 0 };
                } else {
                    // No start bit detected, continue waiting
                    next_state = ReceiveState::Idle { last_bit: bit };
                }
            }
            ReceiveState::DataBit { index } => {
                data |= bit << index;
                if index >= uart.word_len() - 1 {
                    // All data bits received, move to parity bit
                    if uart.is_parity_enabled() {
                        next_state = ReceiveState::ParityBit;
                    } else {
                        next_state = ReceiveState::StopBit { index: 0 };
                    }
                } else {
                    // Move to next data bit
                    next_state = ReceiveState::DataBit { index: index + 1 };
                }
            }
            ReceiveState::ParityBit => {
                next_state = ReceiveState::StopBit { index: 0 };
                let parity = match uart.is_parity_even() {
                    true => get_even_parity(data, uart.word_len()),
                    false => get_odd_parity(data, uart.word_len()),
                };

                if parity != bit {
                    uart.error |= PARITY_ERROR;
                    uart.update_interrupt(interrupts.clone());
                }
            }
            ReceiveState::StopBit { index } => {
                if bit != 1 {
                    // invalid stop bit
                    uart.error |= FRAME_ERROR;
                    uart.update_interrupt(interrupts.clone());
                    next_state = ReceiveState::Idle { last_bit: bit };
                } else if uart.two_stop_bits() && index == 1 {
                    // All stop bits received, move to idle state
                    next_state = ReceiveState::Idle { last_bit: bit };
                    uart.update_interrupt(interrupts.clone());

                    let data = data as u16 | (uart.error as u16) << 8;
                    inspector.emit(InspectionEvent::UartRx {
                        uart_index: IDX as u8,
                        value: data,
                    });

                    if let Err(_why) = uart.rx_fifo.push(data) {
                        uart.error |= OVERRUN_ERROR;
                        uart.update_interrupt(interrupts.clone());
                    }
                } else {
                    // Move to next stop bit
                    next_state = ReceiveState::StopBit { index: index + 1 };
                }
            }
        }
    }

    let clock_clone = clock.clone();
    let uart_ref = uart_ref.clone();
    let gpio_ref = gpio_ref.clone();

    clock.schedule(bit_time, EventType::UartRx(IDX), move || {
        receive(
            uart_ref,
            data,
            next_state,
            clock_clone,
            interrupts,
            gpio_ref,
            inspector,
        );
    });
}

const fn rx_gpio_func<const IDX: usize>() -> FunctionSelect {
    match IDX {
        0 => FunctionSelect::UART0_RX,
        1 => FunctionSelect::UART1_RX,
        _ => unreachable!(),
    }
}
