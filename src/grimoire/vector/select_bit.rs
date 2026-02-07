//! Select bit helper function.
//!
//! Ported from: lib/marisa/grimoire/vector/bit-vector.cc
//!
//! This module provides the select_bit function which finds the position
//! of the i-th set bit within a word.

use super::select_tables::SELECT_TABLE;

/// Finds the position of the i-th set bit in a 64-bit unit.
///
/// # Arguments
///
/// * `i` - The rank of the bit to find (0-indexed)
/// * `bit_id` - The starting bit position for this unit
/// * `unit` - The 64-bit value to search
///
/// # Returns
///
/// The absolute position of the i-th set bit
///
/// This is a simplified implementation. The original C++ version uses SIMD
/// optimizations for better performance.
#[cfg(target_pointer_width = "64")]
#[inline]
pub fn select_bit_u64(i: usize, bit_id: usize, unit: u64) -> usize {
    let mut remaining = i;
    let mut offset = 0usize;

    // Process byte by byte
    for byte_idx in 0..8 {
        let byte = ((unit >> (byte_idx * 8)) & 0xFF) as u8;
        let byte_popcount = byte.count_ones() as usize;

        if remaining < byte_popcount {
            // The i-th bit is in this byte
            return bit_id + offset + SELECT_TABLE[remaining][byte as usize] as usize;
        }

        remaining -= byte_popcount;
        offset += 8;
    }

    // Should not reach here if input is valid
    bit_id + 63
}

/// Finds the position of the i-th set bit in a 32-bit unit (for 32-bit platforms).
///
/// # Arguments
///
/// * `i` - The rank of the bit to find (0-indexed)
/// * `bit_id` - The starting bit position for this unit
/// * `unit` - The 32-bit value to search
///
/// # Returns
///
/// The absolute position of the i-th set bit
#[cfg(target_pointer_width = "32")]
#[inline]
pub fn select_bit_u32(i: usize, bit_id: usize, unit: u32) -> usize {
    let mut remaining = i;
    let mut offset = 0usize;

    // Process byte by byte
    for byte_idx in 0..4 {
        let byte = ((unit >> (byte_idx * 8)) & 0xFF) as u8;
        let byte_popcount = byte.count_ones() as usize;

        if remaining < byte_popcount {
            // The i-th bit is in this byte
            return bit_id + offset + SELECT_TABLE[remaining][byte as usize] as usize;
        }

        remaining -= byte_popcount;
        offset += 8;
    }

    // Should not reach here if input is valid
    bit_id + 31
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_select_bit_u64_basic() {
        // 0b00000001 - first bit at position 0
        assert_eq!(select_bit_u64(0, 0, 0b00000001), 0);

        // 0b00000010 - first bit at position 1
        assert_eq!(select_bit_u64(0, 0, 0b00000010), 1);

        // 0b00000011 - first bit at 0, second at 1
        assert_eq!(select_bit_u64(0, 0, 0b00000011), 0);
        assert_eq!(select_bit_u64(1, 0, 0b00000011), 1);

        // 0b00000101 - first bit at 0, second at 2
        assert_eq!(select_bit_u64(0, 0, 0b00000101), 0);
        assert_eq!(select_bit_u64(1, 0, 0b00000101), 2);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_select_bit_u64_offset() {
        // Test with non-zero bit_id offset
        assert_eq!(select_bit_u64(0, 100, 0b00000001), 100);
        assert_eq!(select_bit_u64(0, 100, 0b00000010), 101);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_select_bit_u64_multiple_bytes() {
        // 0x0101 = bits at positions 0 and 8
        assert_eq!(select_bit_u64(0, 0, 0x0101), 0);
        assert_eq!(select_bit_u64(1, 0, 0x0101), 8);

        // 0xFF00 = 8 bits starting at position 8
        assert_eq!(select_bit_u64(0, 0, 0xFF00), 8);
        assert_eq!(select_bit_u64(7, 0, 0xFF00), 15);
    }

    #[cfg(target_pointer_width = "32")]
    #[test]
    fn test_select_bit_u32_basic() {
        assert_eq!(select_bit_u32(0, 0, 0b00000001), 0);
        assert_eq!(select_bit_u32(0, 0, 0b00000010), 1);
        assert_eq!(select_bit_u32(0, 0, 0b00000011), 0);
        assert_eq!(select_bit_u32(1, 0, 0b00000011), 1);
    }
}
