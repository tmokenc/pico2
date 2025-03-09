mod boot_ram;
mod boot_rom;
mod editor;
mod field;
mod processor_core;
mod sram;

use egui::{Layout, Margin, ScrollArea, Ui};
use egui_dock::{
    DockArea, DockState, NodeIndex, Style, SurfaceIndex, TabDestination, TabInsert, TabViewer,
};
use egui_extras::install_image_loaders;
use rp2350::Rp2350;
use std::collections::{HashMap, HashSet};

const SIDE_PANEL_ITEMS: &[(Window, &str)] = &[
    (Window::Core0, "Core 0"),
    (Window::Core1, "Core 1"),
    (Window::BootRom, "Boot ROM"),
    (Window::Sram, "SRAM"),
    (Window::BootRam, "Boot RAM"),
    (Window::Xip, "XIP"),
];

pub trait Rp2350Component: Default + serde::Serialize + serde::de::DeserializeOwned {
    fn ui(&mut self, ui: &mut Ui, rp2350: &mut Rp2350);
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct App {
    open_windows: HashSet<Window>,
    #[serde(skip)]
    rp2350: Rp2350,

    editor: editor::CodeEditor,
    // components
    core0: processor_core::ProcessorCore<0>,
    core1: processor_core::ProcessorCore<1>,
    boot_rom: boot_rom::Bootroom,
    sram: sram::Sram,
    boot_ram: boot_ram::BootRam,
    field: field::Field,
}

impl TabViewer for App {
    type Tab = Window;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        let title = match tab {
            Window::Editor => "Editor",
            Window::Field => "Field",
            Window::Core0 => "Core 0",
            Window::Core1 => "Core 1",
            Window::BootRom => "Boot ROM",
            Window::Sram => "SRAM",
            Window::BootRam => "Boot RAM",
            Window::Xip => "XIP",
        };

        title.into()
    }

    fn allowed_in_windows(&self, tab: &mut Self::Tab) -> bool {
        match tab {
            Window::Editor | Window::Field => false,
            _ => true,
        }
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        match tab {
            Window::Editor | Window::Field => false,
            _ => true,
        }
    }

    fn on_close(&mut self, tab: &mut Self::Tab) -> bool {
        self.open_windows.remove(tab);
        true
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        egui::Frame::default()
            .inner_margin(Margin::same(10))
            .show(ui, |ui| match tab {
                Window::Editor => self.editor.ui(ui),
                Window::Field => self.field.ui(ui, &mut self.rp2350),
                Window::Core0 => self.core0.ui(ui, &mut self.rp2350),
                Window::Core1 => self.core1.ui(ui, &mut self.rp2350),
                Window::BootRom => self.boot_rom.ui(ui, &mut self.rp2350),
                Window::Sram => self.sram.ui(ui, &mut self.rp2350),
                Window::BootRam => self.boot_ram.ui(ui, &mut self.rp2350),
                Window::Xip => {
                    ui.label("XIP");
                }
            });
    }
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Window {
    // Base
    Editor,
    Field,

    // Processor Cores
    Core0,
    Core1,

    // Memories
    BootRom,
    Sram,
    BootRam,
    Xip,
    // Peripherals
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct SimulatorApp {
    app: App,
    #[serde(skip)]
    dock_state: DockState<Window>,
}

impl Default for SimulatorApp {
    fn default() -> Self {
        // By default we open the Editor and Simulator windows.

        let mut dock_state = DockState::new(vec![Window::Editor, Window::Field]);
        let mut app = App::default();

        // we just added the windows, so we know they exist
        let sim_tab = dock_state.find_tab(&Window::Editor).unwrap();

        dock_state.move_tab(
            sim_tab,
            TabDestination::Node(
                SurfaceIndex::main(),
                NodeIndex::root(),
                TabInsert::Split(egui_dock::Split::Left),
            ),
        );

        app.open_windows.insert(Window::Editor);
        app.open_windows.insert(Window::Field);

        Self { app, dock_state }
    }
}

impl SimulatorApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        install_image_loaders(&cc.egui_ctx);

        Default::default()
    }
}

impl eframe::App for SimulatorApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            // The side panel is often a good place for tools and options.

            ui.heading("Side Panel");

            ui.label("This is a template for eframe apps.");
            ui.label("It is easy to get started!");

            ScrollArea::vertical().show(ui, |ui| {
                ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                    ui.heading("Open Windows");

                    for (window, label) in SIDE_PANEL_ITEMS {
                        let mut open = self.app.open_windows.contains(window);
                        if ui
                            .add(egui::Checkbox::new(&mut open, label.to_owned()))
                            .clicked()
                        {
                            if open {
                                self.dock_state.push_to_focused_leaf(*window);
                                self.app.open_windows.insert(*window);
                            } else {
                                if let Some(tab) = self.dock_state.find_tab(window) {
                                    self.dock_state.remove_tab(tab);
                                }

                                self.app.open_windows.remove(window);
                            }
                        }
                    }
                });
            })
        });

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                DockArea::new(&mut self.dock_state).show_inside(ui, &mut self.app);
            });
    }
}
