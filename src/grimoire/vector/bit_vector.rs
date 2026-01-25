//! Bit vector with rank/select operations.
//!
//! Ported from:
//! - lib/marisa/grimoire/vector/bit-vector.h
//! - lib/marisa/grimoire/vector/bit-vector.cc
//!
//! This module provides a compact bit vector implementation with support for:
//! - Efficient rank queries (count 0s or 1s up to a position)
//! - Efficient select queries (find position of nth 0 or 1)
//! - Space-efficient storage using bit packing

use super::pop_count::{popcount, popcount_u32, popcount_unit, Unit};
use super::rank_index::RankIndex;
use super::vector::Vector;
use crate::base::{ErrorCode, WORD_SIZE};
use crate::grimoire::io::{Mapper, Reader, Writer};

#[cfg(target_pointer_width = "64")]
use super::select_bit::select_bit_u64;
#[cfg(target_pointer_width = "32")]
use super::select_bit::select_bit_u32;

/// Bit vector supporting rank and select operations.
///
/// A bit vector that stores bits compactly and supports efficient
/// rank and select operations through index structures.
#[derive(Default)]
pub struct BitVector {
    /// Storage for bits, packed into Units (u32 or u64 depending on platform).
    units: Vector<Unit>,
    /// Number of bits currently stored.
    size: usize,
    /// Number of 1-bits in the vector.
    num_1s: usize,
    /// Rank index for accelerating rank queries.
    ranks: Vector<RankIndex>,
    /// Select index for 0-bits (optional).
    select0s: Vector<u32>,
    /// Select index for 1-bits (optional).
    select1s: Vector<u32>,
}

impl BitVector {
    /// Creates a new empty bit vector.
    #[inline]
    pub fn new() -> Self {
        BitVector {
            units: Vector::new(),
            size: 0,
            num_1s: 0,
            ranks: Vector::new(),
            select0s: Vector::new(),
            select1s: Vector::new(),
        }
    }

    /// Pushes a bit onto the end of the vector.
    ///
    /// # Arguments
    ///
    /// * `bit` - The bit value to push (true for 1, false for 0)
    ///
    /// # Panics
    ///
    /// Panics if the size would exceed u32::MAX.
    pub fn push_back(&mut self, bit: bool) {
        assert!(
            self.size < u32::MAX as usize,
            "BitVector size cannot exceed u32::MAX"
        );

        // Expand units if needed
        if self.size == WORD_SIZE * self.units.size() {
            self.units.resize(self.units.size() + (64 / WORD_SIZE), 0);
        }

        // Set the bit if true
        if bit {
            let unit_index = self.size / WORD_SIZE;
            let bit_offset = self.size % WORD_SIZE;
            let current = self.units[unit_index];
            self.units[unit_index] = current | ((1 as Unit) << bit_offset);
            self.num_1s += 1;
        }

        self.size += 1;
    }

    /// Returns the bit at the given index.
    ///
    /// # Arguments
    ///
    /// * `i` - The index of the bit to retrieve
    ///
    /// # Returns
    ///
    /// The bit value at index i (true for 1, false for 0)
    ///
    /// # Panics
    ///
    /// Panics if `i >= size()`
    #[inline]
    pub fn get(&self, i: usize) -> bool {
        assert!(i < self.size, "Index out of bounds");
        let unit_index = i / WORD_SIZE;
        let bit_offset = i % WORD_SIZE;
        (self.units[unit_index] & ((1 as Unit) << bit_offset)) != 0
    }

    /// Returns the number of 0-bits in the vector.
    #[inline]
    pub fn num_0s(&self) -> usize {
        self.size - self.num_1s
    }

    /// Returns the number of 1-bits in the vector.
    #[inline]
    pub fn num_1s(&self) -> usize {
        self.num_1s
    }

    /// Returns true if the vector is empty.
    #[inline]
    pub fn empty(&self) -> bool {
        self.size == 0
    }

