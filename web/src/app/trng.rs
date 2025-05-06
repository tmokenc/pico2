/**
 * @file app/trng.rs
 * @author Nguyen Le Duy
 * @date 14/04/2025
 * @brief View window for the TRNG peripheral
 */
use super::Rp2350Component;
use rp2350::Rp2350;
use std::rc::Rc;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Trng {
    // None
}

impl Rp2350Component for Trng {
    const NAME: &'static str = "TRNG";

    fn ui_with_tracker(
        &mut self,
        ui: &mut egui::Ui,
        _rp2350: &mut Rp2350,
        tracker: Rc<crate::Tracker>,
    ) {
        ui.heading("TRNG");
        let tracker = tracker.borrow();

        egui::Grid::new("TRNG")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Last generated");
                match tracker.last_generated_trng.as_ref() {
                    Some(trng) => {
                        ui.label(format!("{:08x}", trng));
                    }
                    None => {
                        ui.label("None");
                    }
                }

                ui.end_row();
            });
    }
}
