/**
 * @file app/sio.rs
 * @author Nguyen Le Duy
 * @date 10/05/2025
 * @brief View window for the SIO peripheral
 */
use super::Rp2350Component;
use egui::collapsing_header::CollapsingState;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Sio {
    // None
}

impl Sio {
    fn mailbox_ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        let mailbox = rp2350.bus.peripherals.sio.mailboxes.borrow();

        egui::Grid::new("SIO MAILBOX")
            .num_columns(3)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                ui.strong("");
                ui.strong("Core 0");
                ui.strong("Core 1");
                ui.end_row();

                let mut fifo0 = mailbox.data[0].iter();
                let mut fifo1 = mailbox.data[1].iter();
                for i in 0..8 {
                    ui.label(format!("FIFO {}", i));
                    ui.label(match fifo0.next() {
                        Some(value) => format!("{:08x}", value),
                        None => "Empty".to_string(),
                    });
                    ui.label(match fifo1.next() {
                        Some(value) => format!("{:08x}", value),
                        None => "Empty".to_string(),
                    });
                    ui.end_row();
                }

                ui.label("Read On Empty Error");
                ui.label(if mailbox.roe[0] { "Yes" } else { "No" });
                ui.label(if mailbox.roe[1] { "Yes" } else { "No" });
                ui.end_row();

                ui.label("Write On Full Error");
                ui.label(if mailbox.wof[0] { "Yes" } else { "No" });
                ui.label(if mailbox.wof[1] { "Yes" } else { "No" });
                ui.end_row();
            });
    }

    fn spinlock_ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        let spinlock = rp2350.bus.peripherals.sio.spinlock.state();

        egui::Grid::new("SIO")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                for i in 0..8 {
                    ui.label(format!("Spinlock {}", i));
                    ui.label(if (spinlock & (1 << i)) != 0 {
                        "Locked"
                    } else {
                        "Unlocked"
                    });
                    ui.end_row();
                }
            });
    }

    fn timer_ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        let timer = rp2350.bus.peripherals.sio.timer.borrow();

        egui::Grid::new("SIO TIMER")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Enabled");
                ui.label(if (timer.ctrl & 1) == 1 { "Yes" } else { "No" });
                ui.end_row();

                ui.label("Current Counter");
                ui.label(format!("{}", timer.counter));
                ui.end_row();

                ui.label("Compare value");
                ui.label(format!("{}", timer.cmp));
                ui.end_row();

                ui.label("Counting speed");
                ui.label(if (timer.ctrl & 0b10) != 0 {
                    "150 MHz"
                } else {
                    "1 MHz"
                });
            });
    }
}

impl Rp2350Component for Sio {
    const NAME: &'static str = "SIO";

    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading("Single Cycle I/O");

        ui.add_space(12.0);

        CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id("sio_mailbox"),
            true,
        )
        .show_header(ui, |ui| {
            ui.heading("Mailbox");
        })
        .body(|ui| {
            self.mailbox_ui(ui, rp2350);
        });

        ui.add_space(12.0);

        CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id("sio_spinlock"),
            true,
        )
        .show_header(ui, |ui| {
            ui.heading("Spinlock");
        })
        .body(|ui| {
            self.spinlock_ui(ui, rp2350);
        });

        ui.add_space(12.0);

        CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id("sio_timer"), true)
            .show_header(ui, |ui| {
                ui.heading("Timer");
            })
            .body(|ui| {
                self.timer_ui(ui, rp2350);
            });
    }
}
