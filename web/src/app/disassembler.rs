/**
 * @file app/disassembler.rs
 * @author Nguyen Le Duy
 * @date 23/04/2025
 * @brief Disassembler view window
 */

const ARM_BOOTROM_DISASSEMBLY: &str = include_str!("../../assets/arm-bootrom.dis");
const RISCV_BOOTROM_DISASSEMBLY: &str = include_str!("../../assets/riscv-bootrom.dis");

use super::Rp2350Component;
use egui::RichText;
use egui_extras::{Column, TableBuilder};
use rp2350::Rp2350;
use std::collections::{HashMap, HashSet};

const COLOR_CORE0: egui::Color32 = egui::Color32::BLUE;
const COLOR_CORE1: egui::Color32 = egui::Color32::GREEN;
const COLOR_BOTH: egui::Color32 = egui::Color32::PURPLE;

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq)]
enum StickOption {
    Core0,
    Core1,
    None,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Disassembler {
    codes: Vec<String>,
    breakpoints: HashSet<u32>,
    pc_to_line_map: HashMap<u32, usize>,
    last_pc_core0: u32,
    last_pc_core1: u32,
    search_buffer: String,
    stick: StickOption,
}

impl Default for Disassembler {
    fn default() -> Self {
        let mut res = Self {
            codes: Vec::new(),
            breakpoints: HashSet::new(),
            pc_to_line_map: HashMap::new(),
            last_pc_core0: 0,
            last_pc_core1: 0,
            search_buffer: String::new(),
            stick: StickOption::Core0,
        };

        res.codes
            .extend(ARM_BOOTROM_DISASSEMBLY.lines().map(String::from));
        res.codes
            .extend(RISCV_BOOTROM_DISASSEMBLY.lines().map(String::from));
        res
    }
}

impl Disassembler {
    pub fn update_file(&mut self, file: &str) {
        let Self { codes, .. } = Self::default();
        self.codes = codes;
        self.codes.extend(file.lines().map(String::from));
        self.update_pc_to_line_map();
    }

    pub fn add_breakpoint(&mut self, addr: u32) {
        self.breakpoints.insert(addr);
    }

    pub fn remove_breakpoint(&mut self, addr: &u32) {
        self.breakpoints.remove(addr);
    }

    pub fn has_breakpoint(&self, addr: &u32) -> bool {
        self.breakpoints.contains(&addr)
    }

    pub fn clear_breakpoints(&mut self) {
        self.breakpoints.clear();
    }

    fn parse_addr(&self, line: &str) -> Option<u32> {
        if !line.contains(':') {
            return None;
        }

        let num = line.split(':').next().unwrap().trim_start();
        u32::from_str_radix(num, 16).ok()
    }

    fn update_pc_to_line_map(&mut self) {
        self.pc_to_line_map.clear();
        for (i, line) in self.codes.iter().enumerate() {
            if let Some(addr) = self.parse_addr(line) {
                self.pc_to_line_map.insert(addr, i);
            }
        }
    }
}

impl Rp2350Component for Disassembler {
    const NAME: &'static str = "Disassembler";

    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        let height = ui.available_height();
        let max_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let dark_mode = ui.visuals().dark_mode;
        let faded_color = ui.visuals().window_fill();
        let faded_color = |color: egui::Color32| -> egui::Color32 {
            use egui::Rgba;
            let t = if dark_mode { 0.95 } else { 0.8 };
            egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
        };

        let pc_core0 = rp2350.processor[0].get_pc();
        let pc_core1 = rp2350.processor[1].get_pc();

        // Controller

        let mut scroll_to = None;

        egui::Grid::new("Disassembler Control Info")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Jump to address");
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.search_buffer)
                        .on_hover_text("Enter an address in hexidecimal to jump to");

