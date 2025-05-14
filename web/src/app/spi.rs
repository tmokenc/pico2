/**
 * @file app/spi.rs
 * @author Nguyen Le Duy
 * @date 11/05/2025
 * @brief View window for the SPI peripheral
 */
use super::Rp2350Component;
use rp2350::Rp2350;
use std::rc::Rc;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Spi<const IDX: usize> {
    // None
}

impl<const IDX: usize> Rp2350Component for Spi<IDX> {
    const NAME: &'static str = "SPI";

    fn ui_with_tracker(
        &mut self,
        ui: &mut egui::Ui,
        _rp2350: &mut Rp2350,
        _tracker: Rc<crate::Tracker>,
    ) {
        ui.heading(format!("SPI {IDX}"));

        ui.label("SPI peripheral is not implemented yet");

        // let tracker = tracker.borrow();
        // match IDX {
        //     0 => view_spi(ui, &rp2350.bus.peripherals.spi0, &tracker.spi[0]),
        //     1 => view_spi(ui, &rp2350.bus.peripherals.spi1, &tracker.spi[1]),
        //     _ => unreachable!(),
        // }
    }
}

/*
fn view_spi<const IDX: usize>(
    ui: &mut egui::Ui,
    spi: &Rc<RefCell<rp2350::peripherals::Spi<IDX>>>,
    _tracker: &SpiTracker,
) {
    let spi = spi.borrow();
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
}
*/
