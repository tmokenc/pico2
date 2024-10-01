#[derive(Default)]
pub struct Clock<const MHZ: u64> {
    ticks: u64,
}

impl<const MHZ: u64> Clock<MHZ> {
    pub fn tick(&mut self) {
        self.ticks += 1;
    }
}
