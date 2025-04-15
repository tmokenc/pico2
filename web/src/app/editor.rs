use crate::api;
use egui::Color32;
use rp2350::simulator::Pico2;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeEditor {
    pub language: String,
    pub code: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            language: "c".into(),
            code: r#"// A very simple example\n\
int main() {
    return 0;
}
"#
            .into(),
        }
    }
}

impl CodeEditor {
    async fn flash_compile_code(&self, pico2: &mut Pico2) {
        let mut uf2: Vec<u8> = Vec::new();
        match api::compile(&self.code).await {
            Ok(_) => {}
            Err(why) => {
                // TODO
            }
        }

        match pico2.flash_uf2(&uf2) {
            Ok(_) => {
                // TODO
            }
            Err(why) => {
                // TODO
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self { language, code } = self;

        ui.horizontal(|ui| {
            ui.label("Language:");
            ui.text_edit_singleline(language);
        });
        ui.horizontal_wrapped(|ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            ui.label("Syntax highlighting powered by ");
            ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
            ui.label(".");
        });

        let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

        let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
            let mut layout_job = egui_extras::syntax_highlighting::highlight(
                ui.ctx(),
                ui.style(),
                &theme,
                string,
                language,
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
