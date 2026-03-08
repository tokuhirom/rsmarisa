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
//!
//! Rank/select queries are handled by `vers-vecs::RsVec` internally.
//! Raw bit storage and the C++ binary format (units + rank/select indices)
//! are preserved for binary compatibility with the original C++ marisa-trie.

use super::rank_index::RankIndex;
use super::vector::Vector;
use crate::base::WORD_SIZE;
use vers_vecs::{BitVec, RsVec};

#[cfg(target_pointer_width = "64")]
type Unit = u64;
#[cfg(target_pointer_width = "32")]
type Unit = u32;

/// Finds the i-th set bit in `unit`, returning its absolute position.
#[inline]
fn select_nth_set_bit(i: usize, bit_id: usize, unit: Unit) -> usize {
    let mut w = unit as u64;
    for _ in 0..i {
        w &= w - 1; // clear the lowest set bit
    }
    bit_id + w.trailing_zeros() as usize
}

/// Bit vector supporting rank and select operations.
///
/// A bit vector that stores bits compactly and supports efficient
/// rank and select operations. Query operations (rank0, rank1, select0,
/// select1) are delegated to `vers-vecs::RsVec` after `build()` is called.
///
/// The C++ binary format (units + rank/select index vectors) is preserved
/// for binary compatibility with the original C++ marisa-trie library.
pub struct BitVector {
    /// Storage for bits, packed into Units (u32 or u64 depending on platform).
    /// Kept for C++ binary format serialization.
    units: Vector<Unit>,
    /// Number of bits currently stored.
    size: usize,
    /// Number of 1-bits in the vector.
    num_1s: usize,
    /// Rank index in C++ format (for serialization only).
    ranks: Vector<RankIndex>,
    /// Select index for 0-bits in C++ format (for serialization only).
    select0s: Vector<u32>,
    /// Select index for 1-bits in C++ format (for serialization only).
    select1s: Vector<u32>,
    /// vers-vecs succinct bit vector used for rank/select queries after build().
    rs_vec: Option<RsVec>,
    /// Whether the select0 index was requested in build().
    enables_select0: bool,
    /// Whether the select1 index was requested in build().
    enables_select1: bool,
}

