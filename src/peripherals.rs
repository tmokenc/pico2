pub trait Peripheral {
    fn read_uint32(&mut self, offset: u64) -> u64;
    fn write_uint32(&mut self, offset: u64);
    fn write_uint32_atomic(&mut self, offset: u64);
}
