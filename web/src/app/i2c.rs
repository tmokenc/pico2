/**
 * @file app/i2c.rs
 * @author Nguyen Le Duy
 * @date 11/05/2025
 * @brief View window for the I2C peripheral
 */
use super::Rp2350Component;
use crate::tracker::I2cTracker;
use rp2350::Rp2350;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct I2c<const IDX: usize> {
    // None
}

impl<const IDX: usize> Rp2350Component for I2c<IDX> {
    const NAME: &'static str = "I2C";

    fn ui_with_tracker(
        &mut self,
        ui: &mut egui::Ui,
        rp2350: &mut Rp2350,
        tracker: Rc<crate::Tracker>,
    ) {
        ui.heading(format!("I2C {IDX}"));

        let tracker = tracker.borrow();
        match IDX {
            0 => view_i2c(ui, &rp2350.bus.peripherals.i2c0, &tracker.i2c[0]),
            1 => view_i2c(ui, &rp2350.bus.peripherals.i2c1, &tracker.i2c[1]),
            _ => unreachable!(),
        }
    }
}

fn view_i2c<const IDX: usize>(
    ui: &mut egui::Ui,
    _i2c: &Rc<RefCell<rp2350::peripherals::I2c<IDX>>>,
    _tracker: &I2cTracker,
) {
    ui.label("I2C peripheral is not implemented yet");
    /*
    let i2c = i2c.borrow();
    egui::Grid::new(format!("Uart {IDX}"))
        .num_columns(2)
        .spacing([40.0, 6.0])
        .striped(false)
        .show(ui, |ui| {
            // is enabled
            ui.label("Enabled");
            if uart.is_enabled() {
                ui.label("Yes");
            } else {
                ui.label("No");
            }
            ui.end_row();

            // is TX enabled
            ui.label("TX Enabled");
            if uart.is_transmit_enabled() {
                ui.label("Yes");
            } else {
                ui.label("No");
            }
            ui.end_row();

            // is RX enabled
            ui.label("RX Enabled");
            if uart.is_receive_enabled() {
                ui.label("Yes");
            } else {
                ui.label("No");
            }
            ui.end_row();

            // Baud rate
            ui.label("Baud Rate");
            ui.label(format!("{}", uart.get_baudrate()));
            ui.end_row();

            // Data bits
            ui.label("Data bits");
            ui.label(format!("{} bits", uart.word_len()));
            ui.end_row();

            // Stop bits
            ui.label("Stop bits");
            ui.label(if uart.two_stop_bits() {
                "2 bits"
            } else {
                "1 bit"
            });
            ui.end_row();

            // Parity Odd/Even/None
            ui.label("Parity");
            if uart.is_parity_enabled() {
                ui.label(if uart.is_parity_even() { "Even" } else { "Odd" });
            } else {
                ui.label("None");
            }
            ui.end_row();
        });

    // FIFO
    // Transmitting FIFO

    // Receiving FIFO

    ui.collapsing("Transmitted value", |ui| {
        ScrollArea::vertical()
            .max_width(f32::INFINITY)
            .max_height(200.0) // TODO
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let mut str = String::with_capacity(uart_tracker.tx.len());

                for ch in &uart_tracker.tx {
                    str.push(char::from(*ch));
                }

                ui.label(RichText::new(str).monospace());
            });
    });

    ui.collapsing("Received value", |ui| {
        ScrollArea::vertical()
            .max_width(f32::INFINITY)
            .max_height(200.0) // TODO
            .stick_to_bottom(true)
            .show(ui, |ui| {
                let mut str = String::with_capacity(uart_tracker.rx.len());

                for ch in &uart_tracker.rx {
                    str.push(char::from(*ch as u8));
                }

                ui.label(RichText::new(str).monospace());
            });
    });
    */
}
