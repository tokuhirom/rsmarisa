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

use super::pop_count::{popcount, popcount_u32, Unit};
use super::rank_index::RankIndex;
use super::vector::Vector;
use crate::base::{ErrorCode, WORD_SIZE};
use crate::grimoire::io::{Mapper, Reader, Writer};

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

    // TODO: Implement build(), rank0(), rank1(), select0(), select1()
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
}
