/**
 * @file: display_mode.rs
 * @author: Nguyen Le Duy
 * @date 31/03/2025
 * @brief: Widget to display number in different formats
 */
use egui::{Label, RichText, Sense};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum DisplayMode {
    Binary,
    Decimal,
    #[default]
    Hexadecimal,
    Character,
}

impl DisplayMode {
    pub fn bin_dec_hex_char(&mut self) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| {
            self.ui(
                ui,
                &[
                    (Self::Binary, "Bin"),
                    (Self::Decimal, "Dec"),
                    (Self::Hexadecimal, "Hex"),
                    (Self::Character, "Char"),
                ],
            )
        }
    }

    pub fn bin_hex(&mut self) -> impl egui::Widget + '_ {
        move |ui: &mut egui::Ui| self.ui(ui, &[(Self::Binary, "Bin"), (Self::Hexadecimal, "Hex")])
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, modes: &[(Self, &str)]) -> egui::Response {
        ui.horizontal(|ui| {
            for (mode, label) in modes {
                let mut text = RichText::new(*label);

                if mode == self {
                    text = text.strong().color(self.active_color());
                }

                let response = ui.add(Label::new(text).sense(Sense::click()));

                if response.clicked() {
                    *self = *mode;
                }
            }
        })
        .response
    }

    fn active_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(0x00, 0x7f, 0x7f)
    }

    pub fn fmt_u8(&self, value: u8) -> String {
        match self {
            DisplayMode::Binary => format!("{:08b}", value),
            DisplayMode::Decimal => format!("{:03}", value),
            DisplayMode::Hexadecimal => format!("{:02X}", value),
            DisplayMode::Character => {
                // convert the value to a char
                let c = value as char;
                format!("{}", c.escape_default())
            }
        }
    }

    pub fn fmt_u32(&self, value: u32) -> String {
        match self {
            DisplayMode::Binary => format!("{:032b}", value),
            DisplayMode::Decimal => format!("{:010}", value),
            DisplayMode::Hexadecimal => format!("{:08X}", value),
            DisplayMode::Character => {
                // convert the value to a char
                match char::from_u32(value) {
                    Some(c) => format!("{}", c.escape_default()),
                    None => format!("0x{:08X}", value),
                }
            }
        }
    }
}
