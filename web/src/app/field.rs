use super::Rp2350Component;
use egui_alignments::AlignedWidget;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Field {
    show_schematic: bool,
}

impl Rp2350Component for Field {
    const NAME: &'static str = "Field";

    fn ui(&mut self, ui: &mut egui::Ui, _rp2350: &mut Rp2350) {
        // add radio button to toggle schematic view
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.show_schematic, false, "Field");
            ui.radio_value(&mut self.show_schematic, true, "Schematic");
        });

        ui.add_space(12.0);

        if self.show_schematic {
            self.schematic_ui(ui);
        } else {
            self.field_ui(ui, _rp2350);
        }
    }
}

impl Field {
    fn schematic_ui(&mut self, ui: &mut egui::Ui) {
        egui::Image::new(egui::include_image!("../../assets/pico2_schematic.webp"))
            .alt_text("Raspberry Pi Pico 2 Schematic")
            .maintain_aspect_ratio(true)
            .max_height(500.0)
            .fit_to_original_size(1.0)
            .center(ui);
    }

    fn field_ui(&mut self, ui: &mut egui::Ui, _rp2350: &mut Rp2350) {
        let img_res = egui::Image::new(egui::include_image!("../../assets/pico2.webp"))
            .alt_text("Raspberry Pi Pico 2")
            .maintain_aspect_ratio(true)
            .max_height(500.0)
            .fit_to_original_size(1.0)
            .center(ui);

        // TODO
    }
}