    /// Returns the number of bits in the vector.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the total size in bytes.
    #[inline]
    pub fn total_size(&self) -> usize {
        self.units.total_size()
            + self.ranks.total_size()
            + self.select0s.total_size()
            + self.select1s.total_size()
    }

    /// Returns the I/O size needed for serialization.
    #[inline]
    pub fn io_size(&self) -> usize {
        self.units.io_size()
            + std::mem::size_of::<u32>() * 2
            + self.ranks.io_size()
            + self.select0s.io_size()
            + self.select1s.io_size()
    }

    /// Clears the bit vector.
    #[inline]
    pub fn clear(&mut self) {
        *self = BitVector::new();
    }

    /// Swaps the contents of two bit vectors.
    #[inline]
    pub fn swap(&mut self, other: &mut BitVector) {
        self.units.swap(&mut other.units);
        std::mem::swap(&mut self.size, &mut other.size);
        std::mem::swap(&mut self.num_1s, &mut other.num_1s);
        self.ranks.swap(&mut other.ranks);
        self.select0s.swap(&mut other.select0s);
        self.select1s.swap(&mut other.select1s);
    }

    /// Disables the select0 index.
    #[inline]
    pub fn disable_select0(&mut self) {
        self.select0s.clear();
    }

    /// Disables the select1 index.
    #[inline]
    pub fn disable_select1(&mut self) {
        self.select1s.clear();
    }

    /// Returns the number of 0-bits in the range [0, i).
    ///
    /// # Arguments
    ///
    /// * `i` - The position (exclusive) to count up to
    ///
    /// # Returns
    ///
    /// The number of 0-bits before position i
    ///
    /// # Panics
    ///
    /// Panics if the ranks index is empty or if i > size()
    #[inline]
    pub fn rank0(&self, i: usize) -> usize {
        assert!(!self.ranks.empty(), "Rank index not built");
        assert!(i <= self.size, "Index out of bounds");
        i - self.rank1(i)
    }

    /// Returns the number of 1-bits in the range [0, i).
    ///
    /// This uses the rank index for efficient O(1) queries.
    ///
    /// # Arguments
    ///
    /// * `i` - The position (exclusive) to count up to
    ///
    /// # Returns
    ///
    /// The number of 1-bits before position i
    ///
    /// # Panics
    ///
    /// Panics if the ranks index is empty or if i > size()
    pub fn rank1(&self, i: usize) -> usize {
        assert!(!self.ranks.empty(), "Rank index not built");
        assert!(i <= self.size, "Index out of bounds");

        let rank_index = &self.ranks[i / 512];
        let mut offset = rank_index.abs();

        // Add relative rank based on 64-bit block position within 512-bit block
        match (i / 64) % 8 {
            1 => offset += rank_index.rel1(),
            2 => offset += rank_index.rel2(),
            3 => offset += rank_index.rel3(),
            4 => offset += rank_index.rel4(),
            5 => offset += rank_index.rel5(),
            6 => offset += rank_index.rel6(),
            7 => offset += rank_index.rel7(),
            _ => {} // case 0: no addition needed
        }

        // Add popcount of bits in the final partial unit
        #[cfg(target_pointer_width = "64")]
        {
            let bit_offset = i % 64;
            if bit_offset > 0 {
                let mask = (1u64 << bit_offset) - 1;
                offset += popcount(self.units[i / 64] & mask);
            }
        }

        #[cfg(target_pointer_width = "32")]
        {
            // For 32-bit, need to handle two units per 64-bit block
            if ((i / 32) & 1) == 1 {
                offset += popcount_u32(self.units[(i / 32) - 1]);
            }
            let bit_offset = i % 32;
            if bit_offset > 0 {
                let mask = (1u32 << bit_offset) - 1;
                offset += popcount_u32(self.units[i / 32] & mask);
            }
        }

        offset
    }

