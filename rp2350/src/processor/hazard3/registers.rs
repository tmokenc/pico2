pub type Register = u8;

pub trait RegisterValue {
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

impl RegisterValue for u16 {
    fn as_u32(&self) -> u32 {
        *self as u32
    }
}

#[derive(Default)]
pub struct Registers {
    pub(super) x: [u32; 32],
}

impl Registers {
    pub(super) fn write(&mut self, rd: Register, value: impl RegisterValue) {
        assert!(rd < 32);
        let value = value.as_u32();
        self.x[rd as usize] = value.as_u32();
        self.x[0] = 0; // x0 is hard wired to 0
    }

    pub fn read(&self, rd: Register) -> u32 {
        assert!(rd < 32);
        self.x[rd as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registers() {
        let mut registers = Registers::default();
        registers.write(1, 10);
        assert_eq!(registers.read(1), 10);
    }

    #[test]
    fn test_x0() {
        let mut registers = Registers::default();
        registers.write(0, 10);
        assert_eq!(registers.read(0), 0);
    }

    #[test]
    #[should_panic]
    fn test_out_of_bounds() {
        let mut registers = Registers::default();
        registers.write(32, 10);
    }
}
