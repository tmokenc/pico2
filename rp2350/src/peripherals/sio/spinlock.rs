#[derive(Default)]
pub struct SpinLock {
    locks: u32,
}

impl SpinLock {
    pub fn lock(&mut self, index: u8) {
        todo!()
    }

    pub fn unlock(&mut self, index: u8) {
        todo!()
    }

    pub fn is_locked(&self, index: u8) -> bool {
        todo!()
    }
}
