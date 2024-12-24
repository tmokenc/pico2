pub(super) type Register = u32;

pub(super) trait RegisterValue {
    fn as_u32(&self) -> u32;
    fn signed(&self) -> i32 {
        self.as_u32() as i32
    }
}

impl RegisterValue for u32 {
    fn as_u32(&self) -> u32 {
        *self
    }
}

impl RegisterValue for i32 {
    fn as_u32(&self) -> u32 {
        *self as u32
    }

    fn signed(&self) -> i32 {
        *self
    }
}

impl RegisterValue for bool {
    fn as_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Default)]
pub struct Registers {
    inner: [u32; 32],
}

impl Registers {
    pub(super) fn write(&mut self, rd: Register, value: impl RegisterValue) {
        assert!(rd <= 32);
        let value = value.as_u32();
        self.inner[rd as usize] = value.as_u32();
    }

    pub fn read(&self, rd: Register) -> u32 {
        assert!(rd <= 32);

        // Hard wired to 0
        if rd == 0 {
            return 0;
        }

        self.inner[rd as usize]
    }
}