    /// Builds the rank and select indices.
    ///
    /// This must be called before using rank() or select() operations.
    ///
    /// # Arguments
    ///
    /// * `enables_select0` - Whether to build select0 index
    /// * `enables_select1` - Whether to build select1 index
    pub fn build(&mut self, enables_select0: bool, enables_select1: bool) {
        // Build indices in-place
        self.build_index_internal(enables_select0, enables_select1);

        // Shrink vectors to save memory
        self.units.shrink();
        self.ranks.shrink();
        if enables_select0 {
            self.select0s.shrink();
        }
        if enables_select1 {
            self.select1s.shrink();
        }
    }

    /// Internal function to build rank/select indices in-place.
    fn build_index_internal(&mut self, enables_select0: bool, enables_select1: bool) {
        let num_bits = self.size;

        // Allocate ranks array
        let num_ranks = (num_bits / 512) + if (num_bits % 512) != 0 { 1 } else { 0 } + 1;
        self.ranks.resize(num_ranks, RankIndex::default());

        let mut num_0s = 0usize;
        let mut num_1s = 0usize;

        let num_units = self.units.size();

        // We need to collect rank data without modifying units
        // Create temporary storage for rank indices
        let mut temp_ranks = Vec::new();
        temp_ranks.resize(self.ranks.size(), RankIndex::default());

        for unit_id in 0..num_units {
            let bit_id = unit_id * WORD_SIZE;

            // Update rank index at 64-bit boundaries
            if (bit_id % 64) == 0 {
                let rank_id = bit_id / 512;
                let rank_abs = temp_ranks[rank_id].abs();
                match (bit_id / 64) % 8 {
                    0 => temp_ranks[rank_id].set_abs(num_1s),
                    1 => temp_ranks[rank_id].set_rel1(num_1s - rank_abs),
                    2 => temp_ranks[rank_id].set_rel2(num_1s - rank_abs),
                    3 => temp_ranks[rank_id].set_rel3(num_1s - rank_abs),
                    4 => temp_ranks[rank_id].set_rel4(num_1s - rank_abs),
                    5 => temp_ranks[rank_id].set_rel5(num_1s - rank_abs),
                    6 => temp_ranks[rank_id].set_rel6(num_1s - rank_abs),
                    7 => temp_ranks[rank_id].set_rel7(num_1s - rank_abs),
                    _ => unreachable!(),
                }
            }

            let unit = self.units[unit_id];
            let unit_num_1s = popcount_unit(unit);

            if enables_select0 {
                let bits_remaining = num_bits - bit_id;
                let unit_num_0s = std::cmp::min(bits_remaining, WORD_SIZE) - unit_num_1s;

                // Wrapping negation to get modulo behavior
                let zero_bit_id = (0usize.wrapping_sub(num_0s)) % 512;
                if unit_num_0s > zero_bit_id {
                    // Use select_bit to find actual position of the zero_bit_id-th 0-bit
                    #[cfg(target_pointer_width = "64")]
                    let pos = select_bit_u64(zero_bit_id, bit_id, !unit);
                    #[cfg(target_pointer_width = "32")]
                    let pos = select_bit_u32(zero_bit_id, bit_id, !unit);
                    self.select0s.push_back(pos as u32);
                }

                num_0s += unit_num_0s;
            }

            if enables_select1 {
                let one_bit_id = (0usize.wrapping_sub(num_1s)) % 512;
                if unit_num_1s > one_bit_id {
                    // Use select_bit to find actual position of the one_bit_id-th 1-bit
                    #[cfg(target_pointer_width = "64")]
                    let pos = select_bit_u64(one_bit_id, bit_id, unit);
                    #[cfg(target_pointer_width = "32")]
                    let pos = select_bit_u32(one_bit_id, bit_id, unit);
                    self.select1s.push_back(pos as u32);
                }
            }

            num_1s += unit_num_1s;
        }

        // Fill in remaining relative ranks for partial last block
        if (num_bits % 512) != 0 {
            let rank_id = (num_bits - 1) / 512;
            let last_block_pos = ((num_bits - 1) / 64) % 8;
            let rank_abs = temp_ranks[rank_id].abs();
            let rel_value = num_1s - rank_abs;

            for rel_idx in (last_block_pos + 1)..=7 {
                match rel_idx {
                    1 => temp_ranks[rank_id].set_rel1(rel_value),
                    2 => temp_ranks[rank_id].set_rel2(rel_value),
                    3 => temp_ranks[rank_id].set_rel3(rel_value),
                    4 => temp_ranks[rank_id].set_rel4(rel_value),
                    5 => temp_ranks[rank_id].set_rel5(rel_value),
                    6 => temp_ranks[rank_id].set_rel6(rel_value),
                    7 => temp_ranks[rank_id].set_rel7(rel_value),
                    _ => {}
                }
            }
        }

        // Set final absolute rank
        if !temp_ranks.is_empty() {
            let last_idx = temp_ranks.len() - 1;
            temp_ranks[last_idx].set_abs(num_1s);
        }

        // Copy temp_ranks back to self.ranks
        for (i, rank) in temp_ranks.into_iter().enumerate() {
            self.ranks[i] = rank;
        }

        if enables_select0 {
            self.select0s.push_back(num_bits as u32);
            self.select0s.shrink();
        }
        if enables_select1 {
            self.select1s.push_back(num_bits as u32);
            self.select1s.shrink();
        }
    }