impl Default for BitVector {
    fn default() -> Self {
        Self::new()
    }
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
            rs_vec: None,
            enables_select0: false,
            enables_select1: false,
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
        std::mem::swap(&mut self.rs_vec, &mut other.rs_vec);
        std::mem::swap(&mut self.enables_select0, &mut other.enables_select0);
        std::mem::swap(&mut self.enables_select1, &mut other.enables_select1);
    }

    /// Maps the bit vector from a mapper.
    ///
    /// Format (matching C++ marisa-trie):
    /// - units: `Vector<u64>`
    /// - size: u32
    /// - num_1s: u32
    /// - ranks: `Vector<RankIndex>`
    /// - select0s: `Vector<u32>`
    /// - select1s: `Vector<u32>`
    ///
    /// # Arguments
    ///
    /// * `mapper` - Mapper to read from
    ///
    /// # Errors
    ///
    /// Returns an error if mapping fails or if num_1s > size.
    pub fn map(&mut self, mapper: &mut crate::grimoire::io::Mapper) -> std::io::Result<()> {
        // Map units
        self.units.map(mapper)?;

        // Map size
        let temp_size: u32 = mapper.map_value()?;
        self.size = temp_size as usize;

        // Map num_1s and validate
        let temp_num_1s: u32 = mapper.map_value()?;
        if temp_num_1s as usize > self.size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "num_1s exceeds size",
            ));
        }
        self.num_1s = temp_num_1s as usize;

        // Map rank and select indices (kept for C++ format serialization)
        self.ranks.map(mapper)?;
        self.select0s.map(mapper)?;
        self.select1s.map(mapper)?;

        // Rebuild RsVec from raw bits for rank/select queries.
        // Select support is inferred from the C++ select index presence.
        self.enables_select0 = !self.select0s.empty();
        self.enables_select1 = !self.select1s.empty();
        self.rs_vec = Some(self.build_rs_vec());

        Ok(())
    }

    /// Reads the bit vector from a reader.
    ///
    /// Format (matching C++ marisa-trie):
    /// - units: `Vector<u64>`
    /// - size: u32
    /// - num_1s: u32
    /// - ranks: `Vector<RankIndex>`
    /// - select0s: `Vector<u32>`
    /// - select1s: `Vector<u32>`
    ///
    /// # Arguments
    ///
    /// * `reader` - Reader to read from
    ///
    /// # Errors
    ///
    /// Returns an error if reading fails or if num_1s > size.
    pub fn read(&mut self, reader: &mut crate::grimoire::io::Reader) -> std::io::Result<()> {
        // Read units
        self.units.read(reader)?;

        // Read size
        let temp_size: u32 = reader.read()?;
        self.size = temp_size as usize;

        // Read num_1s and validate
        let temp_num_1s: u32 = reader.read()?;
        if temp_num_1s as usize > self.size {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "num_1s exceeds size",
            ));
        }
        self.num_1s = temp_num_1s as usize;

        // Read rank and select indices (kept for C++ format serialization)
        self.ranks.read(reader)?;
        self.select0s.read(reader)?;
        self.select1s.read(reader)?;

        // Rebuild RsVec from raw bits for rank/select queries.
        // Select support is inferred from the C++ select index presence.
        self.enables_select0 = !self.select0s.empty();
        self.enables_select1 = !self.select1s.empty();
        self.rs_vec = Some(self.build_rs_vec());

        Ok(())
    }

    /// Writes the bit vector to a writer.
    ///
    /// Format (matching C++ marisa-trie):
    /// - units: `Vector<u64>`
    /// - size: u32
    /// - num_1s: u32
    /// - ranks: `Vector<RankIndex>`
    /// - select0s: `Vector<u32>`
    /// - select1s: `Vector<u32>`
    ///
    /// # Arguments
    ///
    /// * `writer` - Writer to write to
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails.
    pub fn write(&self, writer: &mut crate::grimoire::io::Writer) -> std::io::Result<()> {
        // Write units
        self.units.write(writer)?;

        // Write size and num_1s as u32
        writer.write(&(self.size as u32))?;
        writer.write(&(self.num_1s as u32))?;

        // Write rank and select indices
        self.ranks.write(writer)?;
        self.select0s.write(writer)?;
        self.select1s.write(writer)?;

        Ok(())
    }

    /// Disables the select0 index.
    #[inline]
    pub fn disable_select0(&mut self) {
        self.select0s.clear();
        self.enables_select0 = false;
    }

    /// Disables the select1 index.
    #[inline]
    pub fn disable_select1(&mut self) {
        self.select1s.clear();
        self.enables_select1 = false;
    }

    /// Returns the number of 0-bits in the range [0, i).
    ///
    /// # Panics
    ///
    /// Panics (in debug) if `build()` has not been called or if i > size().
    #[inline]
    pub fn rank0(&self, i: usize) -> usize {
        debug_assert!(i <= self.size, "Index out of bounds");
        debug_assert!(self.rs_vec.is_some(), "Rank index not built");
        // SAFETY: rs_vec is guaranteed to be Some after build() is called,
        // and build() must be called before any query operation.
        unsafe {
            self.rs_vec
                .as_ref()
                .unwrap_unchecked()
                .rank0(i)
        }
    }

    /// Returns the number of 1-bits in the range [0, i).
    ///
    /// # Panics
    ///
    /// Panics (in debug) if `build()` has not been called or if i > size().
    #[inline]
    pub fn rank1(&self, i: usize) -> usize {
        debug_assert!(i <= self.size, "Index out of bounds");
        debug_assert!(self.rs_vec.is_some(), "Rank index not built");
        // SAFETY: rs_vec is guaranteed to be Some after build() is called.
        unsafe {
            self.rs_vec
                .as_ref()
                .unwrap_unchecked()
                .rank1(i)
        }
    }

    /// Builds the rank and select indices.
    ///
    /// This must be called before using rank() or select() operations.
    ///
    /// # Arguments
    ///
    /// * `enables_select0` - Whether to build the select0 index
    /// * `enables_select1` - Whether to build the select1 index
    pub fn build(&mut self, enables_select0: bool, enables_select1: bool) {
        self.enables_select0 = enables_select0;
        self.enables_select1 = enables_select1;

        // Build C++ format rank/select indices for binary serialization.
        self.build_index_internal(enables_select0, enables_select1);

        // Shrink C++ format vectors to save memory
        self.units.shrink();
        self.ranks.shrink();
        if enables_select0 {
            self.select0s.shrink();
        }
        if enables_select1 {
            self.select1s.shrink();
        }

        // Build vers-vecs RsVec from raw bits for rank/select queries.
        self.rs_vec = Some(self.build_rs_vec());
    }

    /// Builds a `vers-vecs::RsVec` from the raw bit storage in `units`.
    ///
    /// This RsVec is used for all rank/select query operations.
    fn build_rs_vec(&self) -> RsVec {
        let mut bit_vec = BitVec::with_capacity(self.size);

        #[cfg(target_pointer_width = "64")]
        {
            let full_words = self.size / 64;
            let remaining = self.size % 64;
            for i in 0..full_words {
                bit_vec.append_word(self.units[i]);
            }
            if remaining > 0 {
                // Append only the valid remaining bits
                let last_word = self.units[full_words];
                for j in 0..remaining {
                    bit_vec.append_bit((last_word >> j) & 1);
                }
            }
        }

        #[cfg(target_pointer_width = "32")]
        {
            for i in 0..self.size {
                let unit = self.units[i / WORD_SIZE];
                bit_vec.append_bit(((unit >> (i % WORD_SIZE)) & 1) as u64);
            }
        }

        RsVec::from_bit_vec(bit_vec)
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

        for unit_id in 0..num_units {
            let bit_id = unit_id * WORD_SIZE;

            // Update rank index at 64-bit boundaries
            if (bit_id % 64) == 0 {
                let rank_id = bit_id / 512;
                let rank_abs = self.ranks[rank_id].abs();
                match (bit_id / 64) % 8 {
                    0 => self.ranks[rank_id].set_abs(num_1s),
                    1 => self.ranks[rank_id].set_rel1(num_1s - rank_abs),
                    2 => self.ranks[rank_id].set_rel2(num_1s - rank_abs),
                    3 => self.ranks[rank_id].set_rel3(num_1s - rank_abs),
                    4 => self.ranks[rank_id].set_rel4(num_1s - rank_abs),
                    5 => self.ranks[rank_id].set_rel5(num_1s - rank_abs),
                    6 => self.ranks[rank_id].set_rel6(num_1s - rank_abs),
                    7 => self.ranks[rank_id].set_rel7(num_1s - rank_abs),
                    _ => unreachable!(),
                }
            }

            let unit = self.units[unit_id];
            let unit_num_1s = unit.count_ones() as usize;

            if enables_select0 {
                let bits_remaining = num_bits - bit_id;
                let unit_num_0s = std::cmp::min(bits_remaining, WORD_SIZE) - unit_num_1s;

                // Wrapping negation to get modulo behavior
                let zero_bit_id = (0usize.wrapping_sub(num_0s)) % 512;
                if unit_num_0s > zero_bit_id {
                    let pos = select_nth_set_bit(zero_bit_id, bit_id, !unit);
                    self.select0s.push_back(pos as u32);
                }

                num_0s += unit_num_0s;
            }

            if enables_select1 {
                let one_bit_id = (0usize.wrapping_sub(num_1s)) % 512;
                if unit_num_1s > one_bit_id {
                    let pos = select_nth_set_bit(one_bit_id, bit_id, unit);
                    self.select1s.push_back(pos as u32);
                }
            }

            num_1s += unit_num_1s;
        }

        // Fill in remaining relative ranks for partial last block
        if (num_bits % 512) != 0 {
            let rank_id = (num_bits - 1) / 512;
            let last_block_pos = ((num_bits - 1) / 64) % 8;
            let rank_abs = self.ranks[rank_id].abs();
            let rel_value = num_1s - rank_abs;

            for rel_idx in (last_block_pos + 1)..=7 {
                match rel_idx {
                    1 => self.ranks[rank_id].set_rel1(rel_value),
                    2 => self.ranks[rank_id].set_rel2(rel_value),
                    3 => self.ranks[rank_id].set_rel3(rel_value),
                    4 => self.ranks[rank_id].set_rel4(rel_value),
                    5 => self.ranks[rank_id].set_rel5(rel_value),
                    6 => self.ranks[rank_id].set_rel6(rel_value),
                    7 => self.ranks[rank_id].set_rel7(rel_value),
                    _ => {}
                }
            }
        }

        // Set final absolute rank
        if self.ranks.size() > 0 {
            let last_idx = self.ranks.size() - 1;
            self.ranks[last_idx].set_abs(num_1s);
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
    /// Returns the position of the i-th 0-bit (0-indexed).
    ///
    /// # Panics
    ///
    /// Panics if the select0 index was not enabled in `build()`, or if
    /// `build()` has not been called.
    pub fn select0(&self, i: usize) -> usize {
        debug_assert!(
            self.enables_select0,
            "Select0 index not built"
        );
        // SAFETY: rs_vec is guaranteed to be Some after build() is called.
        unsafe {
            self.rs_vec
                .as_ref()
                .unwrap_unchecked()
                .select0(i)
        }
    }

    /// Returns the position of the i-th 1-bit (0-indexed).
    ///
    /// # Panics
    ///
    /// Panics if the select1 index was not enabled in `build()`, or if
    /// `build()` has not been called.
    pub fn select1(&self, i: usize) -> usize {
        debug_assert!(
            self.enables_select1,
            "Select1 index not built"
        );
        // SAFETY: rs_vec is guaranteed to be Some after build() is called.
        unsafe {
            self.rs_vec
                .as_ref()
                .unwrap_unchecked()
                .select1(i)
        }
    }
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

    #[test]
    fn test_bit_vector_select1_basic() {
        let mut bv = BitVector::new();

        // Create pattern: 1001 0010 0100 1000 (bits at positions 0, 3, 6, 11, 15)
        bv.push_back(true); // 0
        bv.push_back(false); // 1
        bv.push_back(false); // 2
        bv.push_back(true); // 3
        bv.push_back(false); // 4
        bv.push_back(false); // 5
        bv.push_back(true); // 6
        bv.push_back(false); // 7
        bv.push_back(false); // 8
        bv.push_back(true); // 9
        bv.push_back(false); // 10
        bv.push_back(false); // 11

        bv.build(false, true);

        // Find positions of 1-bits
        assert_eq!(bv.select1(0), 0);
        assert_eq!(bv.select1(1), 3);
        assert_eq!(bv.select1(2), 6);
        assert_eq!(bv.select1(3), 9);
    }

    #[test]
    fn test_bit_vector_select0_basic() {
        let mut bv = BitVector::new();

        // Create pattern: 1001 0010 0100 (0-bits at positions 1, 2, 4, 5, 7, 8, 10, 11)
        bv.push_back(true); // 0
        bv.push_back(false); // 1
        bv.push_back(false); // 2
        bv.push_back(true); // 3
        bv.push_back(false); // 4
        bv.push_back(false); // 5
        bv.push_back(true); // 6
        bv.push_back(false); // 7
        bv.push_back(false); // 8
        bv.push_back(true); // 9
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

    #[test]
    #[should_panic(expected = "Select1 index not built")]
    fn test_bit_vector_select1_without_build() {
        let mut bv = BitVector::new();
        bv.push_back(true);
        bv.build(false, false); // Don't build select1 index
        bv.select1(0); // Should panic
    }

    #[test]
    #[should_panic(expected = "Select0 index not built")]
    fn test_bit_vector_select0_without_build() {
        let mut bv = BitVector::new();
        bv.push_back(false);
        bv.build(false, false); // Don't build select0 index
        bv.select0(0); // Should panic
    }

    #[test]
    fn test_bit_vector_write_read() {
        // Rust-specific: Test BitVector serialization
        use crate::grimoire::io::{Reader, Writer};

        let mut bv = BitVector::new();
        for i in 0..100 {
            bv.push_back(i % 3 == 0);
        }
        bv.build(true, true);

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        bv.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut bv2 = BitVector::new();
        bv2.read(&mut reader).unwrap();

        // Verify
        assert_eq!(bv2.size(), 100);
        assert_eq!(bv2.num_1s(), 34); // 100/3 rounded up
        for i in 0..100 {
            assert_eq!(bv2.get(i), i % 3 == 0);
        }

        // Verify rank operations work
        for i in 0..=100 {
            assert_eq!(bv2.rank1(i), bv.rank1(i));
        }
    }

    #[test]
    fn test_bit_vector_write_read_empty() {
        // Rust-specific: Test empty BitVector serialization
        use crate::grimoire::io::{Reader, Writer};

        let bv = BitVector::new();

        // Write to buffer
        let mut writer = Writer::from_vec(Vec::new());
        bv.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Read back
        let mut reader = Reader::from_bytes(&data);
        let mut bv2 = BitVector::new();
        bv2.read(&mut reader).unwrap();

        assert_eq!(bv2.size(), 0);
        assert_eq!(bv2.num_1s(), 0);
        assert!(bv2.empty());
    }

    #[test]
    fn test_bit_vector_read_invalid_num_1s() {
        // Rust-specific: Test validation of num_1s <= size
        use crate::grimoire::io::{Reader, Writer};

        // Create invalid data where num_1s > size
        let mut writer = Writer::from_vec(Vec::new());

        // Write empty units vector
        let empty_vec: crate::grimoire::vector::vector::Vector<u64> =
            crate::grimoire::vector::vector::Vector::new();
        empty_vec.write(&mut writer).unwrap();

        // Write size = 10, num_1s = 20 (invalid!)
        writer.write(&10u32).unwrap();
        writer.write(&20u32).unwrap();

        // Write empty rank/select vectors
        let empty_ranks: crate::grimoire::vector::vector::Vector<super::RankIndex> =
            crate::grimoire::vector::vector::Vector::new();
        empty_ranks.write(&mut writer).unwrap();

        let empty_u32: crate::grimoire::vector::vector::Vector<u32> =
            crate::grimoire::vector::vector::Vector::new();
        empty_u32.write(&mut writer).unwrap();
        empty_u32.write(&mut writer).unwrap();

        let data = writer.into_inner().unwrap();

        // Try to read - should fail
        let mut reader = Reader::from_bytes(&data);
        let mut bv = BitVector::new();
        let result = bv.read(&mut reader);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
    }
}
