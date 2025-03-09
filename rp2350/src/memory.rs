use std::ops::Deref;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryOutOfBoundsError;

type MemoryResult<T> = Result<T, MemoryOutOfBoundsError>;

pub struct GenericMemory<const N: usize> {
    data: Vec<u8>,
}

impl<const N: usize> Deref for GenericMemory<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<const N: usize> Default for GenericMemory<N> {
    fn default() -> Self {
        Self { data: vec![0; N] }
    }
}

impl<const N: usize> GenericMemory<N> {
    pub fn new(data: &[u8]) -> Self {
        assert!(data.len() <= N);

        Self {
            data: data.to_vec(),
        }
    }

    pub fn read_u32(&self, address: u32) -> MemoryResult<u32> {
        // Check if the address is out of bounds
        if address as usize + 3 >= N {
            return Err(MemoryOutOfBoundsError);
        }

        Ok(u32::from_le_bytes([
            self.data[address as usize],
            self.data[address as usize + 1],
            self.data[address as usize + 2],
            self.data[address as usize + 3],
        ]))
    }

    pub fn write_u32(&mut self, address: u32, value: u32) -> MemoryResult<()> {
        // Check if the address is out of bounds
        if address as usize + 3 >= N {
            return Err(MemoryOutOfBoundsError);
        }

        let bytes = value.to_le_bytes();
        self.data[address as usize] = bytes[0];
        self.data[address as usize + 1] = bytes[1];
        self.data[address as usize + 2] = bytes[2];
        self.data[address as usize + 3] = bytes[3];

        Ok(())
    }

    pub fn read_u16(&self, address: u32) -> MemoryResult<u16> {
        // Check if the address is out of bounds
        if address as usize + 1 >= N {
            return Err(MemoryOutOfBoundsError);
        }

        Ok(u16::from_le_bytes([
            self.data[address as usize],
            self.data[address as usize + 1],
        ]))
    }

    pub fn write_u16(&mut self, address: u32, value: u16) -> MemoryResult<()> {
        if address as usize + 1 >= N {
            return Err(MemoryOutOfBoundsError);
        }

        let bytes = value.to_le_bytes();
        self.data[address as usize] = bytes[0];
        self.data[address as usize + 1] = bytes[1];

        Ok(())
    }

    pub fn read_u8(&self, address: u32) -> MemoryResult<u8> {
        if address as usize >= N {
            return Err(MemoryOutOfBoundsError);
        }

        Ok(self.data[address as usize])
    }

    pub fn write_u8(&mut self, address: u32, value: u8) -> MemoryResult<()> {
        if address as usize >= N {
            return Err(MemoryOutOfBoundsError);
        }

        self.data[address as usize] = value;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_access() {
        let mut memory: GenericMemory<1024> = Default::default();

        memory.write_u32(0, 0x12345678).unwrap();
        assert_eq!(memory.read_u32(0).unwrap(), 0x12345678);

        memory.write_u16(4, 0x1234).unwrap();
        assert_eq!(memory.read_u16(4).unwrap(), 0x1234);

        memory.write_u8(6, 0x12).unwrap();
        assert_eq!(memory.read_u8(6).unwrap(), 0x12);
    }

    #[test]
    fn test_memory_access_out_of_bounds() {
        let mut memory: GenericMemory<1024> = GenericMemory::default();

        assert_eq!(memory.read_u32(1024).unwrap_err(), MemoryOutOfBoundsError);
        assert_eq!(
            memory.write_u32(1024, 0x12345678).unwrap_err(),
            MemoryOutOfBoundsError
        );

        assert_eq!(memory.read_u16(1024).unwrap_err(), MemoryOutOfBoundsError);
        assert_eq!(
            memory.write_u16(1024, 0x1234).unwrap_err(),
            MemoryOutOfBoundsError
        );

        assert_eq!(memory.read_u8(1024).unwrap_err(), MemoryOutOfBoundsError);
        assert_eq!(
            memory.write_u8(1024, 0x12).unwrap_err(),
            MemoryOutOfBoundsError
        );
    }
}