    /// Returns the position of the i-th 0-bit.
    ///
    /// # Arguments
    ///
    /// * `i` - The rank of the 0-bit to find (0-indexed)
    ///
    /// # Returns
    ///
    /// The position of the i-th 0-bit
    ///
    /// # Panics
    ///
    /// Panics if the select0 index is empty or if i >= num_0s()
    #[cfg(target_pointer_width = "64")]
    pub fn select0(&self, mut i: usize) -> usize {
        assert!(!self.select0s.empty(), "Select0 index not built");
        assert!(i < self.num_0s(), "Index out of bounds");

        let select_id = i / 512;
        assert!(select_id + 1 < self.select0s.size());

        // Fast path for exact 512-bit boundaries
        if (i % 512) == 0 {
            return self.select0s[select_id] as usize;
        }

        // Binary/linear search to find the rank block
        let mut begin = self.select0s[select_id] as usize / 512;
        let mut end = (self.select0s[select_id + 1] as usize + 511) / 512;

        if begin + 10 >= end {
            // Linear search for small ranges
            while i >= ((begin + 1) * 512) - self.ranks[begin + 1].abs() {
                begin += 1;
            }
        } else {
            // Binary search for large ranges
            while begin + 1 < end {
                let middle = (begin + end) / 2;
                if i < (middle * 512) - self.ranks[middle].abs() {
                    end = middle;
                } else {
                    begin = middle;
                }
            }
        }

        let rank_id = begin;
        i -= (rank_id * 512) - self.ranks[rank_id].abs();

        // Find the unit within the rank block using relative ranks
        let rank = &self.ranks[rank_id];
        let mut unit_id = rank_id * 8;

        if i < (256 - rank.rel4()) {
            if i < (128 - rank.rel2()) {
                if i >= (64 - rank.rel1()) {
                    unit_id += 1;
                    i -= 64 - rank.rel1();
                }
            } else if i < (192 - rank.rel3()) {
                unit_id += 2;
                i -= 128 - rank.rel2();
            } else {
                unit_id += 3;
                i -= 192 - rank.rel3();
            }
        } else if i < (384 - rank.rel6()) {
            if i < (320 - rank.rel5()) {
                unit_id += 4;
                i -= 256 - rank.rel4();
            } else {
                unit_id += 5;
                i -= 320 - rank.rel5();
            }
        } else if i < (448 - rank.rel7()) {
            unit_id += 6;
            i -= 384 - rank.rel6();
        } else {
            unit_id += 7;
            i -= 448 - rank.rel7();
        }

        // Use select_bit to find the exact position within the unit
        // For select0, we need to invert the bits
        select_bit_u64(i, unit_id * 64, !self.units[unit_id])
    }

