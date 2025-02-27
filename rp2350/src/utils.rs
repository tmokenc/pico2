//! Utility functions and data structures for the emulator.

pub mod fifo;

pub use fifo::{Fifo, FifoError};

use num_traits::{AsPrimitive, PrimInt};

pub fn extract_bit<T>(bits: T, bit: T) -> T
where
    T: PrimInt + AsPrimitive<u32> + Copy, // Ensures integer traits and conversion to u32
    u32: AsPrimitive<T>,                  // Allows conversion from u32 back to T
{
    extract_bits(bits, bit..=bit)
}

pub fn extract_bits<T>(bits: T, range: std::ops::RangeInclusive<T>) -> T
where
    T: PrimInt + AsPrimitive<u32> + Copy, // Ensures integer traits and conversion to u32
    u32: AsPrimitive<T>,                  // Allows conversion from u32 back to T
{
    let lsb: u32 = range.start().as_();
    let msb: u32 = range.end().as_();
    let bits: u32 = bits.as_();

    let mask = 1u32.checked_shl(msb + 1).map(|v| v - 1).unwrap_or(u32::MAX);
    let result = (bits & mask) >> lsb;

    result.as_() // Convert back to T
}

pub const fn sign_extend(bits: u32, sign_bit: u32) -> u32 {
    (bits & (1 << sign_bit + 1) - 1)
        .overflowing_sub((bits & 1 << sign_bit) << 1)
        .0
}
