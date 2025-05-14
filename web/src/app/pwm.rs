/**
 * @file app/pwm.rs
 * @author Nguyen Le Duy
 * @date 10/05/2025
 * @brief View window for the PWM peripheral
 */
use super::Rp2350Component;
use egui::collapsing_header::CollapsingState;
use rp2350::peripherals::pwm::channel::DivMode;
use rp2350::peripherals::pwm::NOF_CHANNEL;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Pwm {
    // None
}

impl Rp2350Component for Pwm {
    const NAME: &'static str = "PWM";

    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading("PWM");

        let Ok(pwm) = rp2350.bus.peripherals.pwm.try_borrow() else {
            ui.label("PWM peripheral is not available");
            return;
        };

        for i in 0..NOF_CHANNEL {
            CollapsingState::load_with_default_open(
                ui.ctx(),
                ui.make_persistent_id(format!("pwm_channel_{}", i)),
                i == 0,
            )
            .show_header(ui, |ui| {
                ui.label(format!("Channel {}", i));
            })
            .body(|ui| {
                let ref channel = pwm.channels[i];

                egui::Grid::new(format!("pwm_channel_{}", i))
                    .num_columns(2)
                    .spacing([40.0, 6.0])
                    .striped(false)
                    .show(ui, |ui| {
                        ui.label("Enabled");
                        ui.label(if channel.is_enabled() { "Yes" } else { "No" });
                        ui.end_row();

                        ui.label("Mode");
                        ui.label(match channel.divmode() {
                            DivMode::Div => "Div",
                            DivMode::Level => "Level",
                            DivMode::Rise => "Rise",
                            DivMode::Fall => "Fall",
                        });
                        ui.end_row();

                        ui.label("Counter");
                        ui.label(format!("{}", channel.ctr));
                        ui.end_row();

                        ui.label("Top");
                        ui.label(format!("{}", channel.top));
                        ui.end_row();

                        ui.label("Counter compare A");
                        ui.label(format!("{}", channel.cc as u16));
                        ui.end_row();

                        ui.label("Counter compare B");
                        ui.label(format!("{}", (channel.cc >> 16) as u16));
                        ui.end_row();

                        ui.label("Divisor");
                        ui.label(format!("{}.{}", channel.div >> 4, channel.div & 0x0F));
                        ui.end_row();

                        ui.label("PH correct");
                        ui.label(if channel.ph_correct() { "Yes" } else { "No" });
                        ui.end_row();

                        ui.label("Invert A");
                        ui.label(if channel.invert_a() { "Yes" } else { "No" });
                        ui.end_row();

                        ui.label("Invert B");
                        ui.label(if channel.invert_b() { "Yes" } else { "No" });
                        ui.end_row();
                    });
            });
        }
    }
}
