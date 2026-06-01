//! Select bit helper function.
//!
//! Ported from: lib/marisa/grimoire/vector/bit-vector.cc
//!
//! This module provides the select_bit function which finds the position
//! of the i-th set bit within a word.
//!
//! On x86_64 CPUs that support BMI2 (Haswell/Zen and later), we use the
//! PDEP instruction for an essentially branch-free O(1) select. Detection
//! is performed once and the result is cached in an atomic so subsequent
//! calls cost a single relaxed load.

use super::select_tables::SELECT_TABLE;

#[cfg(target_arch = "x86_64")]
use std::sync::atomic::{AtomicU8, Ordering};

/// Cached BMI2 (PDEP) detection result.
///
/// 0 = unknown (not probed yet), 1 = available, 2 = unavailable.
/// `Relaxed` is sufficient: the worst case is multiple threads racing the
/// initial probe, all writing the same value.
#[cfg(target_arch = "x86_64")]
static BMI2_AVAILABLE: AtomicU8 = AtomicU8::new(0);

#[cfg(target_arch = "x86_64")]
#[inline]
fn has_bmi2() -> bool {
    match BMI2_AVAILABLE.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => {
            let detected = std::is_x86_feature_detected!("bmi2");
            BMI2_AVAILABLE.store(if detected { 1 } else { 2 }, Ordering::Relaxed);
            detected
        }
    }
}

/// PDEP-based select within a 64-bit word.
///
/// Places `1 << i` into the i-th set bit position of `unit` and returns
/// its position. Requires BMI2 (caller must check `has_bmi2()`).
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "bmi2")]
#[inline]
unsafe fn select_bit_u64_pdep(i: usize, bit_id: usize, unit: u64) -> usize {
    // SAFETY: caller ensures BMI2 is available; intrinsic itself is safe
    // beyond that.
    let bit = core::arch::x86_64::_pdep_u64(1u64 << i, unit);
    bit_id + bit.trailing_zeros() as usize
}

/// Portable byte-table fallback for select within a 64-bit word.
#[inline]
fn select_bit_u64_table(i: usize, bit_id: usize, unit: u64) -> usize {
    let mut remaining = i;
    let mut offset = 0usize;

    // Process byte by byte
    for byte_idx in 0..8 {
        let byte = ((unit >> (byte_idx * 8)) & 0xFF) as u8;
        let byte_popcount = byte.count_ones() as usize;

        if remaining < byte_popcount {
            return bit_id + offset + SELECT_TABLE[remaining][byte as usize] as usize;
        }

        remaining -= byte_popcount;
        offset += 8;
    }

    bit_id + 63
}

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
/// The absolute position of the i-th set bit.
#[cfg(target_pointer_width = "64")]
#[inline]
pub fn select_bit_u64(i: usize, bit_id: usize, unit: u64) -> usize {
    #[cfg(all(target_arch = "x86_64", target_feature = "bmi2"))]
    {
        // Build-time BMI2: no detection needed, inline the PDEP path.
        // SAFETY: target_feature implies the instruction is available.
        return unsafe { select_bit_u64_pdep(i, bit_id, unit) };
    }
    #[cfg(all(target_arch = "x86_64", not(target_feature = "bmi2")))]
    {
        if has_bmi2() {
            // SAFETY: BMI2 confirmed available at runtime.
            return unsafe { select_bit_u64_pdep(i, bit_id, unit) };
        }
    }
    select_bit_u64_table(i, bit_id, unit)
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

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_select_bit_u64_high_bytes() {
        // Bits in upper bytes
        let unit: u64 = 1u64 << 40 | 1u64 << 50 | 1u64 << 63;
        assert_eq!(select_bit_u64(0, 0, unit), 40);
        assert_eq!(select_bit_u64(1, 0, unit), 50);
        assert_eq!(select_bit_u64(2, 0, unit), 63);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_select_bit_u64_fallback_matches_pdep() {
        // Cross-check fallback against the dispatcher across many inputs.
        for unit in [
            0x0123_4567_89AB_CDEFu64,
            0xFFFF_FFFF_FFFF_FFFFu64,
            0x8000_0000_0000_0001u64,
            0xAAAA_AAAA_AAAA_AAAAu64,
            0x5555_5555_5555_5555u64,
        ] {
            let ones = unit.count_ones() as usize;
            for i in 0..ones {
                assert_eq!(
                    select_bit_u64(i, 0, unit),
                    select_bit_u64_table(i, 0, unit),
                    "mismatch at i={} unit={:#x}",
                    i,
                    unit
                );
            }
        }
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
