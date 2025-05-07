use crate::simulator::TaskCommand;
use api_types::Language;
use egui::ComboBox;
use futures::channel::mpsc::Sender;

pub struct Example {
    pub name: &'static str,
    pub code: &'static str,
}

pub const EXAMPLES: &[Example] = &[
    Example {
        name: "UART",
        code: include_str!("../../assets/examples/uart.c"),
    },
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
        name: "Watchdog",
        code: include_str!("../../assets/examples/watchdog.c"),
    },
];

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeEditor {
    pub language: Language,
    pub code: String,
    pub skip_bootrom: bool,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            language: Language::C,
            code: String::from(EXAMPLES[0].code),
            skip_bootrom: true,
        }
    }
}

impl CodeEditor {
    pub fn ui(&mut self, ui: &mut egui::Ui, tx: &mut Sender<TaskCommand>) {
        let Self {
            language,
            code,
            skip_bootrom,
        } = self;

        ui.horizontal(|ui| {
            ui.label("Language");
            ComboBox::from_label("")
                .selected_text(format!("{:?}", language))
                .show_ui(ui, |ui| {
                    ui.selectable_value(language, Language::C, "C");
                    // Add more languages here if needed
                });

            ui.add_space(30.0);

            if ui
                .button("Flash")
                .on_hover_text("Flash the code to the Pico2")
                .clicked()
            {
                let _ = tx.try_send(TaskCommand::FlashCode(
                    language.clone(),
                    code.clone(),
                    *skip_bootrom,
                ));
            }

            ui.checkbox(skip_bootrom, "Skip Bootrom")
                .on_hover_text("Skip the bootrom code");
        });

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
