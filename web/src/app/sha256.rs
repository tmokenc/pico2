use super::Rp2350Component;
use rp2350::Rp2350;

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Sha256;

impl Rp2350Component for Sha256 {
    const NAME: &'static str = "SHA-256 Accelerator";

    fn ui(&mut self, ui: &mut egui::Ui, rp2350: &mut Rp2350) {
        ui.heading("Boot ROM");
        let sha256 = rp2350.bus.peripherals.sha256.borrow();

        egui::Grid::new("SHA256 Info")
            .num_columns(2)
            .spacing([40.0, 6.0])
            .striped(false)
            .show(ui, |ui| {
                ui.label("Byte swap");
                ui.label(format!("{}", sha256.bswap));
                ui.end_row();

                ui.label("DMA size");
                ui.label(format!("{}", sha256.dma_size));
                ui.end_row();

                ui.label("Error WDATA not ready");
                ui.label(format!("{}", sha256.err_wdata_not_rdy));
                ui.end_row();

                ui.label("Sum valid");
                ui.label(format!("{}", sha256.sum_vld));
                ui.end_row();

                ui.label("WDATA ready");
                ui.label(format!("{}", sha256.wdata_rdy));
                ui.end_row();

                let digest_str = sha256
                    .sum
                    .iter()
                    .map(|byte| format!("{:02x}", byte))
                    .collect::<String>();

                ui.label("Last computed digest");
                ui.label(digest_str);
                ui.end_row();
            });
    }
}
