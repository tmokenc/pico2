/**
 * @file app/processor_core.rs
 * @author Nguyen Le Duy
 * @date 04/05/2025
 * @brief View window for the processor core
 */
use super::Rp2350Component;
use crate::tracker::ProcessorTracker;
use crate::widgets::DisplayMode;
use egui::collapsing_header::CollapsingState;
use egui::Margin;
use egui::RichText;
use egui_extras::Column;
use egui_extras::TableBuilder;
use rp2350::processor::cortex_m33::CortexM33;
use rp2350::processor::hazard3::Registers as Hazard3Registers;
use rp2350::processor::hazard3::{Hazard3, State as Hazard3State};
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
    const NAME: &'static str = name::<T>();

    fn ui_with_tracker(
        &mut self,
        ui: &mut egui::Ui,
        rp2350: &mut Rp2350,
        tracker: std::rc::Rc<crate::Tracker>,
    ) {
        ui.heading(format!("Processor Core {}", T));

        let track = tracker.borrow();
        let ref processor_tracker = track.processor[T];

        // Show processor details
        match rp2350.processor[T] {
            Rp2350Core::Arm(ref processor) => self.ui_arm(ui, processor),
            Rp2350Core::RiscV(ref processor) => self.ui_riscv(ui, processor, processor_tracker),
        }

        // Show processor tracker
        ui.add_space(12.0);
        show_processor_tracker::<T>(ui, processor_tracker);
    }
}

const fn name<const T: usize>() -> &'static str {
    if T == 0 {
        "Processor Core 0"
    } else {
        "Processor Core 1"
    }
}

impl<const T: usize> ProcessorCore<T> {
    fn ui_arm(&mut self, ui: &mut egui::Ui, _cortex_m33: &CortexM33) {
        ui.heading("ARM Cortex-M33");
        // TODO
    }

    fn ui_riscv(&mut self, ui: &mut egui::Ui, hazard3: &Hazard3, tracker: &ProcessorTracker) {
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

                ui.label("State");
                ui.label(match hazard3.state {
                    Hazard3State::Wfi => "WFI".to_owned(),
                    Hazard3State::Stall(cycles, _) => format!("Stall for ({cycles} cycles)"),
                    Hazard3State::Normal => "Running".to_owned(),
                    Hazard3State::Sleep(_) => "Sleep".to_owned(),
                    Hazard3State::BusWaitStore(_) => "Bus Wait Store".to_owned(),
                    Hazard3State::BusWaitLoad(rd, _) => format!("Bus Wait Load (rd: x{rd})"),
                    Hazard3State::Atomic { .. } => "Executing atomic instruction".to_owned(),
                });
                ui.end_row();

                // let excecuted_inst = hazard3.csrs.minstret;
                let executed_cycles = tracker.ticks;

                ui.label("Executed");
                ui.label(format!("{}", tracker.inst_count));
                ui.end_row();

                ui.label("IPC");
                ui.label(format!(
                    "{}",
                    (tracker.inst_count as f64) / (executed_cycles as f64)
                ));
                ui.end_row();

                ui.label("PC");
                ui.label(format!("0x{:08x}", hazard3.pc));
                ui.end_row();
            });

        ui.add_space(12.0);

        CollapsingState::load_with_default_open(
            ui.ctx(),
            ui.make_persistent_id(register_name::<T>()),
            true,
        )
        .show_header(ui, |ui| {
            ui.heading("Registers");
        })
        .body(|ui| {
            self.hazard3_registers_ui(ui, &hazard3.registers);
        });
    }

    fn hazard3_registers_ui(&mut self, ui: &mut egui::Ui, registers: &Hazard3Registers) {
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

const fn register_name<const T: usize>() -> &'static str {
    if T == 0 {
        "ProcessorCore0Registers"
    } else {
        "ProcessorCore1Registers"
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

// Helper to get the ID for the collapsing header
const fn tracker_name<const T: usize>() -> &'static str {
    if T == 0 {
        "ProcessorCore0Tracker"
    } else {
        "ProcessorCore1Tracker"
    }
}

// Helper to get the ID for the collapsing header
const fn log_name<const T: usize>() -> &'static str {
    if T == 0 {
        "ProcessorCore0Log"
    } else {
        "ProcessorCore1Log"
    }
}

fn show_processor_tracker<const T: usize>(ui: &mut egui::Ui, tracker: &ProcessorTracker) {
    CollapsingState::load_with_default_open(
        ui.ctx(),
        ui.make_persistent_id(tracker_name::<T>()),
        false,
    )
    .show_header(ui, |ui| {
        ui.heading("Last executed instructions");
    })
    .body(|ui| {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            // .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(100.0))
            .column(Column::exact(100.0))
            .column(Column::exact(100.0))
            .min_scrolled_height(200.0)
            .max_scroll_height(200.0) // 10 rows
            .stick_to_bottom(true)
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label(RichText::new("Name").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Code").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Address").strong());
                });
            })
            .body(|body| {
                body.rows(20.0, tracker.instruction_log.len(), |mut row| {
                    let instruction = &tracker.instruction_log[row.index()];
                    row.col(|ui| {
                        ui.label(instruction.name);
                    });
                    row.col(|ui| {
                        ui.label(format!("{:08x}", instruction.code));
                    });
                    row.col(|ui| {
                        ui.label(format!("0x{:08x}", instruction.address));
                    });
                });
            });
    });

    ui.add_space(12.0);

    CollapsingState::load_with_default_open(
        ui.ctx(),
        ui.make_persistent_id(log_name::<T>()),
        false,
    )
    .show_header(ui, |ui| {
        ui.heading("Instruction count");
    })
    .body(|ui| {
        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            // .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(100.0))
            .column(Column::exact(50.0))
            .min_scrolled_height(200.0)
            .max_scroll_height(200.0) // 10 rows
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label(RichText::new("Name").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Count").strong());
                });
            })
            .body(|body| {
                body.rows(20.0, tracker.instruction_count.len(), |mut row| {
                    let (name, count) = tracker.instruction_count.iter().nth(row.index()).unwrap();
                    row.col(|ui| {
                        ui.label(*name);
                    });
                    row.col(|ui| {
                        ui.label(format!("{}", count));
                    });
                });
            });
    });
}