    /// Returns the position of the i-th 1-bit.
    ///
    /// # Arguments
    ///
    /// * `i` - The rank of the 1-bit to find (0-indexed)
    ///
    /// # Returns
    ///
    /// The position of the i-th 1-bit
    ///
    /// # Panics
    ///
    /// Panics if the select1 index is empty or if i >= num_1s()
    #[cfg(target_pointer_width = "64")]
    pub fn select1(&self, mut i: usize) -> usize {
        assert!(!self.select1s.empty(), "Select1 index not built");
        assert!(i < self.num_1s(), "Index out of bounds");

        let select_id = i / 512;
        assert!(select_id + 1 < self.select1s.size());

        // Fast path for exact 512-bit boundaries
        if (i % 512) == 0 {
            return self.select1s[select_id] as usize;
        }

        // Binary/linear search to find the rank block
        let mut begin = self.select1s[select_id] as usize / 512;
        let mut end = (self.select1s[select_id + 1] as usize + 511) / 512;

        if begin + 10 >= end {
            // Linear search for small ranges
            while i >= self.ranks[begin + 1].abs() {
                begin += 1;
            }
        } else {
            // Binary search for large ranges
            while begin + 1 < end {
                let middle = (begin + end) / 2;
                if i < self.ranks[middle].abs() {
                    end = middle;
                } else {
                    begin = middle;
                }
            }
        }

        let rank_id = begin;
        i -= self.ranks[rank_id].abs();

        // Find the unit within the rank block using relative ranks
        let rank = &self.ranks[rank_id];
        let mut unit_id = rank_id * 8;

        if i < rank.rel4() {
            if i < rank.rel2() {
                if i >= rank.rel1() {
                    unit_id += 1;
                    i -= rank.rel1();
                }
            } else if i < rank.rel3() {
                unit_id += 2;
                i -= rank.rel2();
            } else {
                unit_id += 3;
                i -= rank.rel3();
            }
        } else if i < rank.rel6() {
            if i < rank.rel5() {
                unit_id += 4;
                i -= rank.rel4();
            } else {
                unit_id += 5;
                i -= rank.rel5();
            }
        } else if i < rank.rel7() {
            unit_id += 6;
            i -= rank.rel6();
        } else {
            unit_id += 7;
            i -= rank.rel7();
        }

        // Use select_bit to find the exact position within the unit
        select_bit_u64(i, unit_id * 64, self.units[unit_id])
    }

    // TODO: Implement 32-bit versions of select0() and select1()
    // TODO: Implement map(), read(), write() for serialization
}

