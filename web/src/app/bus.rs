use crate::tracker::BusEvent;

/**
 * @file app/bus.rs
 * @author Nguyen Le Duy
 * @date 12/05/2025
 * @brief View window for the Bus system
 */
use super::Rp2350Component;
use egui::RichText;
use egui_extras::{Column, TableBuilder};
use rp2350::common::{DataSize, Requestor};
use rp2350::Rp2350;
use std::rc::Rc;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Bus {
    // None
}

impl Rp2350Component for Bus {
    const NAME: &'static str = "Bus";

    fn ui_with_tracker(
        &mut self,
        ui: &mut egui::Ui,
        _rp2350: &mut Rp2350,
        tracker: Rc<crate::Tracker>,
    ) {
        ui.heading("Bus Events");
        let tracker = tracker.borrow();
        let ref bus = tracker.bus;

        egui::Grid::new("Bus")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                // TODO
                ui.end_row();
            });

        ui.add_space(12.0);

        TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            // .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .column(Column::exact(100.0))
            .column(Column::exact(100.0))
            .column(Column::exact(100.0))
            .column(Column::exact(100.0))
            .column(Column::exact(100.0))
            .min_scrolled_height(200.0)
            .max_scroll_height(200.0) // 10 rows
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.label(RichText::new("Mode").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Requestor").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Address").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Size").strong());
                });
                header.col(|ui| {
                    ui.label(RichText::new("Value").strong());
                });
            })
            .body(|body| {
                body.rows(20.0, bus.events.len(), |mut row| {
                    match bus.events.get(row.index()) {
                        Some(BusEvent::Read {
                            requestor,
                            address,
                            size,
                        }) => {
                            row.col(|ui| {
                                ui.label("Read");
                            });
                            row.col(|ui| {
                                ui.label(match requestor {
                                    Requestor::Proc0 => "Core 0".to_string(),
                                    Requestor::Proc1 => "Core 1".to_string(),
                                    Requestor::DmaR => "DMA Read".to_string(),
                                    Requestor::DmaW => "DMA Write".to_string(),
                                });
                            });
                            row.col(|ui| {
                                ui.label(format!("{:#010x}", address));
                            });
                            row.col(|ui| {
                                ui.label(match size {
                                    DataSize::Byte => "8 bits",
                                    DataSize::HalfWord => "16 bits",
                                    DataSize::Word => "32 bits",
                                });
                            });
                            row.col(|_ui| {});
                        }

                        Some(BusEvent::Write {
                            requestor,
                            address,
                            value,
                            size,
                        }) => {
                            row.col(|ui| {
                                ui.label("Write");
                            });
                            row.col(|ui| {
                                ui.label(match requestor {
                                    Requestor::Proc0 => "Core 0".to_string(),
                                    Requestor::Proc1 => "Core 1".to_string(),
                                    Requestor::DmaR => "DMA Read".to_string(),
                                    Requestor::DmaW => "DMA Write".to_string(),
                                });
                            });
                            row.col(|ui| {
                                ui.label(format!("{:#010x}", address));
                            });
                            row.col(|ui| {
                                ui.label(match size {
                                    DataSize::Byte => "8 bits",
                                    DataSize::HalfWord => "16 bits",
                                    DataSize::Word => "32 bits",
                                });
                            });
                            row.col(|ui| {
                                ui.label(format!("{:#010x}", value));
                            });
                        }

                        None => return,
                    }
                });
            });
    }
}
