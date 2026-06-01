//! Population count utilities.
//!
//! Ported from: lib/marisa/grimoire/vector/pop-count.h
//!
//! Provides efficient population count (number of set bits) functions.
//! Rust's standard library provides hardware-accelerated implementations
//! via count_ones(), which compiles to POPCNT instruction when available.

/// Counts the number of set bits in a 64-bit value.
///
/// This function uses Rust's built-in count_ones() which is optimized
/// to use hardware POPCNT instruction when available.
///
/// # Arguments
///
/// * `x` - The value to count bits in
///
/// # Returns
///
/// The number of bits set to 1
#[inline]
pub fn popcount(x: u64) -> usize {
    x.count_ones() as usize
}

/// Counts the number of set bits in a 32-bit value.
///
/// # Arguments
///
/// * `x` - The value to count bits in
///
/// # Returns
///
/// The number of bits set to 1
#[inline]
pub fn popcount_u32(x: u32) -> usize {
    x.count_ones() as usize
}

/// Type alias for the unit type used in bit vectors.
///
/// rsmarisa fixes the bit-vector word at 64 bits on every target (see
/// [`crate::base::WORD_SIZE`]), so `Unit` is always `u64`.
pub type Unit = u64;

/// Counts the number of set bits in a Unit value.
#[inline]
pub fn popcount_unit(x: Unit) -> usize {
    popcount(x)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::WORD_SIZE;

    #[test]
    fn test_popcount() {
        assert_eq!(popcount(0), 0);
        assert_eq!(popcount(1), 1);
        assert_eq!(popcount(0b1010), 2);
        assert_eq!(popcount(u64::MAX), 64);
        assert_eq!(popcount(0x00FF00FF00FF00FF), 32);
    }

    #[test]
    fn test_popcount_u32() {
        assert_eq!(popcount_u32(0), 0);
        assert_eq!(popcount_u32(1), 1);
        assert_eq!(popcount_u32(0b1010), 2);
        assert_eq!(popcount_u32(u32::MAX), 32);
        assert_eq!(popcount_u32(0x00FF00FF), 16);
    }

    #[test]
    fn test_popcount_unit() {
        assert_eq!(popcount_unit(0), 0);
        assert_eq!(popcount_unit(1), 1);
        assert_eq!(popcount_unit(0b1010), 2);
        assert_eq!(popcount_unit(u64::MAX), 64);
    }

    #[test]
    fn test_word_size_consistency() {
        // Word size and Unit are fixed at 64 bits on every target.
        assert_eq!(WORD_SIZE, 64);
        assert_eq!(std::mem::size_of::<Unit>(), 8);
    }
}