// Note: We cannot implement Index<usize> for BitVector because
// Index::index() must return a reference, but we need to return
// a bool value. Use get() method instead.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_vector_new() {
        let bv = BitVector::new();
        assert_eq!(bv.size(), 0);
        assert!(bv.empty());
        assert_eq!(bv.num_0s(), 0);
        assert_eq!(bv.num_1s(), 0);
    }

    #[test]
    fn test_bit_vector_push_back() {
        let mut bv = BitVector::new();
        bv.push_back(true);
        bv.push_back(false);
        bv.push_back(true);
        bv.push_back(true);

        assert_eq!(bv.size(), 4);
        assert_eq!(bv.num_1s(), 3);
        assert_eq!(bv.num_0s(), 1);

        assert!(bv.get(0));
        assert!(!bv.get(1));
        assert!(bv.get(2));
        assert!(bv.get(3));
    }

    #[test]
    fn test_bit_vector_large() {
        let mut bv = BitVector::new();

        // Push 100 bits: alternating 1 and 0
        for i in 0..100 {
            bv.push_back(i % 2 == 0);
        }

        assert_eq!(bv.size(), 100);
        assert_eq!(bv.num_1s(), 50);
        assert_eq!(bv.num_0s(), 50);

        for i in 0..100 {
            assert_eq!(bv.get(i), i % 2 == 0);
        }
    }

    #[test]
    fn test_bit_vector_clear() {
        let mut bv = BitVector::new();
        bv.push_back(true);
        bv.push_back(false);

        bv.clear();
        assert_eq!(bv.size(), 0);
        assert!(bv.empty());
    }

    #[test]
    fn test_bit_vector_swap() {
        let mut bv1 = BitVector::new();
        let mut bv2 = BitVector::new();

        bv1.push_back(true);
        bv2.push_back(false);
        bv2.push_back(true);

        bv1.swap(&mut bv2);

        assert_eq!(bv1.size(), 2);
        assert_eq!(bv2.size(), 1);
        assert!(!bv1.get(0));
        assert!(bv1.get(1));
        assert!(bv2.get(0));
    }

    #[test]
    #[should_panic(expected = "Index out of bounds")]
    fn test_bit_vector_out_of_bounds() {
        let bv = BitVector::new();
        bv.get(0);
    }

    #[test]
    fn test_bit_vector_build_and_rank() {
        let mut bv = BitVector::new();

        // Create a pattern: 1100 1100 1100 ... (alternating pairs)
        for i in 0..100 {
            bv.push_back((i / 2) % 2 == 0);
        }

        // Build the rank index
        bv.build(false, false);

        // Test rank1 queries
        assert_eq!(bv.rank1(0), 0); // Before first bit
        assert_eq!(bv.rank1(1), 1); // After first 1
        assert_eq!(bv.rank1(2), 2); // After two 1s
        assert_eq!(bv.rank1(3), 2); // Still two 1s (positions 0-1 are 1, 2 is 0)
        assert_eq!(bv.rank1(4), 2); // Still two 1s
        assert_eq!(bv.rank1(6), 4); // Positions 0,1,4,5 are 1

        // Test rank0 queries
        assert_eq!(bv.rank0(0), 0);
        assert_eq!(bv.rank0(4), 2); // Positions 2,3 are 0
        assert_eq!(bv.rank0(6), 2); // Still positions 2,3 are 0

        // Verify total counts
        assert_eq!(bv.rank1(100), 50);
        assert_eq!(bv.rank0(100), 50);
    }

    #[test]
    fn test_bit_vector_build_all_ones() {
        let mut bv = BitVector::new();

        for _ in 0..64 {
            bv.push_back(true);
        }

        bv.build(false, false);

        assert_eq!(bv.rank1(0), 0);
        assert_eq!(bv.rank1(32), 32);
        assert_eq!(bv.rank1(64), 64);
        assert_eq!(bv.rank0(64), 0);
    }

    #[test]
    fn test_bit_vector_build_all_zeros() {
        let mut bv = BitVector::new();

        for _ in 0..64 {
            bv.push_back(false);
        }

        bv.build(false, false);

        assert_eq!(bv.rank1(0), 0);
        assert_eq!(bv.rank1(32), 0);
        assert_eq!(bv.rank1(64), 0);
        assert_eq!(bv.rank0(0), 0);
        assert_eq!(bv.rank0(32), 32);
        assert_eq!(bv.rank0(64), 64);
    }

    #[test]
    fn test_bit_vector_build_large() {
        let mut bv = BitVector::new();

        // Create a pattern with 1000 bits
        for i in 0..1000 {
            bv.push_back(i % 3 == 0); // Every 3rd bit is 1
        }

        bv.build(false, false);

        // Verify some rank queries
        let expected_rank1_at_300 = (0..300).filter(|&i| i % 3 == 0).count();
        assert_eq!(bv.rank1(300), expected_rank1_at_300);

        let expected_rank1_at_1000 = (0..1000).filter(|&i| i % 3 == 0).count();
        assert_eq!(bv.rank1(1000), expected_rank1_at_1000);
        assert_eq!(bv.rank0(1000), 1000 - expected_rank1_at_1000);
    }

    #[test]
    #[should_panic(expected = "Rank index not built")]
    fn test_bit_vector_rank_without_build() {
        let mut bv = BitVector::new();
        bv.push_back(true);
        bv.rank1(1); // Should panic - index not built
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_bit_vector_select1_basic() {
        let mut bv = BitVector::new();

        // Create pattern: 1001 0010 0100 1000 (bits at positions 0, 3, 6, 11, 15)
        bv.push_back(true);  // 0
        bv.push_back(false); // 1
        bv.push_back(false); // 2
        bv.push_back(true);  // 3
        bv.push_back(false); // 4
        bv.push_back(false); // 5
        bv.push_back(true);  // 6
        bv.push_back(false); // 7
        bv.push_back(false); // 8
        bv.push_back(true);  // 9
        bv.push_back(false); // 10
        bv.push_back(false); // 11

        bv.build(false, true);

        // Find positions of 1-bits
        assert_eq!(bv.select1(0), 0);
        assert_eq!(bv.select1(1), 3);
        assert_eq!(bv.select1(2), 6);
        assert_eq!(bv.select1(3), 9);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_bit_vector_select0_basic() {
        let mut bv = BitVector::new();

        // Create pattern: 1001 0010 0100 (0-bits at positions 1, 2, 4, 5, 7, 8, 10, 11)
        bv.push_back(true);  // 0
        bv.push_back(false); // 1
        bv.push_back(false); // 2
        bv.push_back(true);  // 3
        bv.push_back(false); // 4
        bv.push_back(false); // 5
        bv.push_back(true);  // 6
        bv.push_back(false); // 7
        bv.push_back(false); // 8
        bv.push_back(true);  // 9
        bv.push_back(false); // 10
        bv.push_back(false); // 11

        bv.build(true, false);

        // Find positions of 0-bits
        assert_eq!(bv.select0(0), 1);
        assert_eq!(bv.select0(1), 2);
        assert_eq!(bv.select0(2), 4);
        assert_eq!(bv.select0(3), 5);
        assert_eq!(bv.select0(4), 7);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_bit_vector_select1_large() {
        let mut bv = BitVector::new();

        // Create pattern with 1000 bits: every 3rd bit is 1
        for i in 0..1000 {
            bv.push_back(i % 3 == 0);
        }

        bv.build(false, true);

        // Verify select1 is the inverse of rank1
        let num_1s = bv.num_1s();
        for i in 0..num_1s {
            let pos = bv.select1(i);
            assert_eq!(bv.rank1(pos), i);
            assert_eq!(bv.rank1(pos + 1), i + 1);
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_bit_vector_select0_large() {
        let mut bv = BitVector::new();

        // Create pattern with 1000 bits: every 3rd bit is 1 (others are 0)
        for i in 0..1000 {
            bv.push_back(i % 3 == 0);
        }

        bv.build(true, false);

        // Verify select0 is the inverse of rank0
        let num_0s = bv.num_0s();
        for i in 0..num_0s {
            let pos = bv.select0(i);
            assert_eq!(bv.rank0(pos), i);
            assert_eq!(bv.rank0(pos + 1), i + 1);
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_bit_vector_select1_all_ones() {
        let mut bv = BitVector::new();

        for _ in 0..100 {
            bv.push_back(true);
        }

        bv.build(false, true);

        // For all 1s, select1(i) should return i
        for i in 0..100 {
            assert_eq!(bv.select1(i), i);
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn test_bit_vector_select0_all_zeros() {
        let mut bv = BitVector::new();

        for _ in 0..100 {
            bv.push_back(false);
        }

        bv.build(true, false);

        // For all 0s, select0(i) should return i
        for i in 0..100 {
            assert_eq!(bv.select0(i), i);
        }
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    #[should_panic(expected = "Select1 index not built")]
    fn test_bit_vector_select1_without_build() {
        let mut bv = BitVector::new();
        bv.push_back(true);
        bv.build(false, false); // Don't build select1 index
        bv.select1(0); // Should panic
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    #[should_panic(expected = "Select0 index not built")]
    fn test_bit_vector_select0_without_build() {
        let mut bv = BitVector::new();
        bv.push_back(false);
        bv.build(false, false); // Don't build select0 index
        bv.select0(0); // Should panic
    }
}
