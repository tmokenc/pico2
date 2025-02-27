//! A generic FIFO (First In First Out) queue implementation.
//! This is used in peripherals like UART and SIO

use core::mem;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FifoError {
    Full,
}

pub struct Fifo<T, const N: usize> {
    data: [T; N],
    head: usize,
    size: usize,
}

impl<T: Default + Copy, const N: usize> Default for Fifo<T, N> {
    fn default() -> Self {
        Fifo {
            data: [Default::default(); N],
            head: 0,
            size: 0,
        }
    }
}

impl<T: Default, const N: usize> Fifo<T, N> {
    pub fn push(&mut self, value: T) -> Result<(), FifoError> {
        if self.is_full() {
            return Err(FifoError::Full);
        }

        self.data[self.head] = value;
        self.head = (self.head + 1) % N;
        self.size += 1;

        Ok(())
    }

    pub fn pop(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let index = (self.head + N - self.size) % N;
        let value = mem::replace(&mut self.data[index], Default::default());
        self.size -= 1;

        Some(value)
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn is_full(&self) -> bool {
        self.size == N
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fifo() {
        let mut fifo = Fifo::<u8, 4>::default();

        assert!(fifo.is_empty());
        assert!(fifo.pop().is_none());

        fifo.push(1).unwrap();
        assert!(!fifo.is_empty());
        assert!(!fifo.is_full());
        fifo.push(2).unwrap();
        assert!(!fifo.is_empty());
        assert!(!fifo.is_full());
        fifo.push(3).unwrap();
        assert!(!fifo.is_empty());
        assert!(!fifo.is_full());
        fifo.push(4).unwrap();
        assert!(!fifo.is_empty());
        assert!(fifo.is_full());
        assert_eq!(fifo.push(5), Err(FifoError::Full));
        assert!(!fifo.is_empty());
        assert!(fifo.is_full());

        assert_eq!(fifo.pop(), Some(1));
        assert!(!fifo.is_empty());
        assert!(!fifo.is_full());
        assert_eq!(fifo.pop(), Some(2));
        assert!(!fifo.is_empty());
        assert!(!fifo.is_full());
        assert_eq!(fifo.pop(), Some(3));
        assert!(!fifo.is_empty());
        assert!(!fifo.is_full());
        assert_eq!(fifo.pop(), Some(4));
        assert!(fifo.is_empty());
        assert!(fifo.pop().is_none());
        assert!(fifo.is_empty());
    }
}
