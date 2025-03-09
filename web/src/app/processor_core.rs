use super::Rp2350Component;
use crate::widgets::DisplayMode;
use rp2350::Rp2350;
use std::collections::HashMap;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ProcessorCore<const T: usize> {
    registers: HashMap<u8, DisplayMode>,
}

impl<const T: usize> Rp2350Component for ProcessorCore<T> {
    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading(format!("Processor Core {}", T));

        egui::Grid::new("MemoryView")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| self.show_info_grid(ui, rp2350));

        ui.add_space(12.0);

        self.registers_ui(ui, rp2350);
    }
}

impl<const T: usize> ProcessorCore<T> {
    fn show_info_grid(&mut self, ui: &mut egui::Ui, _rp2350: &mut Rp2350) {
        ui.label("Type");
        ui.label("RISC-V");

        ui.end_row();

        ui.label("Executed");
        ui.label(format!("{} instructions", 0));

        ui.end_row();

        ui.label("IPC");
        ui.label(format!("{}", 0));

        ui.end_row();

        ui.label("PC");
        ui.label(format!("0x{:08x}", 0));
    }

    fn registers_ui(&mut self, ui: &mut egui::Ui, _rp2350: &mut Rp2350) {
        ui.heading("Registers");
    }
}
