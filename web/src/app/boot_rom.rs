use super::Rp2350Component;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Bootroom {
    view: crate::widgets::MemoryView<0x0000_0000>,
}

impl Rp2350Component for Bootroom {
    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading("Boot ROM");
        self.view.ui(ui, &rp2350.bus.rom);
    }
}
