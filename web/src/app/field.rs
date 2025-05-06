use super::Rp2350Component;
use egui_alignments::AlignedWidget;
use rp2350::gpio::*;
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

    fn field_ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        let img_res = egui::Image::new(egui::include_image!("../../assets/pico2.webp"))
            .alt_text("Raspberry Pi Pico 2")
            .maintain_aspect_ratio(true)
            .max_height(500.0)
            .fit_to_original_size(1.0)
            .center(ui);

        let gpio = rp2350.gpio.borrow();
        for i in 0..30 {
            let pin_state = gpio.pin_state(i);
            pin_state_ui(ui, i, &pin_state);
        }

        // TODO
    }
}

fn pin_state_ui(ui: &mut egui::Ui, index: u8, pin_state: &PinState) {
    ui.label(format!("Pin {}: ", index));
    match pin_state {
        PinState::Output(output_state, function_select) => {
            let text_out = match output_state {
                OutputState::High => "High",
                OutputState::Low => "Low",
            };

            let text_func = format!("{:?}", function_select);

            ui.horizontal(|ui| {
                ui.label(text_out);
                ui.label(text_func);
            });

            // egui::Frame::new()
            //     .corner_radius(10)
            //     .inner_margin(Margin::symmetric(6, 4))
            //     .fill(egui::Color32::from_rgb(0x00, 0x7f, 0x7f))
            //     .show(ui, |ui| {
            //         ui.monospace(RichText::new("RISC-V").strong().color(egui::Color32::WHITE));
            //     });
        }
        PinState::Input(input_state) => {
            let color = match input_state {
                InputState::PullUp => egui::Color32::from_black_alpha(0),
                InputState::PullDown => egui::Color32::from_black_alpha(0),
                InputState::Floating => egui::Color32::from_black_alpha(0),
                InputState::BusKeeper => egui::Color32::from_black_alpha(0),
            };
            ui.label(format!("{:?}", input_state));
        }
    }
}
