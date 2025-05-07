/**
 * @file app/field.rs
 * @author Nguyen Le Duy
 * @date 07/05/2025
 * @brief View schematic and field of Raspberry Pi Pico 2
 */
use super::Rp2350Component;
use egui::Margin;
use egui::RichText;
use rp2350::gpio::*;
use rp2350::Rp2350;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Field {
    show_schematic: bool,
    scene_rect: egui::Rect,
    schematic_rect: egui::Rect,
}

impl Default for Field {
    fn default() -> Self {
        Self {
            show_schematic: false,
            scene_rect: egui::Rect::ZERO,
            schematic_rect: egui::Rect::ZERO,
        }
    }
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
        egui::Scene::new()
            .zoom_range(0.1..=3.0)
            .show(ui, &mut self.schematic_rect, |ui| {
                ui.add(
                    egui::Image::new(egui::include_image!("../../assets/pico2_schematic.webp"))
                        .alt_text("Raspberry Pi Pico 2 Schematic")
                        .maintain_aspect_ratio(true)
                        .max_height(500.0)
                        .fit_to_original_size(1.0),
                )
            });
    }

    fn field_ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        egui::Scene::new()
            .zoom_range(0.1..=3.0)
            .show(ui, &mut self.scene_rect, |ui| {
                ui.horizontal(|ui| {
                    let gpio = rp2350.gpio.borrow();

                    draw_gpio_state(ui, &gpio, true);
                    draw_raspberry_pi_pico2(ui, gpio.pin_state(25).is_high());
                    draw_gpio_state(ui, &gpio, false);
                });
            });
    }
}

#[rustfmt::skip]
fn draw_gpio_state(ui: &mut egui::Ui, gpio: &GpioController, is_left: bool) {
    ui.vertical(|ui| {
        if is_left {
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left)); // Dummy pin for alignment
            ui.add(pin_state(gpio.pin_state(0), is_left));
            ui.add(pin_state(gpio.pin_state(1), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add(pin_state(gpio.pin_state(2), is_left));
            ui.add(pin_state(gpio.pin_state(3), is_left));
            ui.add(pin_state(gpio.pin_state(4), is_left));
            ui.add(pin_state(gpio.pin_state(5), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add(pin_state(gpio.pin_state(6), is_left));
            ui.add(pin_state(gpio.pin_state(7), is_left));
            ui.add(pin_state(gpio.pin_state(8), is_left));
            ui.add(pin_state(gpio.pin_state(9), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add(pin_state(gpio.pin_state(10), is_left));
            ui.add(pin_state(gpio.pin_state(11), is_left));
            ui.add(pin_state(gpio.pin_state(12), is_left));
            ui.add(pin_state(gpio.pin_state(13), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add(pin_state(gpio.pin_state(14), is_left));
            ui.add(pin_state(gpio.pin_state(15), is_left));
        } else {
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left)); // Dummy pin for alignment
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add(pin_state(gpio.pin_state(28), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add(pin_state(gpio.pin_state(27), is_left));
            ui.add(pin_state(gpio.pin_state(26), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add(pin_state(gpio.pin_state(22), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add(pin_state(gpio.pin_state(21), is_left));
            ui.add(pin_state(gpio.pin_state(20), is_left));
            ui.add(pin_state(gpio.pin_state(19), is_left));
            ui.add(pin_state(gpio.pin_state(18), is_left));
            ui.add_visible(false, pin_state(PinState::Input(InputState::Floating), is_left));
            ui.add(pin_state(gpio.pin_state(17), is_left));
            ui.add(pin_state(gpio.pin_state(16), is_left));
        }
    });
}

fn draw_raspberry_pi_pico2(ui: &mut egui::Ui, _led_on: bool) {
    let _img_rect = ui
        .add(
            egui::Image::new(egui::include_image!("../../assets/pico2.webp"))
                .alt_text("Raspberry Pi Pico 2")
                .maintain_aspect_ratio(true)
                .max_height(520.0)
                .fit_to_original_size(1.0),
        )
        .rect;
}

fn pin_state(pin_state: PinState, is_left: bool) -> impl egui::Widget + 'static {
    move |ui: &mut egui::Ui| match pin_state {
        PinState::Output(output_state, function_select) => {
            let text_out = match output_state {
                OutputState::High => "HIGH",
                OutputState::Low => "LOW ",
            };

            let text_func = format!("{:?}", function_select);

            ui.horizontal(|ui| {
                if is_left {
                    ui.add(pin_frame(text_out, egui::Color32::MAGENTA));
                    ui.add(pin_frame(
                        text_func,
                        egui::Color32::from_hex("#151313").unwrap(),
                    ));
                } else {
                    ui.add(pin_frame(
                        text_func,
                        egui::Color32::from_hex("#151313").unwrap(),
                    ));
                    ui.add(pin_frame(text_out, egui::Color32::MAGENTA));
                }
            })
            .response
        }
        PinState::Input(input_state) => {
            let state = match input_state {
                InputState::PullUp => "  PULL UP ",
                InputState::PullDown => " PULL DOWN",
                InputState::Floating => " FLOATING ",
                InputState::BusKeeper => "BUS KEEPER",
            };

            ui.horizontal(|ui| {
                if is_left {
                    ui.add(pin_frame("IN", egui::Color32::from_hex("#4f56e9").unwrap()));
                    ui.add(pin_frame(
                        state,
                        egui::Color32::from_hex("#151313").unwrap(),
                    ));
                } else {
                    ui.add(pin_frame(
                        state,
                        egui::Color32::from_hex("#151313").unwrap(),
                    ));
                    ui.add(pin_frame("IN", egui::Color32::from_hex("#4f56e9").unwrap()));
                }
            })
            .response
        }
    }
}

fn pin_frame(
    text: impl AsRef<str> + 'static,
    background_color: egui::Color32,
) -> impl egui::Widget + 'static {
    move |ui: &mut egui::Ui| {
        egui::Frame::new()
            .inner_margin(Margin::symmetric(6, 4))
            .outer_margin(Margin::symmetric(0, 0))
            .fill(background_color)
            .show(ui, |ui| {
                ui.monospace(
                    RichText::new(text.as_ref())
                        .strong()
                        .color(egui::Color32::WHITE),
                );
            })
            .response
    }
}
