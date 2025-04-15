mod boot_ram;
mod boot_rom;
mod editor;
mod field;
mod flash;
mod processor_core;
mod sha256;
mod sram;
mod watchdog;

use crate::simulator::TaskCommand;
use egui::collapsing_header::CollapsingState;
use egui::{
    Button, Color32, ImageSource, Layout, Margin, ScrollArea, Theme, Ui, UiBuilder, Widget,
};
use egui_alignments::AlignedWidget;
use egui_dock::{
    DockArea, DockState, NodeIndex, SurfaceIndex, TabDestination, TabInsert, TabViewer,
};
use egui_extras::install_image_loaders;
use futures::channel::mpsc::Sender;
use rp2350::simulator::Pico2;
use rp2350::Rp2350;
use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;

pub trait Rp2350Component: Default + serde::Serialize + serde::de::DeserializeOwned {
    const NAME: &'static str;

    fn ui(&mut self, ui: &mut Ui, rp2350: &mut Rp2350);
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
    Flash,

    // Peripherals
    WatchDog,
    Sha256,
    Spi,
    Uart,
    I2c,
    Pwm,
    Dma,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct App {
    #[serde(skip)]
    open_windows: HashSet<Window>,
    #[serde(skip)]
    is_running: bool,
    #[serde(skip)]
    pico2: Rc<RefCell<Pico2>>,
    #[serde(skip)]
    send_task: Option<Sender<TaskCommand>>,

    editor: editor::CodeEditor,
    // components
    core0: processor_core::ProcessorCore<0>,
    core1: processor_core::ProcessorCore<1>,
    boot_rom: boot_rom::Bootroom,
    sram: sram::Sram,
    boot_ram: boot_ram::BootRam,
    field: field::Field,
    flash: flash::Flash,

    // peripherals
    watchdog: watchdog::WatchDog,
    sha256: sha256::Sha256,
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
            Window::Flash => "Flash",
            Window::Xip => "XIP",
            Window::WatchDog => "Watch Dog",
            Window::Sha256 => "SHA-256",
            Window::Spi => "SPI",
            Window::Uart => "UART",
            Window::I2c => "I2C",
            Window::Pwm => "PWM",
            Window::Dma => "DMA",
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
            .show(ui, |ui| {
                let mut pico2 = self.pico2.as_ref().borrow_mut();
                let rp2350: &mut Rp2350 = &mut pico2.mcu;

                match tab {
                    Window::Editor => self.editor.ui(ui),
                    Window::Field => self.field.ui(ui, rp2350),
                    Window::Core0 => self.core0.ui(ui, rp2350),
                    Window::Core1 => self.core1.ui(ui, rp2350),
                    Window::BootRom => self.boot_rom.ui(ui, rp2350),
                    Window::Sram => self.sram.ui(ui, rp2350),
                    Window::BootRam => self.boot_ram.ui(ui, rp2350),
                    Window::Flash => self.flash.ui(ui, rp2350),
                    Window::Xip => {
                        ui.heading("XIP");
                        ui.label("todo");
                    }
                    Window::WatchDog => self.watchdog.ui(ui, rp2350),
                    Window::Sha256 => self.sha256.ui(ui, rp2350),
                    Window::Spi => {
                        ui.heading("SPI");
                        ui.label("todo");
                    }
                    Window::Uart => {
                        ui.heading("UART");
                        ui.label("todo");
                    }
                    Window::I2c => {
                        ui.heading("I2C");
                        ui.label("todo");
                    }
                    Window::Pwm => {
                        ui.heading("PWM");
                        ui.label("todo");
                    }
                    Window::Dma => {
                        ui.heading("DMA");
                        ui.label("todo");
                    }
                }
            });
    }
}

impl Window {
    fn title(&self) -> &str {
        match self {
            Window::Editor => "Editor",
            Window::Field => "Field",
            Window::Core0 => "Core 0",
            Window::Core1 => "Core 1",
            Window::BootRom => "Boot ROM",
            Window::Sram => "SRAM",
            Window::BootRam => "Boot RAM",
            Window::Xip => "XIP",
            Window::Flash => "Flash",
            Window::WatchDog => "Watch Dog",
            Window::Sha256 => "SHA-256",
            Window::Spi => "SPI",
            Window::Uart => "UART",
            Window::I2c => "I2C",
            Window::Pwm => "PWM",
            Window::Dma => "DMA",
        }
    }
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

        install_image_loaders(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        let mut app = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            SimulatorApp::default()
        };

        let pico2 = Rc::clone(&app.app.pico2);
        let sender = crate::simulator::run_pico2_sim(cc.egui_ctx.clone(), pico2);
        app.app.send_task = Some(sender);

