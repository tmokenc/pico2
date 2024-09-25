use std::marker::PhantomData;

pub(crate) struct Memory<const N: usize, const R: u64, const W: u64> {
    data: [u8; N],
}

impl<const N: usize, const R: u64, const W: u64> Memory<N, R, W> {
    pub fn new() -> Self {
        Self { data: [0; N] }
    }

    pub fn read(&self, addr: u64) -> u64 {
        todo!()
    }

    pub fn write(&mut self, addr: u64, value: u64) {
        todo!()
    }

    pub fn write_atomic(&self, addr: u64, value: u64) {
        todo!()
    }
}
