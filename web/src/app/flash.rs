use super::Rp2350Component;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Flash {
    view: crate::widgets::MemoryView<0x1000_0000>,
}

impl Rp2350Component for Flash {
    const NAME: &'static str = "Flash";

    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading("Flash");
        self.view.ui(ui, &rp2350.bus.flash);
    }
}