        return app;
    }

    fn step(&mut self) {
        if let Some(ref mut send_task) = self.app.send_task {
            let _ = send_task.try_send(TaskCommand::Step);
        }
    }

    fn run(&mut self) {
        if let Some(ref mut send_task) = self.app.send_task {
            let _ = send_task.try_send(TaskCommand::Run);
        }

        self.app.is_running = true;
    }

    fn pause(&mut self) {
        if let Some(ref mut send_task) = self.app.send_task {
            let _ = send_task.try_send(TaskCommand::Pause);
        }

        self.app.is_running = false;
    }

    fn stop(&mut self) {
        if let Some(ref mut send_task) = self.app.send_task {
            let _ = send_task.try_send(TaskCommand::Stop);
        }

        self.app.is_running = false;
    }

    fn top_panel(&mut self, ui: &mut egui::Ui) {
        // The top panel is often a good place for a menu bar:

        let is_web = cfg!(target_arch = "wasm32");
        if !is_web {
            egui::menu::bar(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
            });
        }

        ui.horizontal(|ui| {
            ui.add_space(200.0);

            if ui
                .add(self.top_panel_button(egui::include_image!("../assets/import.svg"), "Import"))
                .clicked()
            {
                self.stop();
                log::info!("Import clicked");
                crate::simulator::pick_file_into_pico2(ui.ctx().clone(), self.app.pico2.clone());
                // TODO
            }

            ui.add_space(100.0);

            if ui
                .add(self.top_panel_button(egui::include_image!("../assets/export.svg"), "Export"))
                .clicked()
            {
                // TODO
            }

            // only show these action with a flashed binary
            if self.app.pico2.borrow().is_flashed {
                ui.add_space(100.0);

                if self.app.is_running {
                    if self
                        .top_panel_button(egui::include_image!("../assets/pause.svg"), "Pause")
                        .ui(ui)
                        .clicked()
                    {
                        self.pause();
                    }

                    ui.add_space(100.0);

                    if self
                        .top_panel_button(egui::include_image!("../assets/stop.svg"), "Stop")
                        .ui(ui)
                        .clicked()
                    {
                        self.stop();
                    }
                } else {
                    if self
                        .top_panel_button(egui::include_image!("../assets/arrow-right.svg"), "Step")
                        .ui(ui)
                        .clicked()
                    {
                        self.step();
                    }

                    ui.add_space(100.0);

                    if self
                        .top_panel_button(egui::include_image!("../assets/play.svg"), "Run")
                        .ui(ui)
                        .clicked()
                    {
                        log::info!("Run clicked");
                        self.run();
                    }
                }
            }
        });
    }

    fn side_panel(&mut self, ui: &mut egui::Ui) {
        // The side panel is often a good place for tools and options.

        egui::widgets::global_theme_preference_buttons(ui);

        ui.add_space(20.0);

        ScrollArea::vertical().show(ui, |ui| {
            ui.with_layout(Layout::top_down_justified(egui::Align::LEFT), |ui| {
                self.side_panel_collapsing(
                    ui,
                    egui::include_image!("../assets/processor.svg"),
                    "Processor Core",
                    &[Window::Core0, Window::Core1],
                );

                self.side_panel_collapsing(
                    ui,
                    egui::include_image!("../assets/memory.svg"),
                    "Memory",
                    &[
                        Window::BootRom,
                        Window::Sram,
                        Window::BootRam,
                        Window::Flash,
                        Window::Xip,
                    ],
                );

                self.side_panel_collapsing(
                    ui,
                    egui::include_image!("../assets/peripherals.svg"),
                    "Peripherals",
                    &[
                        Window::Dma,
                        Window::Spi,
                        Window::Uart,
                        Window::I2c,
                        Window::Pwm,
                        Window::WatchDog,
                        Window::Sha256,
                    ],
                );
            });
        });
    }

    fn top_panel_button(
        &mut self,
        icon: ImageSource<'static>,
        text: &'static str,
    ) -> impl Widget + '_ {
        move |ui: &mut egui::Ui| {
            let img = egui::Image::new(icon)
                .alt_text(text)
                .tint(ui.ctx().theme().default_visuals().text_color())
                .maintain_aspect_ratio(true)
                .max_height(100.0)
                .shrink_to_fit();

            ui.scope_builder(UiBuilder::new().sense(egui::Sense::click()), |ui| {
                // ui.set_height(65.0);
                ui.add(img);
                ui.label(text);
            })
            .response
        }
    }

    fn add_side_panel_items(&mut self, ui: &mut egui::Ui, items: &[Window]) {
        for item in items {
            let mut open = self.app.open_windows.contains(&item);
            if ui
                .add(egui::Checkbox::new(&mut open, item.title()))
                .clicked()
            {
                if open {
                    self.dock_state.push_to_focused_leaf(*item);
                    self.app.open_windows.insert(*item);
                } else {
                    if let Some(tab) = self.dock_state.find_tab(&item) {
                        self.dock_state.remove_tab(tab);
                    }

                    self.app.open_windows.remove(&item);
                }
            }
        }
    }

    fn side_panel_collapsing(
        &mut self,
        ui: &mut egui::Ui,
        header_img: impl Into<ImageSource<'static>>,
        header_text: &str,
        items: &[Window],
    ) {
        CollapsingState::load_with_default_open(ui.ctx(), ui.make_persistent_id(header_text), true)
            .show_header(ui, |ui| {
                ui.horizontal(|ui| {
                    let img = egui::Image::new(header_img)
                        .alt_text("Processor Core")
                        .tint(ui.ctx().theme().default_visuals().text_color())
                        .maintain_aspect_ratio(true)
                        .max_height(65.0)
                        .shrink_to_fit();
                    ui.add(img);
                    ui.heading(header_text);
                })
            })
            .body(|ui| self.add_side_panel_items(ui, items));
    }
}

impl eframe::App for SimulatorApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel")
            .frame(egui::Frame::side_top_panel(&ctx.style()).inner_margin(10.0))
            .show(ctx, |ui| self.top_panel(ui));
        egui::SidePanel::left("side_panel").show(ctx, |ui| self.side_panel(ui));
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(0.))
            .show(ctx, |ui| {
                DockArea::new(&mut self.dock_state).show_inside(ui, &mut self.app)
            });
    }
}