                    if ui.button("Go").clicked() {
                        let address_str = self.search_buffer.trim_start_matches("0x");
                        if let Ok(address) = u32::from_str_radix(address_str, 16) {
                            if let Some(&line_index) = self.pc_to_line_map.get(&address) {
                                scroll_to = Some(line_index);
                            }
                        }
                    }
                });
                ui.end_row();


                ui.label("Jump to PC");

                ui.horizontal(|ui| {
                    if ui.button("Core 0").clicked() {
                        if let Some(&line_index) = self.pc_to_line_map.get(&pc_core0) {
                            scroll_to = Some(line_index);
                        }
                    }

                    if ui.button("Core 1").clicked() {
                        if let Some(&line_index) = self.pc_to_line_map.get(&pc_core1) {
                            scroll_to = Some(line_index);
                        }
                    }
                });

                ui.end_row();

                ui.label("Stick to");
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.stick, StickOption::Core0, " Core 0 ");
                    ui.radio_value(&mut self.stick, StickOption::Core1, " Core 1 ");
                    ui.radio_value(&mut self.stick, StickOption::None, " None ");
                });
                ui.end_row();

                ui.label("Color References");
                #[rustfmt::skip]
                ui.horizontal(|ui| {
                    ui.label(RichText::new("CORE 0").monospace().background_color(faded_color(COLOR_CORE0)));
                    ui.add_space(4.0);
                    ui.label(RichText::new("CORE 1").monospace().background_color(faded_color(COLOR_CORE1)));
                    ui.add_space(4.0);
                    ui.label(RichText::new("BOTH").monospace().background_color(faded_color(COLOR_BOTH)));
                });
                ui.end_row();

                if ui.button("Clear Breakpoints").clicked() {
                    self.clear_breakpoints();
                }

            });

        match self.stick {
            StickOption::Core0 => {
                if self.last_pc_core0 != pc_core0 {
                    self.last_pc_core0 = pc_core0;
                    if let Some(&line_index) = self.pc_to_line_map.get(&pc_core0) {
                        scroll_to = Some(line_index);
                    }
                }
            }
            StickOption::Core1 => {
                if self.last_pc_core1 != pc_core1 {
                    self.last_pc_core1 = pc_core1;
                    if let Some(&line_index) = self.pc_to_line_map.get(&pc_core1) {
                        scroll_to = Some(line_index);
                    }
                }
            }
            StickOption::None => {}
        }

        ui.add_space(12.0);

        // View
        let mut table = TableBuilder::new(ui)
            .striped(false)
            .resizable(false)
            // breakpoint column
            .column(Column::auto())
            .column(Column::remainder())
            .animate_scrolling(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::LEFT))
            .min_scrolled_height(0.0)
            .max_scroll_height(height);

        if let Some(row) = scroll_to {
            table = table.scroll_to_row(row, None);
        }

        table.body(|body| {
            body.rows(max_height, self.codes.len(), |mut row| {
                let line_index = row.index();
                let bg_color = {
                    let index_core0 = self.pc_to_line_map.get(&pc_core0);
                    let index_core1 = self.pc_to_line_map.get(&pc_core1);
                    match (index_core0, index_core1) {
                        (Some(&index0), Some(&index1))
                            if line_index == index0 && line_index == index1 =>
                        {
                            Some(COLOR_BOTH)
                        }
                        (Some(&index0), _) if line_index == index0 => Some(COLOR_CORE0),
                        (_, Some(&index1)) if line_index == index1 => Some(COLOR_CORE1),
                        _ => None,
                    }
                };

                row.col(|ui| {
                    let Some(addr) = self.parse_addr(&self.codes[line_index]) else {
                        return;
                    };

                    let has_breakpoint = self.breakpoints.contains(&addr);
                    let center = ui.available_rect_before_wrap().center();
                    let radius = 6.0;

                    // Create a square bounding box around the circle
                    let rect =
                        egui::Rect::from_center_size(center, egui::Vec2::splat(radius * 2.0));

                    // Allocate space with interaction sense (click + hover)
                    let response = ui
                        .allocate_rect(rect, egui::Sense::HOVER | egui::Sense::CLICK)
                        .on_hover_ui(|ui| {
                            ui.label("Toggle Breakpoint");
                        });
                    let mut color = None;

                    // show tooltip

                    if has_breakpoint {
                        color = Some(egui::Color32::RED);
                    } else if response.hovered() {
                        color = Some(faded_color(egui::Color32::LIGHT_RED));
                    }

                    if let Some(color) = color {
                        ui.painter().circle_filled(center, radius, color);
                    }

                    if response.clicked() {
                        if has_breakpoint {
                            self.remove_breakpoint(&addr);
                        } else {
                            self.add_breakpoint(addr);
                        }
                    } else if response.hovered() {
                        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                    }
                });

                row.col(|ui| {
                    if let Some(bg_color) = bg_color {
                        ui.painter().rect_filled(
                            ui.available_rect_before_wrap(),
                            0.0,
                            faded_color(bg_color),
                        );
                    }

                    let line = &self.codes[line_index];
                    ui.monospace(line);
                });
            });
        });
    }
}
