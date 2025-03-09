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
    pub fn bin_dec_hex_char(&mut self, ui: &mut egui::Ui) {
        self.ui(
            ui,
            &[
                (Self::Binary, "Bin"),
                (Self::Decimal, "Dec"),
                (Self::Hexadecimal, "Hex"),
                (Self::Character, "Char"),
            ],
        );
    }

    pub fn bin_hex(&mut self, ui: &mut egui::Ui) {
        self.ui(ui, &[(Self::Binary, "Bin"), (Self::Hexadecimal, "Hex")]);
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, modes: &[(Self, &str)]) {
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
        });
    }

    fn active_color(&self) -> egui::Color32 {
        egui::Color32::from_rgb(0x00, 0x7f, 0x7f)
    }

    // take a generic type T that can be formatted as binary and hex
    pub fn display<T>(&self, value: T) -> String
    where
        T: std::fmt::UpperHex + std::fmt::Binary + std::fmt::Display + Into<u32>,
    {
        match self {
            DisplayMode::Binary => format!("{:08b}", value),
            DisplayMode::Decimal => format!("{:03}", value),
            DisplayMode::Hexadecimal => format!("{:02X}", value),
            DisplayMode::Character => {
                // convert the value to a char
                let c = char::from_u32(value.into()).unwrap();
                format!("{}", c.escape_default())
            }
        }
    }
}
