use super::*;
use crate::memory::GenericMemory;
use std::cell::RefCell;

pub struct BootRam {
    pub data: GenericMemory<{ 256 * 4 }>,
    write_onces: [u32; 2],
    boot_locks: RefCell<u8>, // 8 in total
}

impl Default for BootRam {
    fn default() -> Self {
        Self {
            data: GenericMemory::default(),
            write_onces: [0; 2],
            boot_locks: RefCell::new(0),
        }
    }
}

impl Peripheral for BootRam {
    fn read(&self, address: u16, ctx: &PeripheralAccessContext) -> PeripheralResult<u32> {
        if !ctx.secure {
            return Err(PeripheralError::MissingPermission);
        }

        match address {
            // Write once
            0x800 => Ok(self.write_onces[0]),
            0x804 => Ok(self.write_onces[1]),
            // Status
            0x808 => Ok(*self.boot_locks.borrow() as u32),
            // locks
            0x80C..=0x828 => {
                let mut locks = self.boot_locks.borrow_mut();
                let lock_position = (address - 0x80C) / 4;
                let lock_mask = 1u8 << lock_position as u8;
                if (*locks & lock_mask) != 0 {
                    Ok(0)
                } else {
                    *locks |= lock_mask;
                    Ok(lock_mask as u32)
                }
            }

            _ => self
                .data
                .read_u32(address as u32)
                .map_err(|_| PeripheralError::OutOfBounds),
        }
    }

    fn write_raw(
        &mut self,
        address: u16,
        value: u32,
        ctx: &PeripheralAccessContext,
    ) -> PeripheralResult<()> {
        if !ctx.secure {
            return Err(PeripheralError::MissingPermission);
        }

        match address {
            // 256 words of data
            0x800 => self.write_onces[0] |= value,
            0x804 => self.write_onces[1] |= value,
            0x808 => {
                let mut locks = self.boot_locks.borrow_mut();
                *locks = value as u8;
            }

            // write to unclaim lock
            0x80C..=0x828 => {
                let lock_position = (address - 0x80C) / 4;
                let lock_mask = 1 << lock_position;
                let mut locks = self.boot_locks.borrow_mut();
                *locks &= !lock_mask;
            }

            _ => {
                self.data
                    .write_u32(address as u32, value)
                    .map_err(|_| PeripheralError::OutOfBounds)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common::Requestor;

    macro_rules! setup {
        ($bootram:ident, $ctx:ident) => {
            let $ctx = PeripheralAccessContext {
                requestor: Requestor::Proc0,
                secure: true,
                ..Default::default()
            };
            let mut $bootram = BootRam::default();
        };
    }

    #[test]
    fn test_bootram() {
        setup!(bootram, ctx);
        bootram.write_raw(0x800, 0x1, &ctx).unwrap();
        assert_eq!(bootram.read(0x800, &ctx).unwrap(), 0x1);
    }

    #[test]
    fn test_bootram_locks() {
        setup!(bootram, ctx);
        assert_eq!(bootram.read(0x80C, &ctx), Ok(1));
        assert_eq!(bootram.read(0x80C, &ctx), Ok(0));
        assert_eq!(bootram.write(0x80C, 0x1, &ctx), Ok(()));
        assert_eq!(bootram.read(0x80C, &ctx), Ok(1));
    }

    #[test]
    fn test_bootram_write_once() {
        setup!(bootram, ctx);
        assert_eq!(bootram.read(0x800, &ctx), Ok(0));
        assert_eq!(bootram.write(0x800, 0x1, &ctx), Ok(()));
        assert_eq!(bootram.read(0x800, &ctx), Ok(1));
        assert_eq!(bootram.write(0x800, 0x1, &ctx), Ok(()));
        assert_eq!(bootram.read(0x800, &ctx), Ok(1));
    }

    #[test]
    fn test_bootram_write_once_2() {
        setup!(bootram, ctx);
        assert_eq!(bootram.read(0x804, &ctx), Ok(0));
        assert_eq!(bootram.write(0x804, 0x1, &ctx), Ok(()));
        assert_eq!(bootram.read(0x804, &ctx), Ok(1));
        assert_eq!(bootram.write(0x804, 0x1, &ctx), Ok(()));
        assert_eq!(bootram.read(0x804, &ctx), Ok(1));
    }

    #[test]
    fn test_bootram_multiple_locks() {
        setup!(bootram, ctx);
        assert_eq!(bootram.read(0x80C, &ctx), Ok(1));
        assert_eq!(bootram.read(0x810, &ctx), Ok(2));
        assert_eq!(bootram.read(0x814, &ctx), Ok(4));
        assert_eq!(bootram.read(0x818, &ctx), Ok(8));
        assert_eq!(bootram.read(0x81C, &ctx), Ok(16));
        assert_eq!(bootram.read(0x820, &ctx), Ok(32));
        assert_eq!(bootram.read(0x824, &ctx), Ok(64));
        assert_eq!(bootram.read(0x828, &ctx), Ok(128));

        // Clear some locks
        assert_eq!(bootram.write(0x80C, 0x1, &ctx), Ok(()));
        assert_eq!(bootram.write(0x810, 0x2, &ctx), Ok(()));
        assert_eq!(bootram.write(0x814, 0x4, &ctx), Ok(()));

        assert_eq!(bootram.read(0x808, &ctx), Ok(0xFF & !0b111));

        assert_eq!(bootram.read(0x80C, &ctx), Ok(1));
        assert_eq!(bootram.read(0x810, &ctx), Ok(2));
        assert_eq!(bootram.read(0x814, &ctx), Ok(4));
        assert_eq!(bootram.read(0x818, &ctx), Ok(0));
        assert_eq!(bootram.read(0x81C, &ctx), Ok(0));
        assert_eq!(bootram.read(0x820, &ctx), Ok(0));
        assert_eq!(bootram.read(0x824, &ctx), Ok(0));
        assert_eq!(bootram.read(0x828, &ctx), Ok(0));
    }
}
