/**
 * @file app/timer.rs
 * @author Nguyen Le Duy
 * @date 14/04/2025
 * @brief View window for the Timer peripheral
 */
use super::Rp2350Component;
use egui::{RichText, ScrollArea};
use rp2350::peripherals::timer::CountSource;
use rp2350::Rp2350;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Timer<const IDX: usize> {
    // None
}

impl<const IDX: usize> Rp2350Component for Timer<IDX> {
    const NAME: &'static str = "Timer";

    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading(format!("Timer {IDX}"));
        match IDX {
            0 => view_timer(ui, &rp2350.bus.peripherals.timer0),
            1 => view_timer(ui, &rp2350.bus.peripherals.timer1),
            _ => unreachable!(),
        }
    }
}

fn view_timer<const IDX: usize>(
    ui: &mut egui::Ui,
    timer: &Rc<RefCell<rp2350::peripherals::Timer<IDX>>>,
) {
    let timer = timer.borrow();
    egui::Grid::new(format!("Uart {IDX}"))
        .num_columns(2)
        .spacing([40.0, 6.0])
        .striped(false)
        .show(ui, |ui| {
            ui.label("Current Counter");
            ui.label(format!("{}", timer.counter));
            ui.end_row();

            ui.label("Paused");
            if timer.is_paused {
                ui.label("Yes");
            } else {
                ui.label("No");
            }
            ui.end_row();

            ui.label("Locked");
            if timer.is_locked {
                ui.label("Yes");
            } else {
                ui.label("No");
            }
            ui.end_row();

            ui.label("Counting speed");
            ui.label(match timer.source {
                CountSource::_1MHz => "1 MHz",
                CountSource::ClkSys => "150 MHz",
            });
            ui.end_row();

            for (i, alarm) in timer.alarm.iter().enumerate() {
                ui.label(format!("Alarm {i}"));
                let mut text = format!("{}", alarm.time);
                if !alarm.armed {
                    text.push_str(" (not armed)");
                }

                ui.label(text);
                ui.end_row();
            }
        });
}
