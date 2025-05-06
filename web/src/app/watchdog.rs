/**
 * @file app/watchdog.rs
 * @author Nguyen Le Duy
 * @date 23/04/2025
 * @brief View window for the WatchDog peripheral
 */
use super::Rp2350Component;
use crate::widgets::DisplayMode;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct WatchDog {
    scratches: [DisplayMode; 8],
}

impl Rp2350Component for WatchDog {
    const NAME: &'static str = "WatchDog";

    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading("Watch Dog");

        let watchdog = &rp2350.bus.peripherals.watch_dog;

        egui::Grid::new("WatchDog Control Info")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Enable");
                ui.label(format!("{}", watchdog.enable));
                ui.end_row();

                ui.label("Pause Debug 0");
                ui.label(format!("{}", watchdog.pause_dbg0));
                ui.end_row();

                ui.label("Pause Debug 1");
                ui.label(format!("{}", watchdog.pause_dbg1));
                ui.end_row();

                ui.label("Pause JTag");
                ui.label(format!("{}", watchdog.pause_jtag));
                ui.end_row();

                ui.label("Timer");
                ui.label(format!("{:#010x}", watchdog.timer));
                ui.end_row();
            });

        ui.heading("Reason");

        egui::Grid::new("WatchDog Reason Info")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Timer");
                ui.label(format!("{}", watchdog.reason_timer));
                ui.end_row();

                ui.label("Force");
                ui.label(format!("{}", watchdog.reason_force));
                ui.end_row();
            });

        egui::Grid::new("WatchDog Scratches")
            .num_columns(3)
            .spacing([40.0, 6.0])
            .striped(true)
            .show(ui, |ui| {
                for (i, value) in watchdog.scratch.iter().enumerate() {
                    ui.label(format!("Scratch {}:", i));
                    ui.add(self.scratches[i].bin_dec_hex_char());
                    ui.label(self.scratches[i].fmt_u32(*value));
                    ui.end_row();
                }
            });
    }
}
