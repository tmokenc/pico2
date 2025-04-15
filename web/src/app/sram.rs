use super::Rp2350Component;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Sram {
    view: crate::widgets::MemoryView<0x2000_0000>,
}

impl Rp2350Component for Sram {
    const NAME: &'static str = "SRAM";

    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading("SRAM");
        self.view.ui(ui, &rp2350.bus.sram);
    }
}
