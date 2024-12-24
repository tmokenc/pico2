#[derive(Default)]
pub struct Fifo {
    stack: [u32; 4],
    len: u8,
}

impl Fifo {
    pub fn push(&mut self, data: u32) {
        todo!()
    }

    pub fn pop(&mut self) -> Option<u32> {
        todo!()
    }

}
