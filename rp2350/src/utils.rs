/**
 * @file utils.rs
 * @author Nguyen Le Duy
 * @date 02/01/2025
 * @brief Define utility functions
 */
pub mod fifo;

pub use fifo::*;

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

/// Set a state of a bit (on or off) in a u32 variable.
pub fn set_bit_state(bits: &mut u32, bit: u32, state: bool) {
    if state {
        set_bit(bits, bit);
    } else {
        clear_bit(bits, bit);
    }
}

// write 1 to clear
pub fn w1c(dst: &mut u32, bits: u32, mask: u32) {
    let to_clear = bits & mask;
    *dst &= !to_clear;
}

pub fn clear_bits(bits: &mut u32, mask: u32) {
    *bits &= !mask;
}

pub fn set_bit(bits: &mut u32, position: u32) {
    *bits |= 1 << position;
}

pub fn clear_bit(bits: &mut u32, position: u32) {
    *bits &= !(1 << position);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bit() {
        let bits: u32 = 0b10101010;
        assert_eq!(extract_bit(bits, 2), 0);
        assert_eq!(extract_bit(bits, 3), 1);
    }

    #[test]
    fn test_extract_bits() {
        let bits: u32 = 0b10101010;
        assert_eq!(extract_bits(bits, 2..=5), 0b1010);
        assert_eq!(extract_bits(bits, 4..=6), 0b010);
    }

    #[test]
    fn test_sign_extend() {
        assert_eq!(sign_extend(0b11111111, 7), 0xFFFFFFFF);
        assert_eq!(sign_extend(0b01111111, 7), 0x0000007F);
    }

    #[test]
    fn test_set_bit_state() {
        let mut bits: u32 = 0b00000000;
        set_bit_state(&mut bits, 2, true);
        assert_eq!(bits, 0b00000100);
        set_bit_state(&mut bits, 2, false);
        assert_eq!(bits, 0b00000000);
    }

    #[test]
    fn test_set_bit() {
        let mut bits: u32 = 0b00000000;
        set_bit(&mut bits, 2);
        assert_eq!(bits, 0b00000100);
        set_bit(&mut bits, 2);
        assert_eq!(bits, 0b00000100);
    }

    #[test]
    fn test_clear_bit() {
        let mut bits: u32 = 0b00000100;
        clear_bit(&mut bits, 2);
        assert_eq!(bits, 0b00000000);
        clear_bit(&mut bits, 2);
        assert_eq!(bits, 0b00000000);
    }

    #[test]
    fn test_clear_bits() {
        let mut bits: u32 = 0b11111111;
        clear_bits(&mut bits, 0b00001111);
        assert_eq!(bits, 0b11110000);
        clear_bits(&mut bits, 0b11110000);
        assert_eq!(bits, 0b00000000);
    }

    #[test]
    fn test_w1c() {
        let mut bits: u32 = 0b11111111;
        w1c(&mut bits, 0b01001110, 0b10111111);
        assert_eq!(bits, 0b11110001);
        w1c(&mut bits, 0b11110000, 0b11111111);
        assert_eq!(bits, 0b00000001);
    }
}
