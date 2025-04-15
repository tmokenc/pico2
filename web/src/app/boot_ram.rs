use super::Rp2350Component;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct BootRam {
    view: crate::widgets::MemoryView<0x400e0000>,
}

impl Rp2350Component for BootRam {
    const NAME: &'static str = "Boot RAM";

    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading("Boot RAM");
        self.view.ui(ui, &rp2350.bus.peripherals.bootram.data);
    }
}
