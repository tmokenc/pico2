use crate::{Result, Time};
pub struct ReadAccess<T> {
    pub value: T,
    pub access_time: Time,
}

pub struct WriteAccess {
    pub access_time: Time,
}

pub trait MemoryAccess {
    fn read_u32(&self, address: u32) -> Result<ReadAccess<u32>>;
    fn write_u32(&mut self, address: u32, value: u32) -> Result<WriteAccess>;

    fn read_u16(&self, address: u32) -> Result<ReadAccess<u16>> {
        self.read_u32(address & !0b1).map(|result| ReadAccess {
            access_time: result.access_time,
            value: match address & 0b1 {
                0 => result.value as u16,
                _ => (result.value >> 16) as u16,
            },
        })
    }

    fn write_u16(&mut self, address: u32, value: u16) -> Result<WriteAccess> {
        let mut word = self.read_u32(address & !0b1)?.value;
        match address & 0b1 {
            0 => word = (word & 0xFFFF0000) | value as u32,
            _ => word = (word & 0x0000FFFF) | ((value as u32) << 16),
        }
        self.write_u32(address & !1, word)
    }

    fn read_u8(&self, address: u32) -> Result<ReadAccess<u8>> {
        self.read_u32(address & !0b11).map(|result| ReadAccess {
            access_time: result.access_time,
            value: result.value.to_le_bytes()[(address & 0b11) as usize],
        })
    }

    fn write_u8(&mut self, address: u32, value: u8) -> Result<WriteAccess> {
        let mut word = self.read_u32(address & !0b11)?.value;
        let mut bytes = word.to_le_bytes();
        bytes[(address & 0b11) as usize] = value;
        self.write_u32(address & !0b11, u32::from_le_bytes(bytes))
    }
}

pub(crate) struct GenericMemory<const N: usize, const R: u64, const W: u64> {
    data: [u8; N],
}

impl<const N: usize, const R: u64, const W: u64> From<[u8; N]> for GenericMemory<N, R, W> {
    fn from(data: [u8; N]) -> Self {
        let mut result = Self::new();
        result.data = data;
        result
    }
}

impl<const N: usize, const R: u64, const W: u64> GenericMemory<N, R, W> {
    pub fn new() -> Self {
        Self { data: [0; N] }
    }

    // pub fn write_atomic(&self, addr: u32, value: u32) {
    //     if W == 1 {
    //         self.write(addr, value);
    //     } else {
    //         todo!("Write not allowed")
    //     }
    // }
}

impl<const N: usize, const R: u64, const W: u64> MemoryAccess for GenericMemory<N, R, W> {
    fn read_u32(&self, address: u32) -> Result<ReadAccess<u32>> {
        Ok(ReadAccess {
            value: u32::from_le_bytes([
                self.data[address as usize],
                self.data[address as usize + 1],
                self.data[address as usize + 2],
                self.data[address as usize + 3],
            ]),
            access_time: R,
        })
    }

    fn write_u32(&mut self, address: u32, value: u32) -> Result<WriteAccess> {
        let bytes = value.to_le_bytes();
        self.data[address as usize] = bytes[0];
        self.data[address as usize + 1] = bytes[1];
        self.data[address as usize + 2] = bytes[2];
        self.data[address as usize + 3] = bytes[3];
        Ok(WriteAccess { access_time: W })
    }
}
