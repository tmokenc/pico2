use crate::simulator::TaskCommand;
use api_types::Language;
use egui::{Color32, ComboBox};
use futures::channel::mpsc::Sender;

struct Example {
    name: &'static str,
    code: &'static str,
}

const EXAMPLES: &[Example] = &[
    Example {
        name: "Blink",
        code: include_str!("../../assets/examples/blink.c"),
    },
    Example {
        name: "DMA",
        code: include_str!("../../assets/examples/dma.c"),
    },
    Example {
        name: "GPIO",
        code: include_str!("../../assets/examples/gpio.c"),
    },
    Example {
        name: "Multicore",
        code: include_str!("../../assets/examples/multicore.c"),
    },
    Example {
        name: "Multicore FIFO IRQ",
        code: include_str!("../../assets/examples/multicore_fifo_irq.c"),
    },
    Example {
        name: "PWM",
        code: include_str!("../../assets/examples/pwm.c"),
    },
    Example {
        name: "SPI",
        code: include_str!("../../assets/examples/spi.c"),
    },
    Example {
        name: "Timer",
        code: include_str!("../../assets/examples/timer.c"),
    },
    Example {
        name: "UART",
        code: include_str!("../../assets/examples/uart.c"),
    },
    Example {
        name: "Watchdog",
        code: include_str!("../../assets/examples/watchdog.c"),
    },
];

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeEditor {
    pub language: Language,
    pub code: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            language: Language::C,
            code: String::from(EXAMPLES[0].code),
        }
    }
}

impl CodeEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui, tx: &mut Sender<TaskCommand>) {
        let Self { language, code } = self;

        ComboBox::from_label("Language")
            .selected_text(format!("{:?}", language))
            .show_ui(ui, |ui| {
                ui.selectable_value(language, Language::C, "C");
                // Add more languages here if needed
            });

        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Syntax highlighting powered by ");
            ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
            ui.label(".");
        });

        if ui
            .button("Flash")
            .on_hover_text("Flash the code to the Pico2")
            .clicked()
        {
            let _ = tx.try_send(TaskCommand::FlashCode(language.clone(), code.clone()));
        }

        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                string,
                &format!("{:?}", language),
            );
            layout_job.wrap.max_width = wrap_width;
            ui.fonts(|f| f.layout_job(layout_job))
        };

        egui::ScrollArea::vertical().show(ui, |ui| {
            let size = ui.available_size_before_wrap();

            ui.add_sized(
                size,
                egui::TextEdit::multiline(code)
                    .font(egui::TextStyle::Monospace) // for cursor height
                    .code_editor()
                    .lock_focus(true)
                    .desired_width(f32::INFINITY)
                    .layouter(&mut layouter),
            );
        });
    }
}
