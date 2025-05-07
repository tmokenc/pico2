/**
 * @file peripherals/sio/spinlock.rs
 * @author Nguyen Le Duy
 * @date 04/01/2025
 * @brief 32-bit spinlock implementation
 */
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Default)]
pub struct SpinLock {
    locks: Rc<RefCell<u32>>,
}

impl SpinLock {
    pub fn state(&self) -> u32 {
        *self.locks.borrow()
    }

    /// Get the state of the lock at the given index
    /// return 0 on unlocked, non-zero on locked
    pub fn lock_state(&self, index: u16) -> u32 {
        self.state() & (1 << index) as u32
    }

    /// Attemp to claim the lock
    /// Return 0 if the lock is already locked,
    /// `1 << index` if the lock was successfully claimed
    /// This was made to match the specification
    pub fn claim(&self, index: u16) -> u32 {
        let mut locks = self.locks.borrow_mut();
        let mask = 1 << index;
        if *locks & mask != 0 {
            0
        } else {
            *locks |= mask;
            mask
        }
    }

    /// Release the lock at the given index
    pub fn release(&self, index: u16) {
        let mut locks = self.locks.borrow_mut();
        *locks &= !(1 << index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinlock() {
        let spinlock = SpinLock::default();

        assert_eq!(spinlock.state(), 0);

        assert_eq!(spinlock.claim(0), 1);
        assert_eq!(spinlock.state(), 1);

        assert_eq!(spinlock.claim(0), 0);
        assert_eq!(spinlock.state(), 1);

        assert_eq!(spinlock.claim(1), 2);
        assert_eq!(spinlock.state(), 3);

        spinlock.release(0);
        assert_eq!(spinlock.state(), 2);

        spinlock.release(1);
        assert_eq!(spinlock.state(), 0);
    }
}
