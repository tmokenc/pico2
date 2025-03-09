use super::Rp2350Component;
use eframe::APP_KEY;
use egui::Vec2;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Field {
    show_schematic: bool,
}

impl Rp2350Component for Field {
    fn ui(&mut self, ui: &mut egui::Ui, _rp2350: &mut Rp2350) {
        // add radio button to toggle schematic view
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.show_schematic, false, "Field");
            ui.radio_value(&mut self.show_schematic, true, "Schematic");
        });

        let image = egui::Image::new(egui::include_image!("../../assets/pico2.png"));

        ui.add(image);

        if self.show_schematic {
            self.schematic_ui(ui, _rp2350);
        } else {
            self.field_ui(ui, _rp2350);
        }
    }
}

impl Field {
    fn schematic_ui(&mut self, ui: &mut egui::Ui, _rp2350: &mut Rp2350) {
        ui.label("Schematic view");
        // TODO
    }

    fn field_ui(&mut self, ui: &mut egui::Ui, _rp2350: &mut Rp2350) {
        ui.label("Field view");
        // TODO
    }
}
