/**
 * @file app/bus.rs
 * @author Nguyen Le Duy
 * @date 12/05/2025
 * @brief View window for the Bus system
 */
use super::Rp2350Component;
use rp2350::Rp2350;
use std::rc::Rc;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Bus {
    // None
}

impl Rp2350Component for Bus {
    const NAME: &'static str = "Bus";

    fn ui_with_tracker(
        &mut self,
        ui: &mut egui::Ui,
        _rp2350: &mut Rp2350,
        tracker: Rc<crate::Tracker>,
    ) {
        ui.heading("Bus");
        let tracker = tracker.borrow();

        egui::Grid::new("Bus")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                // TODO
                ui.end_row();
            });
    }
}
