use super::Rp2350Component;
use crate::widgets::DisplayMode;
use egui::Margin;
use egui::RichText;
use rp2350::processor::cortex_m33::CortexM33;
use rp2350::processor::hazard3::Hazard3;
use rp2350::processor::hazard3::Registers as Hazard3Registers;
use rp2350::processor::Rp2350Core;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
struct RegisterOption {
    // show: bool,
    display_mode: DisplayMode,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct ProcessorCore<const T: usize> {
    registers: [RegisterOption; 32],
    show_with_naming_convention: bool,
}

impl<const T: usize> Rp2350Component for ProcessorCore<T> {
    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading(format!("Processor Core {}", T));

        match rp2350.processor[T] {
            Rp2350Core::Arm(ref processor) => self.ui_arm(ui, processor),
            Rp2350Core::RiscV(ref processor) => self.ui_riscv(ui, processor),
        }
    }
}

impl<const T: usize> ProcessorCore<T> {
    fn ui_arm(&mut self, ui: &mut egui::Ui, _cortex_m33: &CortexM33) {
        ui.heading("ARM Cortex-M33");
        // TODO
    }

    fn ui_riscv(&mut self, ui: &mut egui::Ui, hazard3: &Hazard3) {
        egui::Grid::new("ProcessorInfo")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Type");

                egui::Frame::new()
                    .corner_radius(10)
                    .inner_margin(Margin::symmetric(6, 4))
                    .fill(egui::Color32::from_rgb(0x00, 0x7f, 0x7f))
                    .show(ui, |ui| {
                        ui.monospace(RichText::new("RISC-V").strong().color(egui::Color32::WHITE));
                    });

                ui.end_row();

                ui.label("Executed");
                ui.label(format!("{} instructions", 0));

                ui.end_row();

                ui.label("IPC");
                ui.label(format!("{}", 0));

                ui.end_row();

                ui.label("PC");
                ui.label(format!("0x{:08x}", 0));
            });

        ui.add_space(12.0);

        self.hazard3_registers_ui(ui, &hazard3.registers);
    }

    fn hazard3_registers_ui(&mut self, ui: &mut egui::Ui, registers: &Hazard3Registers) {
        ui.heading("Registers");

        // option to show with naming convention
        ui.checkbox(
            &mut self.show_with_naming_convention,
            "Show with naming convention",
        );

        for (reg_opt, index) in self.registers.iter_mut().zip(0..) {
            let name = riscv_register_name(index, self.show_with_naming_convention);
            let value = registers.read(index);
            ui.add(register_ui(name, value, &mut reg_opt.display_mode));
        }
    }
}

fn register_ui(
    mut name: String,
    value: u32,
    display_mode: &mut DisplayMode,
) -> impl egui::Widget + '_ {
    // pad the name to 4 characters
    while name.len() < 4 {
        name.push(' ');
    }

    move |ui: &mut egui::Ui| {
        ui.horizontal(|ui| {
            ui.monospace(name);
            ui.add_space(6.0);
            ui.add(display_mode.bin_dec_hex_char());
            ui.add_space(6.0);
            ui.monospace(display_mode.fmt_u32(value));
        })
        .response
    }
}

fn riscv_register_name(register: u8, with_convention: bool) -> String {
    if !with_convention {
        return format!("x{register}");
    }

    match register {
        0 => "zero".to_string(),
        1 => "ra".to_string(),
        2 => "sp".to_string(),
        3 => "gp".to_string(),
        4 => "tp".to_string(),
        5 => "t0".to_string(),
        6 => "t1".to_string(),
        7 => "t2".to_string(),
        8 => "s0".to_string(),
        9 => "s1".to_string(),
        10 => "a0".to_string(),
        11 => "a1".to_string(),
        12 => "a2".to_string(),
        13 => "a3".to_string(),
        14 => "a4".to_string(),
        15 => "a5".to_string(),
        16 => "a6".to_string(),
        17 => "a7".to_string(),
        18 => "s2".to_string(),
        19 => "s3".to_string(),
        20 => "s4".to_string(),
        21 => "s5".to_string(),
        22 => "s6".to_string(),
        23 => "s7".to_string(),
        24 => "s8".to_string(),
        25 => "s9".to_string(),
        26 => "s10".to_string(),
        27 => "s11".to_string(),
        28 => "t3".to_string(),
        29 => "t4".to_string(),
        30 => "t5".to_string(),
        31 => "t6".to_string(),
        _ => "unknown".to_string(),
    }
}
