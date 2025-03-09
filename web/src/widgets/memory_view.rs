//! File: memory_view.rs
//! Author: Nguyen Le Duy
//! Description: Memory view widget for the RP2350 emulator
//! TODO Search functionality

use egui_extras::{Column, TableBuilder};
use rp2350::memory::GenericMemory;

use super::DisplayMode;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct MemoryView<const OFFSET: usize> {
    bytes_per_row: usize,
    address_buffer: String,
    display_mode: DisplayMode,
}

impl<const OFFSET: usize> Default for MemoryView<OFFSET> {
    fn default() -> Self {
        Self {
            bytes_per_row: 16,
            address_buffer: String::new(),
            display_mode: DisplayMode::default(),
        }
    }
}

impl<const OFFSET: usize> MemoryView<OFFSET> {
    pub fn ui<const N: usize>(&mut self, ui: &mut egui::Ui, mem: &GenericMemory<N>) {
        let mut address = None;

        egui::Grid::new("MemoryView")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| self.show_info_grid::<N>(ui, &mut address));

        ui.add_space(12.0);

        egui::ScrollArea::horizontal().show(ui, |ui| {
            self.show_table_mem(ui, mem, address);
        });
    }

    fn show_info_grid<const N: usize>(&mut self, ui: &mut egui::Ui, address: &mut Option<u32>) {
        ui.label("Size:");
        ui.label(format_memory_length(N));
        ui.end_row();

        ui.label("Offset");
        ui.label(format!("0x{:08X}", OFFSET));
        ui.end_row();

        ui.label("Go to address:");
        ui.horizontal(|ui| {
            let res = ui
                .text_edit_singleline(&mut self.address_buffer)
                .on_hover_text("Enter an address to jump to");

            if (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                || ui.button("Go").clicked()
            {
                let address_str = self.address_buffer.trim_start_matches("0x");
                *address = u32::from_str_radix(address_str, 16).ok();
            }
        });

        ui.end_row();

        ui.label("Bytes per row:");
        // radio buttons for bytes per row 4 8 16
        ui.horizontal(|ui| {
            ui.radio_value(&mut self.bytes_per_row, 4, "4");
            ui.radio_value(&mut self.bytes_per_row, 8, "8");
            ui.radio_value(&mut self.bytes_per_row, 16, "16");
        });

        ui.end_row();

        ui.label("Display mode:");
        self.display_mode.bin_hex(ui);
        ui.end_row();
    }

    fn show_table_mem(&mut self, ui: &mut egui::Ui, mem: &[u8], address: Option<u32>) {
        let height = ui.available_height();
        let num_rows = (mem.len() + self.bytes_per_row - 1) / self.bytes_per_row;

        let text_height = egui::TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let mut table = TableBuilder::new(ui)
            .striped(true)
            .resizable(false)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::auto())
            .column(Column::auto())
            .column(Column::remainder())
            .min_scrolled_height(0.0)
            .max_scroll_height(height);

        if let Some(address) = address {
            let row = address as usize / self.bytes_per_row;
            table = table.scroll_to_row(row, None);
        }

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label("Offset");
                });
                header.col(|ui| {
                    ui.label("Value");
                });
                header.col(|ui| {
                    ui.label("ASCII");
                });
            })
            .body(|body| {
                body.rows(text_height, num_rows, |mut row| {
                    let row_index = row.index();

                    row.col(|ui| {
                        ui.label(format!(
                            "{:08X}  ",
                            OFFSET + (row_index * self.bytes_per_row)
                        ));
                    });

                    // Hex
                    row.col(|ui| {
                        let mut string = String::with_capacity(self.bytes_per_row * 3);

                        for col_index in 0..self.bytes_per_row {
                            let index = row_index * self.bytes_per_row + col_index;
                            if index < mem.len() {
                                let fmt = self.display_mode.display(mem[index]);
                                string.push_str(&fmt);
                                string.push(' ');
                            }
                        }

                        ui.monospace(string);
                    });

                    // ASCII
                    row.col(|ui| {
                        let mut string = String::with_capacity(self.bytes_per_row);
                        for col_index in 0..self.bytes_per_row {
                            let index = row_index * self.bytes_per_row + col_index;
                            if index < mem.len() {
                                let c = mem[index];
                                let c = if c.is_ascii() && !c.is_ascii_control() {
                                    c as char
                                } else {
                                    '.'
                                };
                                string.push(c);
                            }
                        }

                        ui.monospace(string);
                    });
                })
            });
    }
}

fn format_memory_length(byte: usize) -> String {
    if byte < 1024 {
        format!("{} B", byte)
    } else if byte < 1024 * 1024 {
        format!("{} KiB", byte >> 10)
    } else {
        format!("{} MiB", byte >> 20)
    }
}
